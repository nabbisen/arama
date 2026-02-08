use iced::widget::{
    Responsive, column, container, image, mouse_area, row, scrollable, space, text,
};
use iced::{Element, Length, Size};
use swdir::DirNode;

use crate::app::utils::gallery::image_similarity::ImageSimilarity;

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

        let selected_source_image_label = text(
            if let Some(selected_source_image) = self.selected_source_image.as_ref() {
                let mut ret = selected_source_image.to_string_lossy().to_string();
                if self.running {
                    ret = format!("{} (calculating...)", ret);
                }
                ret
            } else {
                "".into()
            },
        );

        let content = if self.dir_node.is_none() {
            container(text(""))
        } else if self
            .dir_node
            .as_ref()
            .is_some_and(|dir_node| dir_node.sub_dirs.is_empty() && dir_node.files.is_empty())
        {
            container(text("No images found in folder(s)."))
        } else {
            // Responsiveウィジェットを使って、現在のウィンドウ幅(size)を取得する
            container(Responsive::new(move |size| self.view_grid(size)))
        };

        let container = container(content)
            .center_x(Length::Fill)
            .center_y(Length::Fill);

        // スクロール可能にする
        let scrollable = scrollable(container);

        // settings
        let mut scrollable_with_settings = column![];
        if !self.image_similarity.is_empty() {
            scrollable_with_settings = scrollable_with_settings.push(
                self.gallery_settings
                    .view()
                    .map(Message::GallerySettingsMessage),
            );
        }
        scrollable_with_settings = scrollable_with_settings.push(scrollable);

        column![
            menus,
            root_dir_select,
            selected_source_image_label,
            scrollable_with_settings
        ]
        .into()
    }

    // グリッドレイアウトの計算ロジック
    fn view_grid(&self, size: Size) -> Element<'_, Message> {
        if self.dir_node.is_none() {
            return space().into();
        }

        let total_width = size.width;
        let item_width = self.thumbnail_size as f32 + self.spacing as f32;

        // 1行に収まるカラム数を計算 (ゼロ除算回避のためmax(1)を使用)
        let columns = (total_width / item_width).floor() as usize;
        let columns = columns.max(1);

        if let Some(image_columns) = image_columns(
            self.dir_node.as_ref().unwrap(),
            &self.image_similarity,
            self.gallery_settings.similarity_quality(),
            columns,
            self.thumbnail_size,
            self.spacing,
        ) {
            image_columns
        } else {
            space().into()
        }
    }
}

fn image_columns<'a>(
    dir_node: &'a DirNode,
    image_similarity: &'a ImageSimilarity,
    similarity_quality: f32,
    columns: usize,
    thumbnail_size: u32,
    spacing: u32,
) -> Option<Element<'a, Message>> {
    // 画像パスのリストを、カラム数ごとに分割（チャンク化）して行を作成
    let files_rows: Vec<Element<Message>> = dir_node
        .files
        .chunks(columns)
        .map(|chunk| {
            let images: Vec<Element<Message>> = chunk
                .iter()
                .filter(|path| {
                    if let Some(image_similarity) = image_similarity.get_score(path) {
                        similarity_quality <= image_similarity
                    } else {
                        true
                    }
                })
                .map(|path| {
                    // 画像ウィジェットの作成
                    // ContentFit::Coverで正方形にトリミング表示
                    let image = image(path.as_path())
                        .width(thumbnail_size)
                        .height(thumbnail_size)
                        .content_fit(iced::ContentFit::Cover);
                    let image_similarity =
                        if let Some(image_similarity) = image_similarity.get_score(path) {
                            image_similarity.to_string()
                        } else {
                            "".into()
                        };
                    column![
                        mouse_area(image).on_double_click(Message::ImageSelect(path.clone())),
                        text(image_similarity)
                    ]
                    .into()
                })
                .collect();

            // 画像を横に並べる
            row(images).spacing(spacing).into()
        })
        .collect();

    let sub_dirs_rows = dir_node
        .sub_dirs
        .iter()
        .map(|sub_dir_node| {
            image_columns(
                sub_dir_node,
                image_similarity,
                similarity_quality,
                columns,
                thumbnail_size,
                spacing,
            )
        })
        .filter(|x| x.is_some())
        .collect::<Vec<Option<Element<Message>>>>();

    if files_rows.is_empty() && sub_dirs_rows.is_empty() {
        return None;
    }

    // 行を縦に並べる
    let mut ret = column![];

    if !files_rows.is_empty() {
        ret = ret.push(text(dir_node.path.to_string_lossy()));
        ret = ret.extend(files_rows);
    }

    if !sub_dirs_rows.is_empty() {
        ret = ret.extend(sub_dirs_rows.into_iter().map(|x| x.unwrap()));
    }

    Some(ret.spacing(spacing).into())
}
