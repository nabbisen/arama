use std::path::PathBuf;

use arama_env::MAX_THUMBNAIL_SIZE;
use iced::widget::{column, container, image, mouse_area, row, space, text};
use iced::{Element, widget::scrollable};

use crate::dialog::similarity_read_outcome::{absence_message, status_line};

use super::{SimilarPairsDialog, message::Message};

impl SimilarPairsDialog {
    pub fn view(&self) -> Element<'_, Message> {
        // `None` for at most one frame after the dialog opens, before
        // `EmbeddingsReady` arrives - not a state a user meaningfully
        // sees, so no text is required here the way RFC 036's binding
        // rule requires for the settled states below.
        let pairs = match &self.pairs {
            Some(x) => x,
            None => return container(space()).into(),
        };

        let status: Element<'_, Message> =
            status_line(self.has_read_error, self.ffmpeg_missing_with_videos);

        let header = if let Some(x) = &self.hovered_media_item_path_str {
            container(text(x))
        } else {
            container(space())
        }
        .height(20);

        // RFC 036: an empty `pairs` must still render text explaining why,
        // unless a read failure already explained it via `status` above -
        // showing both would contradict "results may be incomplete" with
        // a confident "nothing similar found".
        let body: Element<'_, Message> = if pairs.is_empty() {
            if self.has_read_error {
                container(space()).into()
            } else {
                absence_message(self.nothing_indexed)
            }
        } else {
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
            mouse_area(scrollable(pairs_columns))
                .on_exit(Message::MediaExit)
                .into()
        };

        column![status, header, body]
            .spacing(10)
            .padding([10, 0])
            .into()
    }
}
