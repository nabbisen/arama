use iced::{
    Element,
    widget::{column, row},
};

use super::{GallerySettings, message::Message};

impl GallerySettings {
    pub fn view(&self, has_image_similarity: bool) -> Element<'_, Message> {
        let mut ret = column![
            self.swdir_depth_limit
                .view()
                .map(Message::SwdirDepthLimitMessage)
        ];
        if has_image_similarity {
            ret = ret.push(row![
                self.similarity_slider.similarity_quality.label(),
                self.similarity_slider
                    .view()
                    .map(Message::SimilaritySliderMessage)
            ])
        }
        ret.into()
    }
}
