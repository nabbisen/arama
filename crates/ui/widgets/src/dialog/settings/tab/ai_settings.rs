pub mod message;
mod update;
mod view;

#[derive(Clone, Debug, Default)]
pub struct AiSettings {
    message: String,
}
