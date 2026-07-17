use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use arama_env::ffmpeg_location::FfmpegLocationPreference;

use super::{FfmpegDiscoveryOutcome, SearchLimit};

#[derive(Clone, Debug)]
pub struct DiscoveryWork {
    pub generation: u64,
    pub preference: FfmpegLocationPreference,
    pub(super) cancellation: Arc<AtomicBool>,
}

impl DiscoveryWork {
    pub fn is_cancelled(&self) -> bool {
        self.cancellation.load(Ordering::Acquire)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoordinatorPublication {
    pub generation: u64,
    pub outcome: FfmpegDiscoveryOutcome,
}

#[derive(Clone, Debug)]
pub struct DiscoveryRequestDisposition {
    pub generation: u64,
    pub work: Option<DiscoveryWork>,
    pub publication: Option<CoordinatorPublication>,
}

#[derive(Debug)]
struct ActiveWork {
    work: DiscoveryWork,
    deadline_published: bool,
}

#[derive(Debug, Default)]
pub struct FfmpegDiscoveryCoordinator {
    latest_generation: u64,
    active: Option<ActiveWork>,
    pending: Option<DiscoveryWork>,
}

impl FfmpegDiscoveryCoordinator {
    pub fn request(&mut self, preference: FfmpegLocationPreference) -> DiscoveryRequestDisposition {
        self.latest_generation = self
            .latest_generation
            .checked_add(1)
            .expect("ffmpeg discovery generation exhausted");
        let work = DiscoveryWork {
            generation: self.latest_generation,
            preference,
            cancellation: Arc::new(AtomicBool::new(false)),
        };
        let Some(active) = &self.active else {
            self.active = Some(ActiveWork {
                work: work.clone(),
                deadline_published: false,
            });
            return DiscoveryRequestDisposition {
                generation: work.generation,
                work: Some(work),
                publication: None,
            };
        };

        active.work.cancellation.store(true, Ordering::Release);
        self.pending = Some(work.clone());
        let publication = active.deadline_published.then_some(CoordinatorPublication {
            generation: work.generation,
            outcome: FfmpegDiscoveryOutcome::SearchLimitReached(SearchLimit::WorkerDraining),
        });
        DiscoveryRequestDisposition {
            generation: work.generation,
            work: None,
            publication,
        }
    }

    /// Publish the coordinator deadline even if blocking work has not returned.
    pub fn deadline_elapsed(&mut self, generation: u64) -> Option<CoordinatorPublication> {
        let active = self.active.as_mut()?;
        if active.work.generation != generation || active.deadline_published {
            return None;
        }
        active.work.cancellation.store(true, Ordering::Release);
        active.deadline_published = true;
        let pending_is_latest = self.pending.is_some();
        Some(CoordinatorPublication {
            generation: if pending_is_latest {
                self.latest_generation
            } else {
                generation
            },
            outcome: FfmpegDiscoveryOutcome::SearchLimitReached(if pending_is_latest {
                SearchLimit::WorkerDraining
            } else {
                SearchLimit::WholeAttempt
            }),
        })
    }

    /// Drain one worker completion and start only the newest pending request.
    pub fn worker_completed(
        &mut self,
        generation: u64,
        outcome: FfmpegDiscoveryOutcome,
    ) -> DiscoveryRequestDisposition {
        let Some(active) = self.active.take() else {
            return DiscoveryRequestDisposition {
                generation,
                work: None,
                publication: None,
            };
        };
        if active.work.generation != generation {
            self.active = Some(active);
            return DiscoveryRequestDisposition {
                generation,
                work: None,
                publication: None,
            };
        }

        let publication = (!active.deadline_published
            && generation == self.latest_generation
            && self.pending.is_none())
        .then_some(CoordinatorPublication {
            generation,
            outcome,
        });
        let work = self.pending.take();
        if let Some(work) = &work {
            self.active = Some(ActiveWork {
                work: work.clone(),
                deadline_published: false,
            });
        }
        DiscoveryRequestDisposition {
            generation,
            work,
            publication,
        }
    }

    pub fn active_generation(&self) -> Option<u64> {
        self.active.as_ref().map(|active| active.work.generation)
    }
}

#[cfg(test)]
mod tests;
