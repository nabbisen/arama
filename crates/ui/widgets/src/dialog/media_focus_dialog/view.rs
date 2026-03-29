use arama_env::MAX_THUMBNAIL_SIZE;
use iced::{
    Alignment::Center,
    widget::{
        Row,
        image::Handle,
        mouse_area, pick_list, row,
        scrollable::{Direction, Scrollbar},
        space, text,
    },
};
use iced::{
    Element,
    Length::Fill,
    widget::{button, column, container, image, scrollable},
};

use super::{MediaFocusDialog, message::Message};

impl MediaFocusDialog {
    pub fn view(&self) -> Element<'_, Message> {
        let path = self.history[self.history_index].clone();

        let path_text = container(text(path.to_string_lossy().to_string())).center_x(Fill);

        let handle = Handle::from_path(&path);
        let img = image(handle);
        let img_container = if self.actual_size {
            scrollable(
                container(img)
                    .width(Fill)
                    .height(Fill)
                    .center(Fill)
                    .padding(10),
            )
            .direction(Direction::Both {
                vertical: Scrollbar::default(),
                horizontal: Scrollbar::default(),
            })
            .width(Fill)
            .height(Fill)
        } else {
            scrollable(container(img).width(Fill).center(Fill))
                .width(Fill)
                .height(Fill)
        };
        let content = mouse_area(img_container)
            .on_double_click(Message::ViewSizeToggle)
            .interaction(iced::mouse::Interaction::Pointer);

        let main_media = column![path_text, content].spacing(10);

        let cache_lookup_strategy_pick_list = row![
            text("Cache lookup strategy"),
            pick_list(
                &super::CacheLookupStrategy::ALL[..],
                Some(self.cache_lookup_strategy),
                Message::CacheLookupStrategyChanged,
            )
        ]
        .spacing(10)
        .padding([0, 20])
        .align_y(Center);

        let history_previous_button = button("⬅").on_press_maybe(if 0 < self.history_index {
            Some(Message::HistoryPrevious)
        } else {
            None
        });
        let history_next_button =
            button("➡").on_press_maybe(if self.history_index < self.history.len() - 1 {
                Some(Message::HistoryNext)
            } else {
                None
            });

        let explore_button = button("📂").on_press(Message::FileShow);

        let control_buttons = row![
            cache_lookup_strategy_pick_list,
            history_previous_button,
            history_next_button,
            explore_button
        ]
        .spacing(10);

        let view_control = container(control_buttons).center_x(Fill);

        // todo
        let similar_media_items = self.similar_media.iter().fold(row![], |r: Row<_>, x| {
            let handle = Handle::from_path(if let Some(thumbnail_path) = &x.thumbnail_path {
                thumbnail_path.to_owned()
            } else {
                x.path.to_owned()
            });
            let item = mouse_area(
                image(handle)
                    .width(MAX_THUMBNAIL_SIZE as u32)
                    .height(MAX_THUMBNAIL_SIZE as u32)
                    .content_fit(iced::ContentFit::Cover),
            )
            .on_enter(Message::MediaItemEnter(x.path.clone()))
            .on_double_click(Message::SimilarMediaItemDoubleClicked(
                x.path.to_owned().into(),
            ))
            .interaction(iced::mouse::Interaction::Pointer);
            r.push(column![item, text(x.similarity)].spacing(5).padding(10))
        });
        let similar_media_items_footer = if let Some(x) = &self.hovered_media_item_path_str {
            container(text(x))
        } else {
            container(space())
        }
        .height(20);
        let similar_media = column![
            mouse_area(scrollable(container(similar_media_items)).horizontal())
                .on_exit(Message::MediaItemExit),
            similar_media_items_footer
        ];

        let close_button = button("Close").on_press(Message::CloseClick);
        let footer = container(close_button).center_x(Fill).padding(10);

        column![main_media, view_control, similar_media, footer]
            .spacing(20)
            .into()
    }
}
