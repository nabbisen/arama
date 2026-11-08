use std::path::PathBuf;

use arama_env::MAX_THUMBNAIL_SIZE;
use arama_i18n::{t, t_with};
use iced::widget::{column, container, image, mouse_area, row, space, text};
use iced::{Element, widget::scrollable};

use crate::dialog::similarity_read_outcome::{absence_message, status_line};

use super::{SimilarPairsDialog, message::Message};

impl SimilarPairsDialog {
    pub fn view(&self) -> Element<'_, Message> {
        // RFC 047 S3.3/claim #2: the scan runs in the background and
        // produces one result when it completes - it does not stream
        // partial results (a non-goal, RFC 047 S5). But it is not
        // instant either, and rendering nothing at all while it runs is
        // exactly the "appears to do nothing" shape the rest of this RFC
        // is about. This is a static sentence, not a progress bar or a
        // live count - "reports that it is running" (S3.3), not
        // progressive delivery.
        let pairs = match &self.pairs {
            Some(x) => x,
            None => {
                return container(text(t("similarity.scanning")).size(arama_theme::body_size()))
                    .into();
            }
        };

        let status: Element<'_, Message> =
            status_line(self.has_read_error, self.ffmpeg_missing_with_videos);

        let truncation_notice: Element<'_, Message> =
            match truncation_notice_text(pairs.len(), self.total_found) {
                Some(notice) => text(notice).size(arama_theme::body_size()).into(),
                None => container(space()).into(),
            };

        let header = if let Some(x) = &self.hovered_media_item_path_str {
            container(text(x))
        } else {
            container(space())
        }
        .height(20);

        // RFC 036: an empty `pairs` must still render text explaining why,
        // unless a read failure already explained it via `status` above -
        // showing both would contradict "results may be incomplete" with
        // a confident "nothing similar found".
        let body: Element<'_, Message> = if pairs.is_empty() {
            if self.has_read_error {
                container(space()).into()
            } else {
                absence_message(self.nothing_indexed)
            }
        } else {
            let pairs_columns = pairs.iter().fold(column![].spacing(10), |c, x| {
                c.push(column![
                    text(x.similarity.to_string()),
                    row![
                        mouse_area(
                            image(PathBuf::from(if let Some(x) = &x.left.thumbnail_path {
                                x.to_owned()
                            } else {
                                x.left.path.to_owned()
                            }))
                            .width(MAX_THUMBNAIL_SIZE as u32)
                            .height(MAX_THUMBNAIL_SIZE as u32)
                            .content_fit(iced::ContentFit::Cover)
                        )
                        .on_enter(Message::MediaItemEnter(x.left.path.to_owned()))
                        .on_double_click(Message::MediaItemDoubleClicked(x.left.path.to_owned()))
                        .interaction(iced::mouse::Interaction::Pointer),
                        mouse_area(
                            image(PathBuf::from(if let Some(x) = &x.right.thumbnail_path {
                                x.to_owned()
                            } else {
                                x.right.path.to_owned()
                            }))
                            .width(MAX_THUMBNAIL_SIZE as u32)
                            .height(MAX_THUMBNAIL_SIZE as u32)
                            .content_fit(iced::ContentFit::Cover)
                        )
                        .on_enter(Message::MediaItemEnter(x.right.path.to_owned()))
                        .on_double_click(Message::MediaItemDoubleClicked(x.right.path.to_owned()))
                        .interaction(iced::mouse::Interaction::Pointer),
                    ]
                    .spacing(10),
                ])
            });
            mouse_area(scrollable(pairs_columns))
                .on_exit(Message::MediaExit)
                .into()
        };

        column![status, truncation_notice, header, body]
            .spacing(10)
            .padding([10, 0])
            .into()
    }
}

/// RFC 047 S3.3: `Some` only when the cap actually truncated something -
/// a cap that kept everything says nothing ("showing 50 of 50" would be
/// noise, not information). Distinguishes "these are the similar pairs"
/// from "these are the top N of M" (RFC 036's absence states made the
/// same distinction one level up, for zero results versus none
/// indexed). Pure and separate from `view()` so the condition and the
/// substitution are directly testable without rendering anything,
/// mirroring `similarity_read_outcome::status_line_text`.
fn truncation_notice_text(retained: usize, total_found: usize) -> Option<String> {
    (total_found > retained).then(|| {
        t_with(
            "similarity.pairs_truncated",
            &[
                ("{retained}", &retained.to_string()),
                ("{total}", &total_found.to_string()),
            ],
        )
    })
}

#[cfg(test)]
mod tests {
    use super::truncation_notice_text;

    #[test]
    fn no_notice_when_nothing_was_truncated() {
        assert_eq!(truncation_notice_text(50, 50), None);
        assert_eq!(truncation_notice_text(3, 3), None);
        assert_eq!(truncation_notice_text(0, 0), None);
    }

    #[test]
    fn notice_names_both_counts_when_truncated() {
        let notice = truncation_notice_text(50, 287).expect("287 > 50 must produce a notice");
        assert!(notice.contains("50"), "{notice}");
        assert!(notice.contains("287"), "{notice}");
        assert!(!notice.contains('{'), "{notice}");
    }
}
