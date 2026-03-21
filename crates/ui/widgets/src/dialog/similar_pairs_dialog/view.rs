use std::path::PathBuf;

use arama_env::MAX_THUMBNAIL_SIZE;
use iced::widget::{column, container, image, mouse_area, row, space, text};
use iced::{Element, widget::scrollable};

use super::{SimilarPairsDialog, message::Message};

impl SimilarPairsDialog {
    pub fn view(&self) -> Element<'_, Message> {
        let pairs = match &self.pairs {
            Some(x) => x,
            None => return text("No valid pairs.").into(),
        };

        let header = if let Some(x) = &self.hovered_media_item_path_str {
            container(text(x))
        } else {
            container(space())
        }
        .height(20);

        let pairs_columns = pairs.iter().fold(column![].spacing(10), |c, x| {
            c.push(column![
                text(x.similarity.to_string()),
                row![
                    mouse_area(
                        image(PathBuf::from(if let Some(x) = &x.left.thumbnail_path {
                            x.to_owned()
                        } else {
                            x.left.path.to_owned()
                        }))
                        .width(MAX_THUMBNAIL_SIZE as u32)
                        .height(MAX_THUMBNAIL_SIZE as u32)
                        .content_fit(iced::ContentFit::Cover)
                    )
                    .on_enter(Message::MediaItemEnter(x.left.path.to_owned()))
                    .on_double_click(Message::MediaItemDoubleClicked(x.left.path.to_owned()))
                    .interaction(iced::mouse::Interaction::Pointer),
                    mouse_area(
                        image(PathBuf::from(if let Some(x) = &x.right.thumbnail_path {
                            x.to_owned()
                        } else {
                            x.right.path.to_owned()
                        }))
                        .width(MAX_THUMBNAIL_SIZE as u32)
                        .height(MAX_THUMBNAIL_SIZE as u32)
                        .content_fit(iced::ContentFit::Cover)
                    )
                    .on_enter(Message::MediaItemEnter(x.right.path.to_owned()))
                    .on_double_click(Message::MediaItemDoubleClicked(x.right.path.to_owned()))
                    .interaction(iced::mouse::Interaction::Pointer),
                ]
                .spacing(10),
            ])
        });
        let pairs = mouse_area(scrollable(pairs_columns)).on_exit(Message::MediaExit);

        column![header, pairs].spacing(10).padding([10, 0]).into()
    }
}
