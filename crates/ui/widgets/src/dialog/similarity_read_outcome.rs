use arama_i18n::t;
use iced::Element;
use iced::widget::{container, space, text};

/// Result of a similarity-dialog cache read (RFC 035): the items obtained,
/// plus whether any read failed along the way. `had_errors` with a
/// non-empty `items` is a partial result; with an empty `items` it is a
/// failed lookup, and must not be presented as an absence of matches.
///
/// RFC 036 extends this to distinguish *why* `items` is empty when it is
/// not a failure: nothing indexed yet, versus a search that ran and found
/// nothing; and separately, whether video comparison did not run because
/// no ffmpeg/ffprobe pair was available. These are ordinary states, not
/// failures, but a dialog that renders nothing for any of them is
/// indistinguishable from one still loading or misfiring.
#[derive(Clone, Debug, Default)]
pub struct SimilarityReadOutcome<T> {
    pub items: Vec<T>,
    pub had_errors: bool,
    /// True when the search space (the target item, or the whole
    /// directory) had nothing indexed to compare against. Meaningful only
    /// when `items` is empty.
    pub nothing_indexed: bool,
    /// True when video paths were part of the search but no ffmpeg/ffprobe
    /// pair was available, so video comparison did not run. Independent
    /// of `had_errors` (RFC 035 §3.1: missing ffmpeg is not a read
    /// failure) and independent of `items`/`nothing_indexed`.
    pub ffmpeg_missing_with_videos: bool,
}

/// RFC 036: the ordered sentences for the dialogs' shared top status line —
/// at most two, a read failure (RFC 035, unchanged wording) and/or a
/// missing-ffmpeg statement — never merged into one sentence and never
/// repeated per item.
pub fn status_line_text(had_errors: bool, ffmpeg_missing_with_videos: bool) -> Vec<String> {
    let mut sentences = Vec::new();
    if had_errors {
        sentences.push(t("similarity.read_error"));
    }
    if ffmpeg_missing_with_videos {
        sentences.push(t("similarity.video_unavailable"));
    }
    sentences
}

/// Renders [`status_line_text`] as a single element. Literal empty space,
/// not an empty text widget, when neither condition applies — an empty
/// text widget still reserves a line of height.
pub fn status_line<Message: 'static>(
    had_errors: bool,
    ffmpeg_missing_with_videos: bool,
) -> Element<'static, Message> {
    let sentences = status_line_text(had_errors, ffmpeg_missing_with_videos);
    if sentences.is_empty() {
        container(space()).into()
    } else {
        text(sentences.join(" ")).into()
    }
}

/// RFC 036: which i18n key explains an empty, non-failed result —
/// distinguishes "nothing indexed yet" from "searched and found nothing".
/// Callers must only use this when `items.is_empty() && !had_errors`; a
/// read failure already has its own explanation via [`status_line`].
pub fn absence_message_key(nothing_indexed: bool) -> &'static str {
    if nothing_indexed {
        "similarity.nothing_indexed"
    } else {
        "similarity.no_results"
    }
}

/// Renders [`absence_message_key`] as a single element.
pub fn absence_message<Message: 'static>(nothing_indexed: bool) -> Element<'static, Message> {
    text(t(absence_message_key(nothing_indexed))).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_line_is_empty_when_neither_applies() {
        assert!(status_line_text(false, false).is_empty());
    }

    #[test]
    fn status_line_has_one_sentence_for_read_error_alone() {
        assert_eq!(status_line_text(true, false).len(), 1);
    }

    #[test]
    fn status_line_has_one_sentence_for_ffmpeg_missing_alone() {
        assert_eq!(status_line_text(false, true).len(), 1);
    }

    #[test]
    fn status_line_combines_both_as_exactly_two_sentences() {
        let sentences = status_line_text(true, true);
        assert_eq!(
            sentences.len(),
            2,
            "a failure and a missing toolchain together must be two sentences, not one merged mechanism"
        );
    }

    #[test]
    fn absence_message_key_distinguishes_not_indexed_from_no_results() {
        assert_eq!(absence_message_key(true), "similarity.nothing_indexed");
        assert_eq!(absence_message_key(false), "similarity.no_results");
    }
}
