use iced::widget::{column, container, image, row, space, text};
use iced::{Element, widget::scrollable};

use super::{SimilarPairs, message::Message};

impl SimilarPairs {
    pub fn view(&self) -> Element<'_, Message> {
        let pairs = match &self.pairs {
            Some(x) => x,
            None => return space().into(),
        };

        let rows: Vec<Element<Message>> = pairs
            .iter()
            .map(|(path1, path2, score)| {
                row![
                    image(path1.to_owned()),
                    image(path2.to_owned()),
                    text(score.to_string())
                ]
                .into()
            })
            .collect();

        row![container(scrollable(column(rows))), text("test")].into()
    }
}
