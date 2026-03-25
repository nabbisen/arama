use arama_env::target_media_type::TargetMediaType;
use iced::{
    Element,
    widget::{checkbox, container, row},
};

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

        container(target_media_types).into()
    }
}
