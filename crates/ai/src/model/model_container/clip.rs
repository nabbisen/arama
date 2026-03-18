const MODEL_NAME: &str = "clip-vit-base-patch32";
const SOURCE_URL: &str = "https://huggingface.co/openai/clip-vit-base-patch32/resolve/main/pytorch_model.bin?download=true";

use super::{ModelContainer, SourceUrl};

pub fn model() -> ModelContainer {
    ModelContainer {
        name: MODEL_NAME.to_owned(),
        source_url: SourceUrl::PyTorch(SOURCE_URL.to_owned()),
    }
}
