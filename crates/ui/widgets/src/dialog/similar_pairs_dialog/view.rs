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

        let pairs_columns = pairs
            .iter()
            .fold(column![].spacing(10), |c, (path1, path2, score)| {
                c.push(column![
                    text(score.to_string()),
                    row![
                        mouse_area(
                            image(path1.to_owned())
                                .width(MAX_THUMBNAIL_SIZE as u32)
                                .height(MAX_THUMBNAIL_SIZE as u32)
                                .content_fit(iced::ContentFit::Cover)
                        )
                        .on_enter(Message::MediaItemEnter(path1.to_owned())),
                        mouse_area(
                            image(path2.to_owned())
                                .width(MAX_THUMBNAIL_SIZE as u32)
                                .height(MAX_THUMBNAIL_SIZE as u32)
                                .content_fit(iced::ContentFit::Cover)
                        )
                        .on_enter(Message::MediaItemEnter(path2.to_owned())),
                    ]
                    .spacing(10),
                ])
            });
        let pairs = mouse_area(scrollable(pairs_columns)).on_exit(Message::MediaExit);

        column![header, pairs].spacing(10).padding([10, 0]).into()
    }
}
