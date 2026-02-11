pub mod message;
pub mod update;
pub mod view;

#[derive(Default)]
pub struct SwdirDepthLimit {
    pub value: Option<usize>,
}
