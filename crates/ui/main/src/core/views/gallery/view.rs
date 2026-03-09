use iced::Length::Fill;
use iced::widget::{Responsive, column, container, row, scrollable, space, text};
use iced::{Element, Size};

use crate::components::gallery::image_cell::ImageCell;

use super::{Gallery, SPACING, message::Message};

impl Gallery {
    // ビュー（UI描画）
    pub fn view(&self) -> Element<'_, Message> {
        column![
            self.gallery_settings
                .view()
                .map(Message::GallerySettingsMessage),
            self.content_view()
        ]
        .into()
    }

    fn content_view(&self) -> Element<'_, Message> {
        let label: Element<Message> = if let Some(dir_node) = self.dir_node.as_ref() {
            text(dir_node.path.to_string_lossy()).into()
        } else {
            space().into()
        };

        // Responsiveウィジェットを使って、現在のウィンドウ幅(size)を取得する
        let grid = container(Responsive::new(move |size| {
            self.grid(size).unwrap_or(text("No file to render.").into())
        }));
        let container = container(grid).center_x(Fill).center_y(Fill);
        let scrollable = scrollable(container);

        column![label, scrollable].into()
    }

    // グリッドレイアウトの計算ロジック
    fn grid(&self, size: Size) -> Option<Element<'_, Message>> {
        let total_width = size.width;
        let item_width = (self.gallery_settings.thumbnail_size() + SPACING) as f32;
        // 1行に収まるカラム数を計算 (ゼロ除算回避のためmax(1)を使用)
        let num_of_columns_in_row = (total_width / item_width).floor() as usize;
        let num_of_columns_in_row = num_of_columns_in_row.max(1);
        if let Some(columns_in_rows) = self.columns_in_rows(num_of_columns_in_row) {
            Some(columns_in_rows)
        } else {
            None
        }
    }

    fn columns_in_rows(&self, num_of_columns_in_row: usize) -> Option<Element<'_, Message>> {
        let thumbnail_size = self.gallery_settings.thumbnail_size() as u32;

        let content = self
            .path_thumbnail_path_map
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

        if content.len() == 0 {
            None
        } else {
            Some(column(content).spacing(SPACING as u32).into())
        }
    }
}
