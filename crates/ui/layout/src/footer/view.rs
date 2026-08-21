use arama_i18n::t;
use iced::{
    Element,
    Length::{Fill, FillPortion},
    widget::{container, row, space, text},
};

use super::{Footer, message::Message};

fn files_line(files_count: usize) -> String {
    format!("{} {}", files_count, t("footer.files_count"))
}

fn dirs_line(dirs_count: usize) -> String {
    format!("({} {})", dirs_count, t("footer.dirs_scanned"))
}

impl Footer {
    pub fn view(&self) -> Element<'_, Message> {
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
                            text(files_line(self.files_count)).style(text::secondary),
                            text(dirs_line(self.dirs_count)).style(text::secondary),
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

#[cfg(test)]
mod tests {
    use arama_i18n::{Locale, set_locale};

    use super::{dirs_line, files_line};

    #[test]
    fn files_line_does_not_depend_on_dirs_count() {
        set_locale(Locale::En);
        assert_eq!(files_line(27), "27 files");
        assert_eq!(files_line(1), "1 files");
        assert_eq!(files_line(0), "0 files");
    }

    #[test]
    fn dirs_line_does_not_depend_on_files_count() {
        set_locale(Locale::En);
        assert_eq!(dirs_line(1), "(1 dirs scanned)");
        assert_eq!(dirs_line(3), "(3 dirs scanned)");
        assert_eq!(dirs_line(0), "(0 dirs scanned)");
    }

    /// The exact case that previously read wrong: 1 file across 3
    /// directories used to report "(3 dir scanned)" because the directory
    /// label branched on `files_count`, not `dirs_count`.
    #[test]
    fn one_file_across_many_directories_reads_correctly() {
        set_locale(Locale::En);
        assert_eq!(files_line(1), "1 files");
        assert_eq!(dirs_line(3), "(3 dirs scanned)");
    }

    #[test]
    fn both_lines_resolve_to_real_text_in_both_locales() {
        for locale in Locale::all() {
            set_locale(*locale);
            assert_ne!(files_line(1), "1 footer.files_count");
            assert_ne!(dirs_line(1), "(1 footer.dirs_scanned)");
        }
        set_locale(Locale::En);
    }
}
