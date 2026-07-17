use std::{
    collections::HashMap,
    future::Future,
    io::Result,
    path::Path,
    sync::{Arc, Mutex, OnceLock},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, bail};
use arama_env::validate_dir;
use tokio::{fs, sync::watch};

use super::{
    CONFIG_JSON, GENERATION_MANIFEST, ModelContainer, ModelDownloadStatus, OPERATION_METADATA,
    PYTORCH_MODEL, SAFETENSORS_MODEL, SourceUrl, models_dir,
    publication::{
        acquire_model_lock, cleanup_directory, next_operation_sequence, publish_generation,
        reconcile_generations,
    },
    specification::validate_model_specification,
    transfer::download_authenticated_file,
};

#[derive(Clone, Debug)]
enum SharedDownloadState {
    Idle,
    Running,
    Finished(std::result::Result<(), String>),
}

struct DownloadGate {
    running: Option<Arc<DownloadGeneration>>,
    next_generation: u64,
}

pub(super) struct DownloadGeneration {
    pub(super) id: u64,
    pub(super) result: watch::Sender<Option<std::result::Result<(), String>>>,
}

pub(super) struct DownloadEntry {
    identity: String,
    gate: Mutex<DownloadGate>,
    state: watch::Sender<SharedDownloadState>,
}

static MODEL_DOWNLOADS: OnceLock<Mutex<HashMap<String, Arc<DownloadEntry>>>> = OnceLock::new();
static PROCESS_NONCE: OnceLock<u128> = OnceLock::new();

impl ModelContainer {
    fn ensure_safetensors_in(&self, directory: &Path) -> Result<()> {
        let is_model_safetensors = match &self.source_url {
            SourceUrl::ModelSafetensors(_) | SourceUrl::ModelSafetensorsConfigJson(_) => true,
            SourceUrl::PyTorch(_) => false,
        };

        if is_model_safetensors {
            return Ok(());
        }

        let pytorch_path = directory.join(PYTORCH_MODEL);

        pt2safetensors::Pt2Safetensors::default()
            .removes_pt_at_conversion_success()
            .convert(pytorch_path, directory.join(SAFETENSORS_MODEL))
            .map_err(|err| {
                std::io::Error::other(format!("failed to convert pytorch to safetensors: {err}"))
            })?;

        Ok(())
    }

    pub fn download_status(&self) -> ModelDownloadStatus {
        if self.clone().ready().unwrap_or(false) {
            return ModelDownloadStatus::Ready;
        }
        let identity = self.identity();
        let Some(entry) = existing_download_entry(&self.name) else {
            return ModelDownloadStatus::Idle;
        };
        if entry.identity != identity {
            return ModelDownloadStatus::Failed;
        }
        match &*entry.state.borrow() {
            SharedDownloadState::Running => ModelDownloadStatus::Downloading,
            SharedDownloadState::Finished(Err(_)) => ModelDownloadStatus::Failed,
            SharedDownloadState::Idle | SharedDownloadState::Finished(Ok(())) => {
                ModelDownloadStatus::Idle
            }
        }
    }

    /// Download, authenticate, and publish one complete model generation.
    ///
    /// Concurrent callers for the same model join the active generation.
    /// Only the owner writes, using an operation-owned staging directory; the
    /// final model/config directory is replaced only after the full generation
    /// validates.
    pub async fn download(&self) -> anyhow::Result<()> {
        let entry = download_entry(&self.name, &self.identity())?;
        let (generation, start_worker) = select_generation(&entry);

        if start_worker {
            spawn_generation_worker(self.clone(), entry.clone(), generation.clone());
        }

        wait_for_generation(generation.result.subscribe()).await
    }

    async fn cleanup_generation(&self, generation: u64) {
        if let Ok(root) = models_dir() {
            let staging = root.join(self.operation_name("stage", generation));
            let _ = cleanup_directory(&staging).await;
        }
    }

    async fn download_generation(&self, generation: u64) -> anyhow::Result<()> {
        validate_model_specification(self)?;
        let root = models_dir()?;
        validate_dir(&root)?;
        let _model_lock = acquire_model_lock(&root, &self.name).await?;
        reconcile_generations(self, &root).await?;
        if self.clone().ready()? {
            return Ok(());
        }
        let operation_sequence = next_operation_sequence(&root, &self.name).await?;
        let staging = root.join(self.operation_name("stage", generation));
        let backup = root.join(self.operation_name("backup", generation));
        let final_directory = root.join(&self.name);
        cleanup_directory(&staging).await?;
        cleanup_directory(&backup).await?;
        fs::create_dir(&staging)
            .await
            .with_context(|| format!("failed to create staging directory {}", staging.display()))?;
        fs::write(
            staging.join(OPERATION_METADATA),
            operation_sequence.to_string(),
        )
        .await
        .context("failed to persist model operation ordering metadata")?;

        let (model_url, primary_path) = match &self.source_url {
            SourceUrl::ModelSafetensors(url) | SourceUrl::ModelSafetensorsConfigJson((url, _)) => {
                (url.as_str(), staging.join(SAFETENSORS_MODEL))
            }
            SourceUrl::PyTorch(url) => (url.as_str(), staging.join(PYTORCH_MODEL)),
        };

        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30 * 60))
            .build()
            .context("failed to build model download client")?;
        if let Err(error) = download_authenticated_file(
            &client,
            model_url,
            &primary_path,
            self.expected_sha256,
            self.max_model_bytes,
        )
        .await
        {
            let _ = cleanup_directory(&staging).await;
            return Err(error);
        }

        if let Err(error) = self.ensure_safetensors_in(&staging) {
            let _ = cleanup_directory(&staging).await;
            return Err(error).context("failed to convert downloaded model to SafeTensors");
        }

        if let SourceUrl::ModelSafetensorsConfigJson((_, config_url)) = &self.source_url {
            let config_path = staging.join(CONFIG_JSON);
            let expected = self
                .config_expected_sha256
                .context("model configuration has no pinned SHA-256 digest")?;
            if let Err(error) = download_authenticated_file(
                &client,
                config_url,
                &config_path,
                expected,
                self.max_config_bytes
                    .context("model configuration has no trusted size bound")?,
            )
            .await
            {
                let _ = cleanup_directory(&staging).await;
                return Err(error).context("failed to download model configuration");
            }
        }

        if let Err(error) = fs::write(staging.join(GENERATION_MANIFEST), self.identity()).await {
            let _ = cleanup_directory(&staging).await;
            return Err(error).context("failed to write authenticated model generation marker");
        }

        if !self.ready_in(&staging) {
            let _ = cleanup_directory(&staging).await;
            bail!("staged model did not satisfy readiness checks");
        }
        publish_generation(&staging, &final_directory, &backup)?;
        reconcile_generations(self, &root).await
    }

    pub(super) fn operation_name(&self, kind: &str, generation: u64) -> String {
        let nonce = PROCESS_NONCE.get_or_init(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        });
        format!(
            ".{}.{}-{}-{}-{generation}",
            self.name,
            kind,
            std::process::id(),
            nonce
        )
    }
}

fn spawn_generation_worker(
    model: ModelContainer,
    entry: Arc<DownloadEntry>,
    generation: Arc<DownloadGeneration>,
) {
    let generation_id = generation.id;
    let worker_model = model.clone();
    supervise_generation(model, entry, generation, async move {
        worker_model.download_generation(generation_id).await
    });
}

pub(super) fn supervise_generation<F>(
    model: ModelContainer,
    entry: Arc<DownloadEntry>,
    generation: Arc<DownloadGeneration>,
    future: F,
) where
    F: Future<Output = anyhow::Result<()>> + Send + 'static,
{
    let generation_id = generation.id;
    let worker = tokio::spawn(future);
    tokio::spawn(async move {
        let result = match worker.await {
            Ok(result) => result,
            Err(error) => {
                model.cleanup_generation(generation_id).await;
                Err(anyhow::anyhow!(
                    "model generation worker terminated before completion: {error}"
                ))
            }
        };
        finish_generation(&entry, &generation, &result);
    });
}

pub(super) fn select_generation(entry: &DownloadEntry) -> (Arc<DownloadGeneration>, bool) {
    let mut gate = entry.gate.lock().expect("model download gate poisoned");
    if let Some(generation) = &gate.running {
        return (generation.clone(), false);
    }
    gate.next_generation = gate.next_generation.saturating_add(1);
    let id = gate.next_generation;
    let (result, _) = watch::channel(None);
    let generation = Arc::new(DownloadGeneration { id, result });
    gate.running = Some(generation.clone());
    entry.state.send_replace(SharedDownloadState::Running);
    (generation, true)
}

pub(super) fn finish_generation(
    entry: &DownloadEntry,
    generation: &DownloadGeneration,
    result: &anyhow::Result<()>,
) {
    let shared_result = result.as_ref().map(|_| ()).map_err(ToString::to_string);
    let mut gate = entry.gate.lock().expect("model download gate poisoned");
    if gate.running.as_ref().map(|running| running.id) == Some(generation.id) {
        gate.running = None;
    }
    generation.result.send_replace(Some(shared_result.clone()));
    entry
        .state
        .send_replace(SharedDownloadState::Finished(shared_result));
}

pub(super) async fn wait_for_generation(
    mut receiver: watch::Receiver<Option<std::result::Result<(), String>>>,
) -> anyhow::Result<()> {
    loop {
        if let Some(result) = receiver.borrow().clone() {
            return result.map_err(anyhow::Error::msg);
        }
        receiver
            .changed()
            .await
            .context("model download worker disappeared")?;
    }
}

pub(super) fn download_entry(name: &str, identity: &str) -> anyhow::Result<Arc<DownloadEntry>> {
    let downloads = MODEL_DOWNLOADS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut downloads = downloads.lock().expect("model download registry poisoned");
    if let Some(entry) = downloads.get(name) {
        if entry.identity != identity {
            bail!(
                "model name '{name}' is already registered with a different immutable specification"
            );
        }
        return Ok(entry.clone());
    }
    let entry = {
        let (state, _) = watch::channel(SharedDownloadState::Idle);
        Arc::new(DownloadEntry {
            identity: identity.to_owned(),
            gate: Mutex::new(DownloadGate {
                running: None,
                next_generation: 0,
            }),
            state,
        })
    };
    downloads.insert(name.to_owned(), entry.clone());
    Ok(entry)
}

fn existing_download_entry(name: &str) -> Option<Arc<DownloadEntry>> {
    MODEL_DOWNLOADS
        .get()?
        .lock()
        .expect("model download registry poisoned")
        .get(name)
        .cloned()
}
