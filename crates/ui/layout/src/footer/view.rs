use arama_i18n::t;
use iced::{
    Element,
    Length::{Fill, Shrink},
    widget::{container, row, space, text, text::Wrapping},
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
                // T055: `Fill` rather than a `FillPortion` tied to the
                // status group's own portion. The path is already
                // elided-by-nature (the user can see it in full elsewhere),
                // so it is the element that absorbs whatever width the
                // status group doesn't need, in either direction - wide
                // when status is short (English), narrow when status is
                // long (Japanese). `Wrapping::None` + `clip(true)` make
                // that narrowing a horizontal truncation instead of a wrap
                // that would grow into a second line and hit the same
                // fixed-height clip this task exists to fix.
                if let Some(x) = &self.image_cell_path {
                    container(
                        text(
                            x.canonicalize()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string(),
                        )
                        .wrapping(Wrapping::None),
                    )
                } else {
                    container(space())
                }
                .align_left(Fill)
                .clip(true),
                // `Shrink`, not a fixed portion: this group's width comes
                // from what the slider + counts + F6 hint actually need in
                // the active locale, instead of an arbitrary one third of
                // the footer that Japanese doesn't fit in (T055 §2). The
                // slider itself is given an explicit width for this to
                // resolve (see thumbnail_size_slider/view.rs) - a `Fill`
                // child under a `Shrink` ancestor has no content size to
                // contribute and collapses instead.
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
                            // exact place someone re-orients from. T055
                            // leaves this element untouched - the fix is the
                            // footer's width allocation, not this hint.
                            text(t("footer.f6_hint")).style(text::secondary),
                        ]
                        .spacing(10)
                    ]
                    .spacing(30)
                )
                .align_right(Shrink),
            ]
            .spacing(10),
        )
        .padding([10, 20])
        .align_right(Fill)
        .height(Shrink)
        .into()
    }
}

#[cfg(test)]
mod tests {
    use super::{dirs_line, files_line};

    // English-only, deliberately: `arama-ui-layout`'s test binary runs
    // these functions' tests in parallel and none of them expect
    // `arama_i18n`'s global locale to move under them - the process
    // default is already `Locale::En` (Task 043 / audit B7), so no
    // `set_locale` call is needed here. A prior version of this module
    // had a fourth test that looped over both locales and was found to
    // race these three when the full workspace suite ran repeatedly -
    // the same shape of race `app/src/core/tests.rs` and
    // `app/src/core/update/cache.rs` each already document. That
    // guarantee (`footer.files_count`/`footer.dirs_scanned` resolve to
    // real text in both locales) now lives in `arama-i18n`'s own,
    // much smaller and already-ordered test binary instead
    // (`crates/i18n/src/tests.rs`'s
    // `task_043_footer_keys_resolve_to_real_text_in_both_locales`),
    // where mutating the global locale is safe.

    #[test]
    fn files_line_does_not_depend_on_dirs_count() {
        assert_eq!(files_line(27), "27 files");
        assert_eq!(files_line(1), "1 files");
        assert_eq!(files_line(0), "0 files");
    }

    #[test]
    fn dirs_line_does_not_depend_on_files_count() {
        assert_eq!(dirs_line(1), "(1 dirs scanned)");
        assert_eq!(dirs_line(3), "(3 dirs scanned)");
        assert_eq!(dirs_line(0), "(0 dirs scanned)");
    }

    /// The exact case that previously read wrong: 1 file across 3
    /// directories used to report "(3 dir scanned)" because the directory
    /// label branched on `files_count`, not `dirs_count`.
    #[test]
    fn one_file_across_many_directories_reads_correctly() {
        assert_eq!(files_line(1), "1 files");
        assert_eq!(dirs_line(3), "(3 dirs scanned)");
    }
}
