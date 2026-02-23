use super::{SimilarPairsDialog, message::Message, output::Output};

impl SimilarPairsDialog {
    pub fn update(&mut self, message: Message) -> Option<Output> {
        match message {
            Message::EmbeddingsReady(pairs) => return Some(Output::EmbeddingsReady(pairs)),
        }
    }
}
