use std::{
    path::Path,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, bail};
use tokio::fs;

use super::{ModelContainer, OPERATION_METADATA};

pub(super) async fn acquire_model_lock(root: &Path, name: &str) -> anyhow::Result<std::fs::File> {
    acquire_model_lock_with_timeout(root, name, Duration::from_secs(30)).await
}

pub(super) async fn acquire_model_lock_with_timeout(
    root: &Path,
    name: &str,
    timeout: Duration,
) -> anyhow::Result<std::fs::File> {
    let name = name.to_owned();
    let path = root.join(format!(".{name}.lock"));
    tokio::task::spawn_blocking(move || {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(&path)
            .with_context(|| format!("failed to open model lock {}", path.display()))?;
        let deadline = Instant::now() + timeout;
        loop {
            match file.try_lock() {
                Ok(()) => return Ok(file),
                Err(std::fs::TryLockError::WouldBlock) => {
                    if Instant::now() >= deadline {
                        bail!(
                            "timed out waiting for another application instance to finish model operation '{}': {}",
                            name,
                            path.display()
                        );
                    }
                    std::thread::sleep(Duration::from_millis(25));
                }
                Err(std::fs::TryLockError::Error(error)) => {
                    return Err(error).with_context(|| {
                        format!("failed to acquire model lock {}", path.display())
                    });
                }
            }
        }
    })
    .await
    .context("model lock task terminated")?
}

fn operation_timestamp() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
}

pub(super) async fn next_operation_sequence(root: &Path, name: &str) -> anyhow::Result<u64> {
    let sequence_path = root.join(format!(".{name}.sequence"));
    let current = match fs::read_to_string(&sequence_path).await {
        Ok(value) => value
            .parse::<u64>()
            .with_context(|| format!("invalid model operation sequence: {value}"))?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => 0,
        Err(error) => return Err(error).context("failed to read model operation sequence"),
    };
    let next = current
        .checked_add(1)
        .context("model operation sequence exhausted")?;
    let temporary = root.join(format!(
        ".{name}.sequence-{}-{}.tmp",
        std::process::id(),
        operation_timestamp()
    ));
    fs::write(&temporary, next.to_string())
        .await
        .context("failed to stage model operation sequence")?;
    if let Err(error) = fs::rename(&temporary, &sequence_path).await {
        let _ = fs::remove_file(&temporary).await;
        return Err(error).context("failed to publish model operation sequence");
    }
    Ok(next)
}

fn artifact_order(path: &Path) -> (u64, String) {
    let durable = std::fs::read_to_string(path.join(OPERATION_METADATA))
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or_default();
    (durable, path.to_string_lossy().into_owned())
}

pub(super) async fn reconcile_generations(
    model: &ModelContainer,
    root: &Path,
) -> anyhow::Result<()> {
    reconcile_generations_with(&RealPublishFilesystem, model, root).await
}

pub(super) async fn reconcile_generations_with(
    filesystem: &impl PublishFilesystem,
    model: &ModelContainer,
    root: &Path,
) -> anyhow::Result<()> {
    let stage_prefix = format!(".{}.stage-", model.name);
    let backup_prefix = format!(".{}.backup-", model.name);
    let mut stages = Vec::new();
    let mut backups = Vec::new();
    let mut entries = fs::read_dir(root)
        .await
        .with_context(|| format!("failed to inspect model directory {}", root.display()))?;
    while let Some(entry) = entries.next_entry().await? {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with(&stage_prefix) {
            stages.push(entry.path());
        } else if name.starts_with(&backup_prefix) {
            backups.push(entry.path());
        }
    }
    stages.sort_by_key(|path| artifact_order(path));
    backups.sort_by_key(|path| artifact_order(path));

    let final_directory = root.join(&model.name);
    if !final_directory.exists() {
        if let Some(stage) = stages.iter().rev().find(|path| model.ready_in(path)) {
            filesystem
                .rename(stage, &final_directory)
                .with_context(|| {
                    format!(
                        "failed to recover authenticated staged model generation {}",
                        stage.display()
                    )
                })?;
        } else if let Some(backup) = backups.iter().rev().find(|path| model.ready_in(path)) {
            filesystem
                .rename(backup, &final_directory)
                .with_context(|| {
                    format!(
                        "failed to restore interrupted model backup {}",
                        backup.display()
                    )
                })?;
        }
    } else if !model.ready_in(&final_directory)
        && let Some(backup) = backups.iter().rev().find(|path| model.ready_in(path))
    {
        let incomplete = root.join(format!(
            ".{}.incomplete-{}",
            model.name,
            operation_timestamp()
        ));
        filesystem
            .rename(&final_directory, &incomplete)
            .context("failed to quarantine incomplete model generation")?;
        if let Err(error) = filesystem.rename(backup, &final_directory) {
            let restore = filesystem.rename(&incomplete, &final_directory).err();
            return match restore {
                Some(restore) => Err(anyhow::anyhow!(
                    "failed to restore authenticated backup: {error}; failed to restore incomplete generation: {restore}"
                )),
                None => Err(error).context("failed to restore authenticated model backup"),
            };
        }
    }

    if model.ready_in(&final_directory) {
        for path in stages.into_iter().chain(backups) {
            if path.exists() {
                match filesystem.remove_dir_all(&path) {
                    Ok(()) => {}
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                    Err(error) => {
                        eprintln!(
                            "authenticated model generation is ready, but stale artifact cleanup failed at {}: {error}",
                            path.display()
                        );
                    }
                }
            }
        }
    }
    Ok(())
}

pub(super) trait PublishFilesystem {
    fn rename(&self, from: &Path, to: &Path) -> std::io::Result<()>;
    fn remove_dir_all(&self, path: &Path) -> std::io::Result<()>;
}

struct RealPublishFilesystem;

impl PublishFilesystem for RealPublishFilesystem {
    fn rename(&self, from: &Path, to: &Path) -> std::io::Result<()> {
        std::fs::rename(from, to)
    }

    fn remove_dir_all(&self, path: &Path) -> std::io::Result<()> {
        std::fs::remove_dir_all(path)
    }
}

pub(super) fn publish_generation(
    staging: &Path,
    final_directory: &Path,
    backup: &Path,
) -> anyhow::Result<()> {
    publish_generation_with(&RealPublishFilesystem, staging, final_directory, backup)
}

pub(super) fn publish_generation_with(
    filesystem: &impl PublishFilesystem,
    staging: &Path,
    final_directory: &Path,
    backup: &Path,
) -> anyhow::Result<()> {
    let had_previous = final_directory.exists();
    if had_previous {
        filesystem
            .rename(final_directory, backup)
            .with_context(|| {
                format!(
                    "failed to preserve prior model generation {}",
                    final_directory.display()
                )
            })?;
    }

    if let Err(error) = filesystem.rename(staging, final_directory) {
        let restore = if had_previous {
            filesystem.rename(backup, final_directory).err()
        } else {
            None
        };
        return match restore {
            Some(restore) => Err(anyhow::anyhow!(
                "failed to publish model generation: {error}; failed to restore prior generation: {restore}"
            )),
            None => Err(error).context("failed to publish model generation"),
        };
    }

    if had_previous && let Err(error) = filesystem.remove_dir_all(backup) {
        eprintln!(
            "model generation published but backup cleanup failed at {}: {error}",
            backup.display()
        );
    }
    Ok(())
}

pub(super) async fn cleanup_directory(path: &Path) -> anyhow::Result<()> {
    match fs::remove_dir_all(path).await {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error)
            .with_context(|| format!("failed to clean model directory {}", path.display())),
    }
}
