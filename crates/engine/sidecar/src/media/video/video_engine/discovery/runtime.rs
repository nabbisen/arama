use std::{
    fmt,
    sync::{Arc, Mutex, mpsc},
    time::Instant,
};

use arama_env::ffmpeg_location::FfmpegLocationPreference;

use super::{
    CoordinatorPublication, DiscoveryWork, FfmpegDiscoveryCoordinator, FfmpegLocatorPolicy,
    ValidatedSelection,
    worker::{WorkerCompletion, run_discovery_work},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FfmpegDiscoveryEvent {
    Started(u64),
    Published(CoordinatorPublication),
    SelectedReady {
        generation: u64,
        validated: ValidatedSelection,
    },
    Superseded,
}

/// A sequential, single-consumer event stream.
///
/// Clones share one receiver; they do not subscribe independently. Callers may
/// move a clone between sequential wait tasks, but must not poll clones
/// concurrently.
#[derive(Clone)]
pub struct FfmpegDiscoveryTicket {
    receiver: Arc<Mutex<mpsc::Receiver<FfmpegDiscoveryEvent>>>,
}

impl fmt::Debug for FfmpegDiscoveryTicket {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("FfmpegDiscoveryTicket").finish()
    }
}

impl FfmpegDiscoveryTicket {
    pub async fn next(&self) -> Option<FfmpegDiscoveryEvent> {
        let receiver = self.receiver.clone();
        tokio::task::spawn_blocking(move || receiver.lock().ok()?.recv().ok())
            .await
            .ok()
            .flatten()
    }

    #[cfg(test)]
    fn next_blocking(&self) -> Option<FfmpegDiscoveryEvent> {
        self.receiver.lock().ok()?.recv().ok()
    }
}

type Worker = Arc<dyn Fn(DiscoveryWork, FfmpegLocatorPolicy) -> WorkerCompletion + Send + Sync>;

#[derive(Clone, Debug)]
pub struct FfmpegDiscoveryRuntime {
    sender: mpsc::Sender<ActorEvent>,
}

impl FfmpegDiscoveryRuntime {
    pub fn new(policy: FfmpegLocatorPolicy) -> Self {
        Self::new_with_worker(policy, Arc::new(run_discovery_work))
    }

    fn new_with_worker(policy: FfmpegLocatorPolicy, worker: Worker) -> Self {
        let (sender, receiver) = mpsc::channel();
        let actor_sender = sender.clone();
        std::thread::Builder::new()
            .name("arama-ffmpeg-locator".to_owned())
            .spawn(move || run_actor(receiver, actor_sender, policy, worker))
            .expect("failed to start ffmpeg locator coordinator");
        Self { sender }
    }

    pub fn request(&self, preference: FfmpegLocationPreference) -> FfmpegDiscoveryTicket {
        let (sender, receiver) = mpsc::channel();
        let _ = self.sender.send(ActorEvent::Request { preference, sender });
        FfmpegDiscoveryTicket {
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }
}

impl Default for FfmpegDiscoveryRuntime {
    fn default() -> Self {
        Self::new(FfmpegLocatorPolicy::default())
    }
}

enum ActorEvent {
    Request {
        preference: FfmpegLocationPreference,
        sender: mpsc::Sender<FfmpegDiscoveryEvent>,
    },
    WorkerCompleted {
        generation: u64,
        completion: WorkerCompletion,
    },
}

fn run_actor(
    receiver: mpsc::Receiver<ActorEvent>,
    actor_sender: mpsc::Sender<ActorEvent>,
    policy: FfmpegLocatorPolicy,
    worker: Worker,
) {
    let mut coordinator = FfmpegDiscoveryCoordinator::default();
    let mut active_sender: Option<mpsc::Sender<FfmpegDiscoveryEvent>> = None;
    let mut pending_sender: Option<(u64, mpsc::Sender<FfmpegDiscoveryEvent>)> = None;
    let mut deadline: Option<(u64, Instant)> = None;

    loop {
        let event = match deadline {
            Some((generation, at)) => {
                let remaining = at.saturating_duration_since(Instant::now());
                match receiver.recv_timeout(remaining) {
                    Ok(event) => Some(event),
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        if let Some(publication) = coordinator.deadline_elapsed(generation) {
                            publish_deadline(publication, &mut active_sender, &pending_sender);
                        }
                        deadline = None;
                        None
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }
            }
            None => match receiver.recv() {
                Ok(event) => Some(event),
                Err(_) => break,
            },
        };
        let Some(event) = event else {
            continue;
        };

        match event {
            ActorEvent::Request { preference, sender } => {
                let disposition = coordinator.request(preference);
                if let Some(work) = disposition.work {
                    let _ = sender.send(FfmpegDiscoveryEvent::Started(work.generation));
                    deadline = Some((work.generation, Instant::now() + policy.attempt_timeout));
                    active_sender = Some(sender);
                    spawn_worker(work, policy, actor_sender.clone(), worker.clone());
                } else {
                    if let Some((_, replaced)) =
                        pending_sender.replace((disposition.generation, sender.clone()))
                    {
                        let _ = replaced.send(FfmpegDiscoveryEvent::Superseded);
                    }
                    if let Some(publication) = disposition.publication {
                        let _ = sender.send(FfmpegDiscoveryEvent::Published(publication));
                    }
                }
            }
            ActorEvent::WorkerCompleted {
                generation,
                completion,
            } => {
                if coordinator.active_generation() != Some(generation) {
                    continue;
                }
                let outcome = completion.outcome();
                let disposition = coordinator.worker_completed(generation, outcome);
                if let Some(publication) = disposition.publication
                    && let Some(sender) = active_sender.take()
                {
                    let event = match completion {
                        WorkerCompletion::SelectedReady(validated) => {
                            FfmpegDiscoveryEvent::SelectedReady {
                                generation: publication.generation,
                                validated,
                            }
                        }
                        WorkerCompletion::Outcome(_) => {
                            FfmpegDiscoveryEvent::Published(publication)
                        }
                    };
                    let _ = sender.send(event);
                }
                deadline = None;
                if let Some(work) = disposition.work {
                    if let Some(sender) = active_sender.take() {
                        let _ = sender.send(FfmpegDiscoveryEvent::Superseded);
                    }
                    let Some((pending_generation, sender)) = pending_sender.take() else {
                        continue;
                    };
                    if pending_generation != work.generation {
                        let _ = sender.send(FfmpegDiscoveryEvent::Superseded);
                        continue;
                    }
                    let _ = sender.send(FfmpegDiscoveryEvent::Started(work.generation));
                    deadline = Some((work.generation, Instant::now() + policy.attempt_timeout));
                    active_sender = Some(sender);
                    spawn_worker(work, policy, actor_sender.clone(), worker.clone());
                } else {
                    active_sender = None;
                }
            }
        }
    }
}

fn publish_deadline(
    publication: CoordinatorPublication,
    active_sender: &mut Option<mpsc::Sender<FfmpegDiscoveryEvent>>,
    pending_sender: &Option<(u64, mpsc::Sender<FfmpegDiscoveryEvent>)>,
) {
    if let Some((generation, sender)) = pending_sender
        && *generation == publication.generation
    {
        let _ = sender.send(FfmpegDiscoveryEvent::Published(publication));
    } else if let Some(sender) = active_sender.take() {
        let _ = sender.send(FfmpegDiscoveryEvent::Published(publication));
    }
}

fn spawn_worker(
    work: DiscoveryWork,
    policy: FfmpegLocatorPolicy,
    sender: mpsc::Sender<ActorEvent>,
    worker: Worker,
) {
    std::thread::spawn(move || {
        let generation = work.generation;
        let completion = worker(work, policy);
        let _ = sender.send(ActorEvent::WorkerCompleted {
            generation,
            completion,
        });
    });
}

#[cfg(test)]
mod tests;
