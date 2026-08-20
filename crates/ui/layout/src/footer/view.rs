use arama_i18n::t;
use iced::{
    Element,
    Length::{Fill, FillPortion},
    widget::{container, row, space, text},
};

use super::{Footer, message::Message};

impl Footer {
    pub fn view(&self) -> Element<'_, Message> {
        let files_label = if 1 < self.files_count {
            "files"
        } else {
            "file"
        };
        let dirs_label = if 1 < self.files_count { "dirs" } else { "dir" };

        container(
            row![
                if let Some(x) = &self.image_cell_path {
                    container(text(
                        x.canonicalize()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string(),
                    ))
                } else {
                    container(space())
                }
                .align_left(FillPortion(2)),
                container(
                    row![
                        self.thumbnail_size_slider
                            .view()
                            .map(Message::ThumbnailSizeSliderMessage),
                        row![
                            text(format!("{} {}", self.files_count, files_label))
                                .style(text::secondary),
                            text(format!("({} {} scanned)", self.dirs_count, dirs_label))
                                .style(text::secondary),
                            // RFC 044 §3.1: F6/Shift+F6 zone cycling has no
                            // other discoverable affordance - documentation
                            // alone is close to no binding at all, per
                            // snora's own review. Permanent rather than
                            // shown only on first movement: the footer is
                            // already one of the three zones this hint
                            // explains how to reach, so hiding it after one
                            // use would remove the explanation from the
                            // exact place someone re-orients from.
                            text(t("footer.f6_hint")).style(text::secondary),
                        ]
                        .spacing(10)
                    ]
                    .spacing(30)
                )
                .align_right(FillPortion(1)),
            ]
            .spacing(10),
        )
        .padding([10, 20])
        .align_right(Fill)
        .height(40)
        .into()
    }
}
