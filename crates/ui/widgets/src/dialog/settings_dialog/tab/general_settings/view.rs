use arama_env::target_media_type::TargetMediaType;
use iced::{
    Element,
    widget::{button, checkbox, column, container, row, text},
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

        container(column![target_media_types, sub_dir_depth_limit].spacing(10)).into()
    }
}
