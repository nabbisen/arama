use super::{similarity_slider, swdir_depth_limit};

#[derive(Debug, Clone)]
pub enum Message {
    SwdirDepthLimitMessage(swdir_depth_limit::message::Message),
    SimilaritySliderMessage(similarity_slider::message::Message),
}
