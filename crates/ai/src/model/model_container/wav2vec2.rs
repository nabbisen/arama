use super::{ModelContainer, SourceUrl};

pub const HIDDEN_DIM: usize = 768;

const MODEL_NAME: &str = "wav2vec2-base-960h";
const SOURCE_URL: &str = "https://huggingface.co/facebook/wav2vec2-base-960h/resolve/main/model.safetensors?download=true";
const CONFIG_JSON_URL: &str =
    "https://huggingface.co/facebook/wav2vec2-base-960h/resolve/main/config.json?download=true";

pub fn model() -> ModelContainer {
    ModelContainer {
        name: MODEL_NAME.to_owned(),
        source_url: SourceUrl::ModelSafetensorsConfigJson((
            SOURCE_URL.to_owned(),
            CONFIG_JSON_URL.to_owned(),
        )),
    }
}
