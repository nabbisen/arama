use std::path::PathBuf;

use arama_cache::{DbLocation, ImageCacheReader, LookupResult};
use arama_env::cache_storage_path;
use iced::{
    Element,
    Length::Fill,
    widget::{container, image, mouse_area, text},
};
use image::Handle;

use super::{ImageCell, message::Message};

impl ImageCell {
    // 'static lifetime against borrow checker in chunk()...map() list operation
    pub fn view(self) -> Element<'static, Message> {
        let thumbnail_path = match ImageCacheReader::onetime(DbLocation::Custom(
            cache_storage_path().expect("failed to get storage path"),
        ))
        .expect("failed to get cache reader")
        .lookup(&self.path)
        {
            Ok(x) => match x {
                LookupResult::Hit(x) => {
                    let path = if let Some(path) = x.thumbnail_path {
                        PathBuf::from(path)
                    } else {
                        self.path.clone()
                    };
                    path
                }
                _ => self.path.clone(),
            },
            Err(err) => {
                return container(text(err.to_string()))
                    .width(self.thumbnail_size)
                    .height(self.thumbnail_size)
                    .center(Fill)
                    .into();
            }
        };

        let handle = Handle::from_path(&thumbnail_path);

        let content = mouse_area(
            image(handle)
                .width(self.thumbnail_size)
                .height(self.thumbnail_size)
                .content_fit(iced::ContentFit::Cover),
        )
        .on_double_click(Message::ImageSelect(self.path.clone()))
        .on_right_press(Message::ContextMenuOpen(self.path));

        content.into()
    }
}
