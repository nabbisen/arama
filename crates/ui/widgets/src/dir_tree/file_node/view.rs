use std::path::PathBuf;

use iced::font::Weight;
use iced::widget::{Column, container, mouse_area, row, scrollable, space, text};
use iced::{Element, Font};
use lucide_icons::iced::{
    icon_chevron_down, icon_chevron_right, icon_file, icon_folder, icon_folder_open,
};

use super::{FileNode, message::Message, util::is_hidden};

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

        let selected = selected_path.as_ref() == Some(&self.path);

        // 1行分の表示（アイコン + 名前）
        let icon = if !self.is_dir {
            icon_file()
        } else if self.is_expanded {
            icon_folder_open()
        } else {
            icon_folder()
        };

        let node_content = row![
            icon,
            text(&self.name).font(if selected {
                Font {
                    weight: Weight::Bold,
                    ..Font::DEFAULT
                }
            } else {
                Font::DEFAULT
            })
        ]
        .spacing(5);

        let mut node = mouse_area(container(node_content).padding([2, 5])).on_double_click(
            Message::ToggleExpand((self.path.clone(), include_file, include_hidden)),
        );

        if !processing {
            node = node
                .interaction(iced::mouse::Interaction::Pointer)
                .on_press(Message::DirClick(self.path.clone()))
                .on_double_click(Message::ToggleExpand((
                    self.path.clone(),
                    include_file,
                    include_hidden,
                )));
        }

        // 開閉切り替えボタン（ディレクトリの場合のみ）
        let is_dir_and_has_children = {
            if self.is_dir {
                std::fs::read_dir(&self.path).is_ok_and(|mut x| {
                    x.any(|x| x.is_ok_and(|x| x.path().is_dir() && !is_hidden(&x.path())))
                })
            } else {
                false
            }
        };
        let row_content = if is_dir_and_has_children {
            let mut toggle_btn = mouse_area(if self.is_expanded {
                icon_chevron_down()
            } else {
                icon_chevron_right()
            });

            if !processing {
                toggle_btn = toggle_btn
                    .on_press(Message::ToggleExpand((
                        self.path.clone(),
                        include_file,
                        include_hidden,
                    )))
                    .interaction(iced::mouse::Interaction::Pointer);
            }

            row![container(toggle_btn).width(16), node].spacing(5)
        } else {
            row![container(space()).width(16), node].spacing(5)
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
