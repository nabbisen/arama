use iced::Task;

use super::{SimilarPairsDialog, message::Message};

impl SimilarPairsDialog {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // The generation has already been checked by the caller
            // (`App::handle_similar_pairs_dialog_message`) before this
            // message reached `update` at all - a stale one never gets
            // here, so there is nothing left to do with it but ignore it.
            Message::EmbeddingsReady(_generation, result) => {
                self.has_read_error = result.outcome.had_errors;
                self.nothing_indexed = result.outcome.nothing_indexed;
                self.ffmpeg_missing_with_videos = result.outcome.ffmpeg_missing_with_videos;
                self.total_found = result.total_found;
                self.pairs = Some(result.outcome.items);
            }
            Message::Progress(compared, total) => self.scan_progress = (compared, total),
            Message::MediaItemEnter(path) => self.hovered_media_item_path_str = Some(path),
            Message::MediaItemDoubleClicked(_) => (),
            Message::MediaExit => self.hovered_media_item_path_str = None,
        }
        Task::none()
    }
}
