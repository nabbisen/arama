use super::{ModelContainer, SourceUrl};

pub const HIDDEN_DIM: usize = 768;

const MODEL_NAME: &str = "distil-small.en";
const SOURCE_URL: &str = "https://huggingface.co/distil-whisper/distil-small.en/blob/main/model.safetensors?download=true";
const CONFIG_JSON_URL: &str =
    "https://huggingface.co/distil-whisper/distil-small.en/resolve/main/config.json?download=true";

pub fn model() -> ModelContainer {
    ModelContainer {
        name: MODEL_NAME.to_owned(),
        source_url: SourceUrl::ModelSafetensorsConfigJson((
            SOURCE_URL.to_owned(),
            CONFIG_JSON_URL.to_owned(),
        )),
    }
}
