use std::{
    io::Result,
    path::{Path, PathBuf},
};

use super::{CONFIG_JSON, MODEL_DIR, PYTORCH_MODEL, SAFETENSORS_MODEL};
use arama_env::local_dir;
use sha2::{Digest, Sha256};

use specification::validate_model_specification;
use transfer::sha256_hex;

pub mod clip;
mod lifecycle;
mod publication;
mod specification;
mod transfer;
pub mod wav2vec2;

// Task 036: `lifecycle` is private (its other items are implementation
// detail, reached only through `ModelContainer`'s own methods), but
// `DownloadProgress` is part of `download_with_progress`'s public
// return type and must be nameable from outside this crate.
pub use lifecycle::DownloadProgress;

const GENERATION_MANIFEST: &str = ".arama-model-generation";
const OPERATION_METADATA: &str = ".arama-model-operation";

#[derive(Clone, Debug)]
pub enum SourceUrl {
    ModelSafetensors(String),
    ModelSafetensorsConfigJson((String, String)),
    PyTorch(String),
}

impl SourceUrl {
    pub fn download_url(&self) -> String {
        let ret = match self {
            SourceUrl::ModelSafetensors(s) => s,
            SourceUrl::ModelSafetensorsConfigJson((s, _)) => s,
            SourceUrl::PyTorch(s) => s,
        };
        ret.to_owned()
    }
}

#[derive(Clone, Debug)]
pub struct ModelContainer {
    name: String,
    source_url: SourceUrl,
    expected_sha256: &'static str,
    config_expected_sha256: Option<&'static str>,
    max_model_bytes: u64,
    max_config_bytes: Option<u64>,
    /// Task 046 (audit B6): `None` in every production `ModelContainer` -
    /// [`new`](Self::new) never sets it, and nothing else can from outside
    /// this module. Tests construct it directly (this module's `tests`
    /// submodule can reach private fields) so their models resolve under
    /// a scratch directory instead of the real platform data directory,
    /// without ever touching `ARAMA_DATA_HOME` or any other process-global
    /// state - the same "pure seam beside the real resolver" shape as
    /// `arama_env::local_dir`'s own (private, `arama_env`-internal)
    /// `local_dir_with_override`.
    root_override: Option<PathBuf>,
    /// T071: `false` in every production `ModelContainer` - `new` never
    /// sets it, and nothing else can from outside this module. Tests that
    /// drive `download()`/`download_with_progress()` end-to-end against a
    /// plain-HTTP loopback fixture set this `true` directly (same
    /// private-field access as `root_override`), since there is no way to
    /// stand up a TLS certificate for `127.0.0.1` in a unit test. It
    /// changes only which builder [`download_client_builder`] starts
    /// from - the connect/overall timeouts `download_generation` adds on
    /// top, and the download path itself, stay exactly what production
    /// runs.
    plain_http_override: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelDownloadStatus {
    Idle,
    Downloading,
    Ready,
    Failed,
}

impl ModelContainer {
    pub fn new(
        name: impl Into<String>,
        source_url: SourceUrl,
        expected_sha256: &'static str,
        config_expected_sha256: Option<&'static str>,
        max_model_bytes: u64,
        max_config_bytes: Option<u64>,
    ) -> anyhow::Result<Self> {
        let model = Self {
            name: name.into(),
            source_url,
            expected_sha256,
            config_expected_sha256,
            max_model_bytes,
            max_config_bytes,
            root_override: None,
            plain_http_override: false,
        };
        validate_model_specification(&model)?;
        Ok(model)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn source_url(&self) -> &SourceUrl {
        &self.source_url
    }

    pub fn expected_sha256(&self) -> &'static str {
        self.expected_sha256
    }

    pub fn config_expected_sha256(&self) -> Option<&'static str> {
        self.config_expected_sha256
    }

    pub fn safetensors_path(&self) -> Result<PathBuf> {
        Ok(self.model_dir()?.join(SAFETENSORS_MODEL))
    }

    pub fn config_json_path(&self) -> Result<PathBuf> {
        Ok(self.model_dir()?.join(CONFIG_JSON))
    }

    pub fn pytorch_path(&self) -> Result<PathBuf> {
        Ok(self.model_dir()?.join(PYTORCH_MODEL))
    }

    pub fn ready(self) -> Result<bool> {
        Ok(self.ready_in(&self.model_dir()?))
    }

    fn model_dir(&self) -> Result<PathBuf> {
        Ok(self.models_dir()?.join(&self.name))
    }

    /// The real `models_dir()`, unless this instance carries a test-only
    /// [`root_override`](Self::root_override) - see its own comment.
    fn models_dir(&self) -> Result<PathBuf> {
        match &self.root_override {
            Some(root) => Ok(root.clone()),
            None => models_dir(),
        }
    }

    /// The real [`model_download_client_builder`], unless this instance
    /// carries a test-only [`plain_http_override`](Self::plain_http_override),
    /// see its own comment. Mirrors [`models_dir`](Self::models_dir)'s shape
    /// exactly.
    fn download_client_builder(&self) -> reqwest::ClientBuilder {
        if self.plain_http_override {
            reqwest::Client::builder()
        } else {
            model_download_client_builder()
        }
    }

    fn ready_in(&self, directory: &Path) -> bool {
        directory.join(SAFETENSORS_MODEL).is_file()
            && (self.config_expected_sha256.is_none() || directory.join(CONFIG_JSON).is_file())
            && std::fs::read_to_string(directory.join(GENERATION_MANIFEST))
                .is_ok_and(|manifest| manifest == self.identity())
    }

    fn identity(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.name.as_bytes());
        match &self.source_url {
            SourceUrl::ModelSafetensors(url) => {
                hasher.update(b"safetensors\0");
                hasher.update(url.as_bytes());
            }
            SourceUrl::ModelSafetensorsConfigJson((model, config)) => {
                hasher.update(b"safetensors-config\0");
                hasher.update(model.as_bytes());
                hasher.update(b"\0");
                hasher.update(config.as_bytes());
            }
            SourceUrl::PyTorch(url) => {
                hasher.update(b"pytorch\0");
                hasher.update(url.as_bytes());
            }
        }
        hasher.update(self.expected_sha256.as_bytes());
        hasher.update(self.config_expected_sha256.unwrap_or_default().as_bytes());
        hasher.update(self.max_model_bytes.to_le_bytes());
        hasher.update(self.max_config_bytes.unwrap_or_default().to_le_bytes());
        sha256_hex(&hasher.finalize())
    }
}

fn models_dir() -> Result<PathBuf> {
    Ok(local_dir()?.join(MODEL_DIR))
}

/// T071: the one place a `reqwest` client for arama's only network traffic
/// (the one-time model download, and the setup screen's size-check `HEAD`)
/// is built - `https_only(true)` so a redirect to a plain-HTTP URL is
/// refused before it connects, not just distrusted afterward. Confirmed
/// directly against the locked `reqwest` version's own source (0.13.4, per
/// `Cargo.lock`), not assumed from the handoff that first named it:
/// `src/redirect.rs:324`, `if self.https_only && next_url.scheme() !=
/// "https"`, returns `url_bad_scheme`; the builder method itself,
/// `ClientBuilder::https_only`, is `src/async_impl/client.rs:2245-2246`,
/// and it is wired into the redirect policy at `src/async_impl/client.rs:1018`
/// (`.with_https_only(config.https_only)`). This is not a behaviour arama
/// implements itself, so a future `reqwest` upgrade that changed it would
/// change this function's own guarantee, not just its callers'.
///
/// Returns a builder, not a built `Client`, so each caller adds its own
/// timeouts without this function needing to know both callers' different
/// needs (a long download vs. a short metadata `HEAD`).
///
/// This does not replace the pinned SHA-256 checksum
/// (`ModelContainer::expected_sha256`/`config_expected_sha256`), which
/// remains the integrity guarantee regardless of scheme - it adds
/// confidentiality of which model is being fetched, and refuses an
/// on-path attacker breaking or falsifying what a plain-HTTP hop could
/// otherwise tamper with.
pub fn model_download_client_builder() -> reqwest::ClientBuilder {
    reqwest::Client::builder().https_only(true)
}

#[cfg(test)]
use lifecycle::{
    download_entry, finish_generation, select_generation, supervise_generation, wait_for_generation,
};
#[cfg(test)]
use publication::{
    PublishFilesystem, acquire_model_lock, acquire_model_lock_with_timeout,
    next_operation_sequence, publish_generation_with, reconcile_generations,
    reconcile_generations_with,
};
#[cfg(test)]
#[path = "model_container/tests.rs"]
mod tests;
