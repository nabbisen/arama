use super::{ModelContainer, SourceUrl};

pub const HIDDEN_DIM: usize = 768;

const MODEL_NAME: &str = "wav2vec2-large-robust-12-ft-emotion-msp-dim";
const SOURCE_URL: &str = "https://huggingface.co/audeering/wav2vec2-large-robust-12-ft-emotion-msp-dim/resolve/main/model.safetensors?download=true";
// const CONFIG_JSON_URL: &str =
//     "https://huggingface.co/distil-whisper/distil-small.en/resolve/main/config.json?download=true";

pub fn model() -> ModelContainer {
    ModelContainer {
        name: MODEL_NAME.to_owned(),
        source_url: SourceUrl::ModelSafetensors(SOURCE_URL.to_owned()),
    }
}
