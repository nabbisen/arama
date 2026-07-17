const MODEL_NAME: &str = "clip-vit-base-patch32";
const MODEL_REVISION: &str = "3d74acf9a28c67741b2f4f2ea7635f0aaf6f0268";
const SOURCE_URL: &str = "https://huggingface.co/openai/clip-vit-base-patch32/resolve/3d74acf9a28c67741b2f4f2ea7635f0aaf6f0268/pytorch_model.bin?download=true";
const SOURCE_SHA256: &str = "a63082132ba4f97a80bea76823f544493bffa8082296d62d71581a4feff1576f";

use super::{ModelContainer, SourceUrl};

pub fn model() -> ModelContainer {
    ModelContainer::new(
        MODEL_NAME,
        SourceUrl::PyTorch(SOURCE_URL.to_owned()),
        SOURCE_SHA256,
        None,
        1024 * 1024 * 1024,
        None,
    )
    .expect("built-in CLIP specification is valid")
}

pub fn revision() -> &'static str {
    MODEL_REVISION
}
