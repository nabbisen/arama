use std::path::Path;

use arama_i18n::t;
use iced::Length::Fill;
use iced::widget::{
    Responsive, button, column, container, mouse_area, row, scrollable, text, text_input,
};
use iced::{Element, Size};

use crate::components::gallery::image_cell::ImageCell;

use super::{Gallery, SPACING, message::Message};

impl Gallery {
    pub fn view(&self, thumbnail_size: u16) -> Element<'_, Message> {
        // ── Filter row ────────────────────────────────────────────────
        let total_files: usize = self
            .dir_path_thumbnail_path_map
            .values()
            .map(|m| m.len())
            .sum();

        let filter_active = !self.filter.is_empty();
        let matching: usize = if filter_active {
            let lc = self.filter.to_lowercase();
            self.dir_path_thumbnail_path_map
                .values()
                .flat_map(|m| m.keys())
                .filter(|p| filename_matches(p, &lc))
                .count()
        } else {
            total_files
        };

        let filter_row = row![
            text_input(&t("gallery.filter.placeholder"), &self.filter)
                .on_input(Message::FilterChanged),
            button(text(t("gallery.filter.clear")))
                .on_press_maybe(filter_active.then_some(Message::FilterClear)),
            text(if filter_active {
                format!("{} {} {}", matching, t("gallery.filter.count_of"), total_files)
            } else {
                String::new()
            })
            .style(text::secondary),
        ]
        .spacing(8);

        // ── Thumbnail grid ────────────────────────────────────────────
        let filter = self.filter.clone();
        let grid = container(Responsive::new(move |size| {
            self.grid(size, thumbnail_size, &filter)
                .unwrap_or_else(|| text(t("gallery.empty")).into())
        }));

        let content = mouse_area(scrollable(container(grid).center_x(Fill).center_y(Fill)))
            .on_exit(Message::CursorExit);

        column![filter_row, content].spacing(8).into()
    }

    fn grid(
        &self,
        responsive_size: Size,
        thumbnail_size: u16,
        filter: &str,
    ) -> Option<Element<'_, Message>> {
        let total_width = responsive_size.width;
        let item_width = (thumbnail_size + SPACING) as f32;
        let num_cols = ((total_width / item_width).floor() as usize).max(1);
        self.columns_in_rows(num_cols, thumbnail_size, filter)
    }

    fn columns_in_rows(
        &self,
        num_of_columns_in_row: usize,
        thumbnail_size: u16,
        filter: &str,
    ) -> Option<Element<'_, Message>> {
        let thumbnail_size = thumbnail_size as u32;
        let filter_lc = filter.to_lowercase();
        let filter_active = !filter_lc.is_empty();

        let content = self
            .dir_path_thumbnail_path_map
            .iter()
            .map(|(dir_path, map)| {
                // Apply filter: keep only entries whose filename matches.
                let entries: Vec<_> = map
                    .iter()
                    .filter(|(path, _)| {
                        !filter_active || filename_matches(path, &filter_lc)
                    })
                    .collect();

                if entries.is_empty() {
                    return vec![];
                }

                let mut ret: Vec<Element<'_, Message>> =
                    vec![text(dir_path.to_string_lossy().to_string()).into()];

                let grid = entries
                    .chunks(num_of_columns_in_row)
                    .map(|chunk| {
                        row(chunk
                            .iter()
                            .map(|(path, thumbnail_path)| {
                                ImageCell::new(path, thumbnail_path, thumbnail_size)
                                    .view()
                                    .map(Message::ImageCellMessage)
                            })
                            .collect::<Vec<_>>())
                        .spacing(SPACING as u32)
                        .into()
                    })
                    .collect::<Vec<Element<'_, Message>>>();

                ret.extend(grid);
                ret
            })
            .filter(|x| !x.is_empty())
            .collect::<Vec<_>>();

        if content.is_empty() {
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

/// Case-insensitive filename match: checks only the last path component.
fn filename_matches(path: &str, filter_lc: &str) -> bool {
    let filename = Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy())
        .unwrap_or_default();
    filename.to_lowercase().contains(filter_lc)
}
