use super::{ModelContainer, SourceUrl};

pub const HIDDEN_DIM: usize = 768;

const MODEL_NAME: &str = "wav2vec2-base-960h";
const MODEL_REVISION: &str = "22aad52d435eb6dbaf354bdad9b0da84ce7d6156";
const SOURCE_URL: &str = "https://huggingface.co/facebook/wav2vec2-base-960h/resolve/22aad52d435eb6dbaf354bdad9b0da84ce7d6156/model.safetensors?download=true";
const CONFIG_JSON_URL: &str = "https://huggingface.co/facebook/wav2vec2-base-960h/resolve/22aad52d435eb6dbaf354bdad9b0da84ce7d6156/config.json?download=true";
const SOURCE_SHA256: &str = "8aa76ab2243c81747a1f832954586bc566090c83a0ac167df6f31f0fa917d74a";
const CONFIG_JSON_SHA256: &str = "d3ec255c063d9f95057b553b19c20135b259875834a4fe9deb218a6be25b4cf3";

pub fn model() -> ModelContainer {
    ModelContainer::new(
        MODEL_NAME,
        SourceUrl::ModelSafetensorsConfigJson((SOURCE_URL.to_owned(), CONFIG_JSON_URL.to_owned())),
        SOURCE_SHA256,
        Some(CONFIG_JSON_SHA256),
        1024 * 1024 * 1024,
        Some(4 * 1024 * 1024),
    )
    .expect("built-in Wav2Vec2 specification is valid")
}

pub fn revision() -> &'static str {
    MODEL_REVISION
}
