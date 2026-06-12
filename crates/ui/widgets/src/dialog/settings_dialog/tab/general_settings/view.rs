use arama_env::target_media_type::TargetMediaType;
use iced::{
    Element,
    widget::{button, checkbox, column, container, row, slider, text},
};
use lucide_icons::iced::{icon_arrow_down, icon_arrow_up};

use super::{GeneralSettings, message::Message};

impl GeneralSettings {
    pub fn view(&self) -> Element<'_, Message> {
        let target_media_types = row![
            checkbox(self.target_media_type.include_image)
                .label("Image")
                .on_toggle(|x| {
                    Message::TargetMediaTypeChanged(TargetMediaType {
                        include_image: x,
                        include_video: self.target_media_type.include_video,
                    })
                }),
            checkbox(self.target_media_type.include_video)
                .label("Video")
                .on_toggle(|x| {
                    Message::TargetMediaTypeChanged(TargetMediaType {
                        include_image: self.target_media_type.include_image,
                        include_video: x,
                    })
                })
        ]
        .spacing(10);

        let sub_dir_depth_limit = row![
            text("Sub dir depth"),
            button(icon_arrow_down().size(12))
                .padding(2)
                .on_press_maybe(if 0 < self.sub_dir_depth_limit {
                    Some(Message::SubDirDepthLimitChanged(
                        self.sub_dir_depth_limit - 1,
                    ))
                } else {
                    None
                }),
            text(self.sub_dir_depth_limit),
            button(icon_arrow_up().size(12)).padding(2).on_press_maybe(
                if self.sub_dir_depth_limit < 2 {
                    Some(Message::SubDirDepthLimitChanged(
                        self.sub_dir_depth_limit + 1,
                    ))
                } else {
                    None
                }
            ),
        ]
        .spacing(5);

        let threshold_slider = row![
            text("Similarity"),
            text("0.50").style(text::secondary),
            slider(0.50_f32..=1.00_f32, self.similarity_threshold, |v| {
                // Round to 2 decimal places so small float noise doesn't
                // propagate to persistent settings.
                Message::SimilarityThresholdChanged((v * 100.0).round() / 100.0)
            })
            .step(0.01_f32),
            text("1.00").style(text::secondary),
            text(format!("{:.2}", self.similarity_threshold)),
        ]
        .spacing(8);

        container(
            column![target_media_types, sub_dir_depth_limit, threshold_slider].spacing(10),
        )
        .into()
    }
}
