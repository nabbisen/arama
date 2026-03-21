use arama_env::MAX_THUMBNAIL_SIZE;
use iced::widget::{column, image, row, space, text};
use iced::{Element, widget::scrollable};

use super::{SimilarPairsDialog, message::Message};

impl SimilarPairsDialog {
    pub fn view(&self) -> Element<'_, Message> {
        let pairs = match &self.pairs {
            Some(x) => x,
            None => return space().into(),
        };

        let columns = pairs
            .iter()
            .fold(column![].spacing(10), |c, (path1, path2, score)| {
                c.push(column![
                    text(score.to_string()),
                    row![
                        image(path1.to_owned())
                            .width(MAX_THUMBNAIL_SIZE as u32)
                            .height(MAX_THUMBNAIL_SIZE as u32)
                            .content_fit(iced::ContentFit::Cover),
                        image(path2.to_owned())
                            .width(MAX_THUMBNAIL_SIZE as u32)
                            .height(MAX_THUMBNAIL_SIZE as u32)
                            .content_fit(iced::ContentFit::Cover),
                    ]
                    .spacing(10),
                ])
            });

        scrollable(columns).into()
    }
}
