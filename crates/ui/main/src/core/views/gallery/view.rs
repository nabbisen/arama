use arama_i18n::t;
use iced::Length::Fill;
use iced::widget::{Responsive, column, container, mouse_area, row, scrollable, text};
use iced::{Element, Size};

use crate::components::gallery::image_cell::ImageCell;

use super::{Gallery, SPACING, message::Message};

impl Gallery {
    pub fn view(&self, thumbnail_size: u16) -> Element<'_, Message> {
        // Use a Responsive widget to obtain the current window width and
        // compute column count from it.
        let grid = container(Responsive::new(move |responsive_size| {
            self.grid(responsive_size, thumbnail_size)
                .unwrap_or_else(|| text(t("gallery.empty")).into())
        }));

        let content = mouse_area(scrollable(container(grid).center_x(Fill).center_y(Fill)))
            .on_exit(Message::CursorExit);

        content.into()
    }

    // Compute the grid layout and return column-of-rows.
    fn grid(&self, responsive_size: Size, thumbnail_size: u16) -> Option<Element<'_, Message>> {
        let total_width = responsive_size.width;
        let item_width = (thumbnail_size + SPACING) as f32;
        // Number of columns per row; clamped to at least 1 to avoid division by zero.
        let num_of_columns_in_row = ((total_width / item_width).floor() as usize).max(1);
        self.columns_in_rows(num_of_columns_in_row, thumbnail_size)
    }

    fn columns_in_rows(
        &self,
        num_of_columns_in_row: usize,
        thumbnail_size: u16,
    ) -> Option<Element<'_, Message>> {
        let thumbnail_size = thumbnail_size as u32;

        let content = self
            .dir_path_thumbnail_path_map
            .iter()
            .map(|(dir_path, map)| {
                let mut ret = vec![text(dir_path.to_string_lossy().to_string()).into()];

                let grid = map
                    .iter()
                    .collect::<Vec<_>>()
                    .chunks(num_of_columns_in_row)
                    .map(|chunk| {
                        row(chunk
                            .iter()
                            .map(|(path, thumbnail_path)| {
                                ImageCell::new(&path, &thumbnail_path, thumbnail_size)
                                    .view()
                                    .map(Message::ImageCellMessage)
                            })
                            .collect::<Vec<Element<Message>>>())
                        .spacing(SPACING as u32)
                        .into()
                    })
                    .collect::<Vec<Element<Message>>>();

                ret.extend(grid);
                ret
            })
            .filter(|x| 1 < x.len())
            .collect::<Vec<Vec<Element<Message>>>>();

        if content.len() == 0 {
            None
        } else {
            Some(
                content
                    .into_iter()
                    .fold(column![].spacing(SPACING as u32), |c, x| c.extend(x))
                    .into(),
            )
        }
    }
}
