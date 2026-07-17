use arama_env::ffmpeg_location::FfmpegLocationPreference;
use arama_sidecar::media::video::video_engine::discovery::FfmpegDiscoveryFailure;

use crate::core::message::FfmpegRequestIntent;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum AuthorityStatus {
    Checking,
    WorkerDraining,
    Ready,
    Failed(FfmpegDiscoveryFailure),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum AuthorityTerminal<T> {
    Ready(T),
    Failed(FfmpegDiscoveryFailure),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum RollbackAction {
    Restored,
    Revalidate(FfmpegLocationPreference),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum SelectionResolution {
    Published,
    Retained(RollbackAction),
    Stale,
}

#[derive(Clone, Debug)]
struct PublishedAuthority<T> {
    preference: FfmpegLocationPreference,
    status: AuthorityStatus,
    toolchain: Option<T>,
}

#[derive(Clone, Debug)]
struct Transaction<T> {
    epoch: u64,
    intent: FfmpegRequestIntent,
    requested_preference: FfmpegLocationPreference,
    prior: PublishedAuthority<T>,
}

#[derive(Clone, Debug)]
pub(crate) struct FfmpegAuthority<T> {
    next_epoch: u64,
    published: PublishedAuthority<T>,
    transaction: Option<Transaction<T>>,
    active_picker: Option<u64>,
}

impl<T: Clone> FfmpegAuthority<T> {
    pub(crate) fn new(preference: FfmpegLocationPreference) -> Self {
        Self {
            next_epoch: 0,
            published: PublishedAuthority {
                preference,
                status: AuthorityStatus::Checking,
                toolchain: None,
            },
            transaction: None,
            active_picker: None,
        }
    }

    pub(super) fn begin_picker(&mut self) -> Option<u64> {
        if self.active_picker.is_some() || self.intent_is(FfmpegRequestIntent::Selection) {
            return None;
        }
        let epoch = self.allocate_epoch();
        self.active_picker = Some(epoch);
        Some(epoch)
    }

    pub(super) fn accept_picker(&mut self, epoch: u64) -> bool {
        if self.active_picker != Some(epoch) {
            return false;
        }
        self.active_picker = None;
        true
    }

    pub(super) fn invalidate_picker(&mut self) {
        self.active_picker = None;
    }

    pub(super) fn abandon_selection(&mut self) -> Option<SelectionResolution> {
        if !self.intent_is(FfmpegRequestIntent::Selection) {
            return None;
        }
        let transaction = self.transaction.take().expect("active selection exists");
        Some(SelectionResolution::Retained(
            self.restore_prior(transaction.prior),
        ))
    }

    #[cfg(test)]
    pub(super) fn picker_active(&self) -> bool {
        self.active_picker.is_some()
    }

    pub(crate) fn begin(
        &mut self,
        intent: FfmpegRequestIntent,
        requested_preference: FfmpegLocationPreference,
    ) -> u64 {
        let epoch = self.allocate_epoch();
        self.active_picker = None;
        let prior = self.published.clone();
        if intent != FfmpegRequestIntent::Selection {
            self.published = PublishedAuthority {
                preference: requested_preference.clone(),
                status: AuthorityStatus::Checking,
                toolchain: None,
            };
        }
        self.transaction = Some(Transaction {
            epoch,
            intent,
            requested_preference,
            prior,
        });
        epoch
    }

    pub(super) fn is_current(&self, epoch: u64) -> bool {
        self.transaction
            .as_ref()
            .is_some_and(|transaction| transaction.epoch == epoch)
    }

    pub(super) fn intent(&self, epoch: u64) -> Option<FfmpegRequestIntent> {
        self.transaction
            .as_ref()
            .filter(|transaction| transaction.epoch == epoch)
            .map(|transaction| transaction.intent)
    }

    pub(super) fn request_preference(&self, epoch: u64) -> Option<&FfmpegLocationPreference> {
        self.transaction
            .as_ref()
            .filter(|transaction| transaction.epoch == epoch)
            .map(|transaction| &transaction.requested_preference)
    }

    pub(super) fn prior_preference(&self, epoch: u64) -> Option<&FfmpegLocationPreference> {
        self.transaction
            .as_ref()
            .filter(|transaction| transaction.epoch == epoch)
            .map(|transaction| &transaction.prior.preference)
    }

    pub(super) fn mark_draining(&mut self, epoch: u64) {
        if self
            .intent(epoch)
            .is_some_and(|intent| intent != FfmpegRequestIntent::Selection)
        {
            self.published.status = AuthorityStatus::WorkerDraining;
        }
    }

    pub(super) fn publish_terminal(&mut self, epoch: u64, terminal: AuthorityTerminal<T>) -> bool {
        if !self.is_current(epoch) {
            return false;
        }
        let transaction = self.transaction.take().expect("current transaction exists");
        if transaction.intent == FfmpegRequestIntent::Selection {
            self.published = transaction.prior;
            return false;
        }
        self.published.preference = transaction.requested_preference;
        match terminal {
            AuthorityTerminal::Ready(toolchain) => {
                self.published.status = AuthorityStatus::Ready;
                self.published.toolchain = Some(toolchain);
            }
            AuthorityTerminal::Failed(failure) => {
                self.published.status = AuthorityStatus::Failed(failure);
                self.published.toolchain = None;
            }
        }
        true
    }

    pub(super) fn finish_selection_persistence<E>(
        &mut self,
        epoch: u64,
        result: Result<(FfmpegLocationPreference, T), E>,
    ) -> SelectionResolution {
        if self.intent(epoch) != Some(FfmpegRequestIntent::Selection) {
            return SelectionResolution::Stale;
        }
        let transaction = self.transaction.take().expect("current transaction exists");
        match result {
            Ok((preference, toolchain)) => {
                self.published = PublishedAuthority {
                    preference,
                    status: AuthorityStatus::Ready,
                    toolchain: Some(toolchain),
                };
                SelectionResolution::Published
            }
            Err(_) => SelectionResolution::Retained(self.restore_prior(transaction.prior)),
        }
    }

    pub(super) fn retain_prior(&mut self, epoch: u64) -> SelectionResolution {
        if self.intent(epoch) != Some(FfmpegRequestIntent::Selection) {
            return SelectionResolution::Stale;
        }
        let transaction = self.transaction.take().expect("current transaction exists");
        SelectionResolution::Retained(self.restore_prior(transaction.prior))
    }

    pub(crate) fn toolchain(&self) -> Option<&T> {
        self.published.toolchain.as_ref()
    }

    pub(super) fn preference(&self) -> &FfmpegLocationPreference {
        &self.published.preference
    }

    pub(super) fn status(&self) -> &AuthorityStatus {
        &self.published.status
    }

    fn restore_prior(&mut self, prior: PublishedAuthority<T>) -> RollbackAction {
        let action = match &prior.status {
            AuthorityStatus::Checking | AuthorityStatus::WorkerDraining => {
                RollbackAction::Revalidate(prior.preference.clone())
            }
            AuthorityStatus::Ready | AuthorityStatus::Failed(_) => RollbackAction::Restored,
        };
        self.published = prior;
        action
    }

    fn intent_is(&self, intent: FfmpegRequestIntent) -> bool {
        self.transaction
            .as_ref()
            .is_some_and(|transaction| transaction.intent == intent)
    }

    fn allocate_epoch(&mut self) -> u64 {
        self.next_epoch = self
            .next_epoch
            .checked_add(1)
            .expect("ffmpeg App intent epoch exhausted");
        self.next_epoch
    }
}

#[cfg(test)]
mod tests;
