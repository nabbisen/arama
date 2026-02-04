use iced::widget::{Responsive, column, container, image, row, scrollable, text};
use iced::{Element, Length, Size};

use super::{Gallery, message::Message};

impl Gallery {
    // ビュー（UI描画）
    pub fn view(&self) -> Element<'_, Message> {
        let menus = self
            .menus
            .view()
            .map(|message| Message::MenusMessage(message));

        let root_dir_select = self
            .root_dir_select
            .view()
            .map(|message| Message::RootDirSelectMessage(message));

        let content = if self.image_paths.is_empty() {
            container(text("No images found in this folder."))
        } else {
            // Responsiveウィジェットを使って、現在のウィンドウ幅(size)を取得する
            container(Responsive::new(move |size| self.view_grid(size)))
        };

        let container = container(content)
            .center_x(Length::Fill)
            .center_y(Length::Fill);

        // スクロール可能にする
        let scrollable = scrollable(container);

        column![menus, root_dir_select, scrollable].into()
    }

    // グリッドレイアウトの計算ロジック
    fn view_grid(&self, size: Size) -> Element<'_, Message> {
        let total_width = size.width;
        let item_width = self.thumbnail_size as f32 + self.spacing as f32;

        // 1行に収まるカラム数を計算 (ゼロ除算回避のためmax(1)を使用)
        let columns = (total_width / item_width).floor() as usize;
        let columns = columns.max(1);

        // 画像パスのリストを、カラム数ごとに分割（チャンク化）して行を作成
        let rows: Vec<Element<Message>> = self
            .image_paths
            .chunks(columns)
            .map(|chunk| {
                let images: Vec<Element<Message>> = chunk
                    .iter()
                    .map(|path| {
                        // 画像ウィジェットの作成
                        // ContentFit::Coverで正方形にトリミング表示
                        image(path)
                            .width(self.thumbnail_size)
                            .height(self.thumbnail_size)
                            .content_fit(iced::ContentFit::Cover)
                            .into()
                    })
                    .collect();

                // 画像を横に並べる
                row(images).spacing(self.spacing).into()
            })
            .collect();

        // 行を縦に並べる
        column(rows).spacing(self.spacing).into()
    }
}
