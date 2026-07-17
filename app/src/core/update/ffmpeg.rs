use std::path::PathBuf;

use app_json_settings::ConfigManager;
use arama_env::ffmpeg_location::FfmpegLocationPreference;
use arama_sidecar::media::video::video_engine::discovery::{
    FfmpegDiscoveryEvent, FfmpegDiscoveryFailure, FfmpegDiscoveryOutcome, FfmpegDiscoveryRuntime,
    FfmpegDiscoveryTicket, FilesystemIssue, PreferenceRetainReason, PreferenceTransition,
    SearchLimit, clear_selection, prepare_selection, publish_validated_selection,
};
use iced::Task;

use super::super::{
    App,
    message::{FfmpegRequestIntent, Message},
};

pub(crate) mod state;
use state::{
    AuthorityStatus, AuthorityTerminal, FfmpegAuthority, RollbackAction, SelectionResolution,
};

#[derive(Clone, Debug, PartialEq, Eq)]
enum SelectionPickPlan {
    Cancelled,
    Rejected {
        preference: FfmpegLocationPreference,
        outcome: FfmpegDiscoveryOutcome,
    },
    Request(FfmpegLocationPreference),
}

pub(crate) fn request_task(
    runtime: &FfmpegDiscoveryRuntime,
    preference: FfmpegLocationPreference,
    epoch: u64,
) -> Task<Message> {
    next_event_task(epoch, runtime.request(preference))
}

fn next_event_task(epoch: u64, ticket: FfmpegDiscoveryTicket) -> Task<Message> {
    let waiting = ticket.clone();
    Task::perform(async move { waiting.next().await }, move |event| {
        Message::FfmpegDiscoveryEvent {
            epoch,
            ticket,
            event,
        }
    })
}

impl App {
    pub(super) fn request_current_ffmpeg(&mut self, intent: FfmpegRequestIntent) -> Task<Message> {
        let preference = self.settings.ffmpeg_location.clone();
        let epoch = self.ffmpeg_authority.begin(intent, preference.clone());
        self.settings_page.set_ffmpeg_select_enabled(true);
        self.publish_ffmpeg_checking();
        request_task(&self.ffmpeg_runtime, preference, epoch)
    }

    pub(super) fn handle_ffmpeg_directory_picked(
        &mut self,
        picker_epoch: u64,
        directory: Option<PathBuf>,
    ) -> Task<Message> {
        let current = self.settings.ffmpeg_location.clone();
        let Some(plan) = run_current_picker(&mut self.ffmpeg_authority, picker_epoch, || {
            plan_selection_pick(&current, directory)
        }) else {
            return Task::none();
        };
        self.settings_page.set_ffmpeg_select_enabled(true);
        match plan {
            SelectionPickPlan::Request(preference) => {
                let epoch = self
                    .ffmpeg_authority
                    .begin(FfmpegRequestIntent::Selection, preference.clone());
                self.settings_page.set_ffmpeg_select_enabled(false);
                self.settings_page
                    .set_ffmpeg_candidate_checking(&preference);
                request_task(&self.ffmpeg_runtime, preference, epoch)
            }
            SelectionPickPlan::Rejected {
                preference,
                outcome,
            } => {
                self.settings_page
                    .set_ffmpeg_candidate_failure(&preference, &outcome);
                self.push_error_toast(
                    "FFmpeg settings",
                    "The selected folder cannot be stored safely. Choose a valid absolute folder."
                        .to_owned(),
                );
                Task::none()
            }
            SelectionPickPlan::Cancelled => Task::none(),
        }
    }

    pub(super) fn clear_ffmpeg_selection(&mut self) -> Task<Message> {
        self.ffmpeg_authority.invalidate_picker();
        let abandoned = self.ffmpeg_authority.abandon_selection();
        self.settings_page.set_ffmpeg_select_enabled(true);
        let current = self.settings.ffmpeg_location.clone();
        let manager = ConfigManager::new().at_current_dir();
        match clear_selection(&mut self.settings, &current, |settings| {
            manager.save(settings)
        }) {
            PreferenceTransition::PublishedAuto => {
                self.request_current_ffmpeg(FfmpegRequestIntent::ClearToAuto)
            }
            PreferenceTransition::Retained { .. } => {
                let resume = if let Some(resolution) = abandoned {
                    self.publish_ffmpeg_authority();
                    self.resume_after_rollback(resolution)
                } else {
                    Task::none()
                };
                self.push_error_toast(
                    "FFmpeg settings",
                    "The automatic-discovery setting could not be saved.".to_owned(),
                );
                resume
            }
            PreferenceTransition::PublishedReady { .. } => unreachable!(),
        }
    }

    pub(super) fn handle_ffmpeg_discovery_event(
        &mut self,
        epoch: u64,
        ticket: FfmpegDiscoveryTicket,
        event: Option<FfmpegDiscoveryEvent>,
    ) -> Task<Message> {
        if !self.ffmpeg_authority.is_current(epoch) {
            return Task::none();
        }
        let Some(event) = event else {
            return self.handle_ffmpeg_stream_closed(epoch);
        };
        match event {
            FfmpegDiscoveryEvent::Started(_) => {
                if self.ffmpeg_authority.intent(epoch) != Some(FfmpegRequestIntent::Selection) {
                    self.publish_ffmpeg_checking();
                }
                next_event_task(epoch, ticket)
            }
            FfmpegDiscoveryEvent::Superseded => Task::none(),
            FfmpegDiscoveryEvent::Published(publication) => {
                if matches!(
                    publication.outcome,
                    FfmpegDiscoveryOutcome::SearchLimitReached(SearchLimit::WorkerDraining)
                ) {
                    self.ffmpeg_authority.mark_draining(epoch);
                    if self.ffmpeg_authority.intent(epoch) == Some(FfmpegRequestIntent::Selection) {
                        if let Some(preference) =
                            self.ffmpeg_authority.request_preference(epoch).cloned()
                        {
                            self.settings_page
                                .set_ffmpeg_candidate_checking(&preference);
                        }
                    } else {
                        self.setup.set_ffmpeg_draining();
                        self.settings_page
                            .set_ffmpeg_draining(self.ffmpeg_authority.preference().clone());
                    }
                    return next_event_task(epoch, ticket);
                }
                let outcome = publication.outcome;
                if self.ffmpeg_authority.intent(epoch) == Some(FfmpegRequestIntent::Selection) {
                    let candidate = self.ffmpeg_authority.request_preference(epoch).cloned();
                    let resolution = self.ffmpeg_authority.retain_prior(epoch);
                    self.settings_page.set_ffmpeg_select_enabled(true);
                    self.publish_ffmpeg_authority();
                    if let Some(candidate) = candidate {
                        self.settings_page
                            .set_ffmpeg_candidate_failure(&candidate, &outcome);
                    }
                    return self.resume_after_rollback(resolution);
                }
                let terminal = match &outcome {
                    FfmpegDiscoveryOutcome::Ready { toolchain, .. } => {
                        AuthorityTerminal::Ready(toolchain.clone())
                    }
                    _ => AuthorityTerminal::Failed(
                        FfmpegDiscoveryFailure::try_from(outcome.clone())
                            .expect("non-Ready discovery outcome is a failure"),
                    ),
                };
                self.ffmpeg_authority.publish_terminal(epoch, terminal);
                let ready = matches!(outcome, FfmpegDiscoveryOutcome::Ready { .. });
                self.setup.set_ffmpeg_ready(ready);
                self.settings_page
                    .set_ffmpeg_outcome(self.ffmpeg_authority.preference().clone(), &outcome);
                Task::none()
            }
            FfmpegDiscoveryEvent::SelectedReady { validated, .. } => {
                if self.ffmpeg_authority.intent(epoch) != Some(FfmpegRequestIntent::Selection) {
                    return Task::none();
                }
                let Some(prior) = self.ffmpeg_authority.prior_preference(epoch).cloned() else {
                    return Task::none();
                };
                let manager = ConfigManager::new().at_current_dir();
                let Some(transition) = run_current_selection(&self.ffmpeg_authority, epoch, || {
                    publish_validated_selection(&mut self.settings, &prior, validated, |settings| {
                        manager.save(settings)
                    })
                }) else {
                    return Task::none();
                };
                match transition {
                    PreferenceTransition::PublishedReady {
                        preference,
                        outcome: FfmpegDiscoveryOutcome::Ready { toolchain, .. },
                    } => {
                        self.ffmpeg_authority
                            .finish_selection_persistence::<()>(epoch, Ok((preference, toolchain)));
                        self.publish_ffmpeg_authority();
                        self.settings_page.set_ffmpeg_select_enabled(true);
                        Task::none()
                    }
                    PreferenceTransition::Retained { .. } => {
                        let resolution = self
                            .ffmpeg_authority
                            .finish_selection_persistence::<()>(epoch, Err(()));
                        self.publish_ffmpeg_authority();
                        self.settings_page.set_ffmpeg_select_enabled(true);
                        self.push_error_toast(
                            "FFmpeg settings",
                            "The validated FFmpeg folder could not be saved.".to_owned(),
                        );
                        self.resume_after_rollback(resolution)
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    fn publish_ffmpeg_checking(&mut self) {
        self.setup.set_ffmpeg_checking();
        self.settings_page
            .set_ffmpeg_checking(self.ffmpeg_authority.preference().clone());
    }

    fn publish_ffmpeg_authority(&mut self) {
        let preference = self.ffmpeg_authority.preference().clone();
        match self.ffmpeg_authority.status().clone() {
            AuthorityStatus::Ready => {
                self.setup.set_ffmpeg_ready(true);
                self.settings_page.set_ffmpeg_ready(preference, true);
            }
            AuthorityStatus::Failed(failure) => {
                self.setup.set_ffmpeg_ready(false);
                self.settings_page
                    .set_ffmpeg_outcome(preference, &failure.into_outcome());
            }
            AuthorityStatus::Checking => self.publish_ffmpeg_checking(),
            AuthorityStatus::WorkerDraining => {
                self.setup.set_ffmpeg_draining();
                self.settings_page.set_ffmpeg_draining(preference);
            }
        }
    }

    fn resume_after_rollback(&mut self, resolution: SelectionResolution) -> Task<Message> {
        let Some(preference) = rollback_revalidation_preference(&resolution).cloned() else {
            return Task::none();
        };
        let epoch = self
            .ffmpeg_authority
            .begin(FfmpegRequestIntent::Recheck, preference.clone());
        request_task(&self.ffmpeg_runtime, preference, epoch)
    }

    fn handle_ffmpeg_stream_closed(&mut self, epoch: u64) -> Task<Message> {
        let outcome =
            FfmpegDiscoveryOutcome::FilesystemUnavailable(FilesystemIssue::MetadataOrIdentity);
        let task = if self.ffmpeg_authority.intent(epoch) == Some(FfmpegRequestIntent::Selection) {
            let candidate = self.ffmpeg_authority.request_preference(epoch).cloned();
            let resolution = self.ffmpeg_authority.retain_prior(epoch);
            self.settings_page.set_ffmpeg_select_enabled(true);
            self.publish_ffmpeg_authority();
            if let Some(candidate) = candidate {
                self.settings_page
                    .set_ffmpeg_candidate_failure(&candidate, &outcome);
            }
            self.resume_after_rollback(resolution)
        } else {
            self.ffmpeg_authority.publish_terminal(
                epoch,
                AuthorityTerminal::Failed(
                    FfmpegDiscoveryFailure::try_from(outcome.clone())
                        .expect("internal stream closure is a discovery failure"),
                ),
            );
            self.setup.set_ffmpeg_ready(false);
            self.settings_page
                .set_ffmpeg_outcome(self.ffmpeg_authority.preference().clone(), &outcome);
            Task::none()
        };
        self.push_error_toast(
            "FFmpeg settings",
            "The FFmpeg validation worker stopped unexpectedly. Re-check the setting.".to_owned(),
        );
        task
    }

    pub(super) fn pick_ffmpeg_directory(&mut self) -> Task<Message> {
        let Some(picker_epoch) = self.ffmpeg_authority.begin_picker() else {
            return Task::none();
        };
        self.settings_page.set_ffmpeg_select_enabled(false);
        Task::perform(
            async {
                rfd::AsyncFileDialog::new()
                    .pick_folder()
                    .await
                    .map(|handle| handle.path().to_path_buf())
            },
            move |directory| Message::FfmpegDirectoryPicked {
                picker_epoch,
                directory,
            },
        )
    }
}

fn rollback_revalidation_preference(
    resolution: &SelectionResolution,
) -> Option<&FfmpegLocationPreference> {
    let SelectionResolution::Retained(RollbackAction::Revalidate(preference)) = resolution else {
        return None;
    };
    Some(preference)
}

fn plan_selection_pick(
    current: &FfmpegLocationPreference,
    directory: Option<PathBuf>,
) -> SelectionPickPlan {
    let picked_preference = directory
        .as_ref()
        .map(|path| FfmpegLocationPreference::SelectedDirectory(path.clone()));
    match prepare_selection(current, directory) {
        Ok(candidate) => SelectionPickPlan::Request(candidate.preference().clone()),
        Err(PreferenceTransition::Retained {
            reason: PreferenceRetainReason::PickerCancelled,
            ..
        }) => SelectionPickPlan::Cancelled,
        Err(PreferenceTransition::Retained {
            reason: PreferenceRetainReason::PersistencePreflight,
            candidate_outcome: Some(outcome),
            ..
        }) => SelectionPickPlan::Rejected {
            preference: picked_preference.expect("preflight rejection has a picked path"),
            outcome,
        },
        Err(_) => SelectionPickPlan::Cancelled,
    }
}

fn run_current_selection<T: Clone, R>(
    authority: &FfmpegAuthority<T>,
    epoch: u64,
    operation: impl FnOnce() -> R,
) -> Option<R> {
    (authority.intent(epoch) == Some(FfmpegRequestIntent::Selection)).then(operation)
}

fn run_current_picker<T: Clone, R>(
    authority: &mut FfmpegAuthority<T>,
    picker_epoch: u64,
    operation: impl FnOnce() -> R,
) -> Option<R> {
    authority.accept_picker(picker_epoch).then(operation)
}

#[cfg(test)]
mod tests;
