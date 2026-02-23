use iced::widget::{column, container, image, row, space, text};
use iced::{Element, widget::scrollable};

use super::{SimilarPairsDialog, message::Message};

impl SimilarPairsDialog {
    pub fn view(&self) -> Element<'_, Message> {
        let pairs = match &self.pairs {
            Some(x) => x,
            None => return space().into(),
        };

        let rows: Vec<Element<Message>> = pairs
            .iter()
            .map(|(path1, path2, score)| {
                column![
                    text(score.to_string()),
                    row![
                        image(path1.to_owned())
                            .width(self.thumbnail_size)
                            .height(self.thumbnail_size)
                            .content_fit(iced::ContentFit::Cover),
                        image(path2.to_owned())
                            .width(self.thumbnail_size)
                            .height(self.thumbnail_size)
                            .content_fit(iced::ContentFit::Cover),
                    ]
                ]
                .padding([10, 0])
                .into()
            })
            .collect();

        row![container(scrollable(column(rows))), text("test")].into()
    }
}
