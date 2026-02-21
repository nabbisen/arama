use iced::{
    Element,
    widget::{row, text},
};

pub mod gallery;

use super::{App, message::Message};
use arama_embedding::model::clip;

impl App {
    pub fn view(&self) -> Element<'_, Message> {
        let has_model = match clip::model().ready() {
            Ok(x) => x,
            Err(err) => return text(err.to_string()).into(),
        };
        if !has_model {
            return self
                .model_loader
                .view()
                .map(Message::ModelLoaderMessage)
                .into();
        }

        let gallery = self
            .gallery
            .view()
            .map(|message| Message::GalleryMessage(message));

        let dir_tree = self.dir_tree.view().map(Message::DirTreeMessage);

        row([dir_tree.into(), gallery.into()]).into()
    }
}
