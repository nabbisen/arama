const MODEL_NAME: &str = "clip-vit-base-patch32";
const SOURCE_URL: &str = "https://huggingface.co/openai/clip-vit-base-patch32/resolve/main/pytorch_model.bin?download=true";

use super::Model;

pub fn model() -> Model {
    Model {
        name: MODEL_NAME.to_owned(),
        source_url: SOURCE_URL.to_owned(),
    }
}
