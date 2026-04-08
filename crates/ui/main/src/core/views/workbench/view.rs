use iced::Length::Fill;
use iced::widget::{Responsive, column, container, mouse_area, row, scrollable, text};
use iced::{Element, Size};

use crate::components::workbench::image_cell::ImageCell;

use super::{SPACING, Workbench, message::Message};

impl Workbench {
    // ビュー（UI描画）
    pub fn view(&self, thumbnail_size: u16) -> Element<'_, Message> {
        // Responsiveウィジェットを使って、現在のウィンドウ幅(size)を取得する
        let grid = container(Responsive::new(move |responsive_size| {
            self.grid(responsive_size, thumbnail_size)
                .unwrap_or(text("No file to render.").into())
        }));

        let content = mouse_area(scrollable(container(grid).center_x(Fill).center_y(Fill)))
            .on_exit(Message::CursorExit);

        content.into()
    }

    // グリッドレイアウトの計算ロジック
    fn grid(&self, responsive_size: Size, thumbnail_size: u16) -> Option<Element<'_, Message>> {
        let total_width = responsive_size.width;
        let item_width = (thumbnail_size + SPACING) as f32;
        // 1行に収まるカラム数を計算 (ゼロ除算回避のためmax(1)を使用)
        let num_of_columns_in_row = (total_width / item_width).floor() as usize;
        let num_of_columns_in_row = num_of_columns_in_row.max(1);
        if let Some(columns_in_rows) = self.columns_in_rows(num_of_columns_in_row, thumbnail_size) {
            Some(columns_in_rows)
        } else {
            None
        }
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
