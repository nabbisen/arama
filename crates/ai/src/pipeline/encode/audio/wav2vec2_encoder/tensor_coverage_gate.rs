//! Task 042 §3: a tensor-set coverage gate.
//!
//! Loads the *real* model's tensors through a recording backend and
//! asserts that every tensor the file ships under the `wav2vec2.`
//! prefix was actually requested by something we load. The requested
//! set is not a hand-maintained list - it is a live trace of what
//! `FeatureExtractor::load` and `FeatureProjection::load` touch, which
//! is itself already driven by `config.json` (conv layer count,
//! `feat_extract_norm`, ...). A tensor present in the file but never
//! requested is exactly what audit A1 (no transformer) and, before the
//! fix, A2 (conv layer 0's `GroupNorm` unread) both look like.
//!
//! Needs the real, downloaded `wav2vec2-base-960h` model - see the
//! `#[ignore]` reason on the test below.

use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use candle_core::{DType, Device, Error, Shape, Tensor};
use candle_nn::{VarBuilder, var_builder::SimpleBackend};

use super::super::wav2vec2_config::Wav2vec2Config;
use super::{FeatureExtractor, FeatureProjection};

/// Wraps an in-memory tensor map and records every name requested
/// through it, so "the set of tensors an encoder loads" can be read
/// back after loading rather than guessed at.
struct RecordingBackend {
    inner: HashMap<String, Tensor>,
    requested: Arc<Mutex<HashSet<String>>>,
}

impl SimpleBackend for RecordingBackend {
    fn get(
        &self,
        s: Shape,
        name: &str,
        _h: candle_nn::Init,
        dtype: DType,
        dev: &Device,
    ) -> candle_core::Result<Tensor> {
        self.requested.lock().unwrap().insert(name.to_owned());
        let tensor = self.inner.get(name).ok_or_else(|| {
            Error::CannotFindTensor {
                path: name.to_string(),
            }
            .bt()
        })?;
        if tensor.shape() != &s {
            return Err(Error::UnexpectedShape {
                msg: format!("shape mismatch for {name}"),
                expected: s,
                got: tensor.shape().clone(),
            }
            .bt());
        }
        tensor.to_device(dev)?.to_dtype(dtype)
    }

    fn get_unchecked(&self, name: &str, dtype: DType, dev: &Device) -> candle_core::Result<Tensor> {
        self.requested.lock().unwrap().insert(name.to_owned());
        let tensor = self.inner.get(name).ok_or_else(|| {
            Error::CannotFindTensor {
                path: name.to_string(),
            }
            .bt()
        })?;
        tensor.to_device(dev)?.to_dtype(dtype)
    }

    fn contains_tensor(&self, name: &str) -> bool {
        self.inner.contains_key(name)
    }
}

#[test]
// Task 042 §3: this gate needs the real, downloaded wav2vec2-base-960h
// model (config.json + model.safetensors, ~361 MB) to have anything to
// compare against - not present in this sandbox or in CI, and the task
// does not authorize adding a model download to the test suite. Left
// red-if-run rather than weakened to run without the file: RFC 046 §3
// replaces this whole loading path (per-segment storage, eventually a
// real transformer), at which point this gate is rewritten anyway, so
// it is not worth building network/CI plumbing for in the meantime.
#[ignore = "needs the real downloaded wav2vec2-base-960h model; see RFC 046 §3"]
fn every_tensor_the_real_model_ships_is_read_by_something_we_load() {
    let model = crate::model::model_container::wav2vec2::model();
    let device = Device::Cpu;

    let config_str = std::fs::read_to_string(model.config_json_path().unwrap()).unwrap();
    let config: Wav2vec2Config = serde_json::from_str(&config_str).unwrap();

    let safetensors_path = model.safetensors_path().unwrap();
    let all_tensors = candle_core::safetensors::load(&safetensors_path, &device).unwrap();
    let w2v_tensors: HashMap<String, Tensor> = all_tensors
        .into_iter()
        .filter_map(|(k, v)| k.strip_prefix("wav2vec2.").map(|k| (k.to_owned(), v)))
        .collect();
    let available: HashSet<String> = w2v_tensors.keys().cloned().collect();

    let requested = Arc::new(Mutex::new(HashSet::new()));
    let backend = RecordingBackend {
        inner: w2v_tensors,
        requested: requested.clone(),
    };
    let vb = VarBuilder::from_backend(Box::new(backend), DType::F32, device);

    // Loads whatever this encoder currently implements. This is
    // deliberately not the full `Wav2vec2Encoder::load` (which reads
    // straight from disk) - the recording backend needs to sit between
    // the real tensors and the loaders it is measuring.
    FeatureExtractor::load(vb.pp("feature_extractor"), &config).unwrap();
    FeatureProjection::load(vb.pp("feature_projection"), 512, config.hidden_size).unwrap();

    let requested = requested.lock().unwrap();
    let unread: Vec<&String> = available.difference(&requested).collect();

    assert!(
        unread.is_empty(),
        "the model ships tensors this encoder never reads: {unread:?} - either load them or \
         account for why they are legitimately unused"
    );
}
