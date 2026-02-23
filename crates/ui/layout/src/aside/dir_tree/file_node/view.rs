use std::path::PathBuf;

use iced::font::Weight;
use iced::widget::{Column, container, mouse_area, row, scrollable, text};
use iced::{Element, Font, mouse};

use super::FileNode;
use super::message::Message;

impl FileNode {
    pub fn view(
        &self,
        selected_path: &Option<PathBuf>,
        depth: u32,
        include_file: bool,
        include_hidden: bool,
        processing: bool,
    ) -> Element<'_, Message> {
        let mut content = Column::new().spacing(5);

        // 1行分の表示（アイコン + 名前）
        let icon = if !self.is_dir {
            "📄"
        } else if self.is_expanded {
            "📂"
        } else {
            "📁"
        };
        // フォルダ名部分のボタン（クリックで選択）
        let txt = format!("{} {}", icon, self.name);

        let selected = selected_path.as_ref() == Some(&self.path);

        let mut label = mouse_area(
            container(text(txt).font(if selected {
                Font {
                    weight: Weight::Bold,
                    ..Font::DEFAULT
                }
            } else {
                Font::DEFAULT
            }))
            .padding([2, 5]),
        );
        if !processing {
            label = label
                .interaction(mouse::Interaction::Pointer)
                .on_press(Message::DirClick(self.path.clone()))
                .on_double_click(Message::ToggleExpand((
                    self.path.clone(),
                    include_file,
                    include_hidden,
                )));
        }

        // 開閉切り替えボタン（ディレクトリの場合のみ）
        let row_content = if self.is_dir {
            let mut toggle_btn = mouse_area(text(if self.is_expanded { "▼" } else { "▶" }));
            if !processing {
                toggle_btn = toggle_btn
                    .on_press(Message::ToggleExpand((
                        self.path.clone(),
                        include_file,
                        include_hidden,
                    )))
                    .interaction(mouse::Interaction::Pointer);
            }
            row![toggle_btn, label].spacing(5)
        } else {
            row![text("  "), label].spacing(5)
        };

        // インデントの適用
        let row_container = container(row_content).padding([0, (depth * 20) as u16]);
        content = content.push(row_container);

        // 展開されている場合、子要素を再帰的に描画
        if self.is_expanded && self.is_dir {
            for child in &self.children {
                content = content.push(child.view(
                    selected_path,
                    depth + 1,
                    include_file,
                    include_hidden,
                    processing,
                ))
            }
        }

        scrollable(content).width(320).into()
    }
}
