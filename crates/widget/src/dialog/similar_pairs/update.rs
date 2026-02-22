use super::{SimilarPairs, message::Message, output::Output};

impl SimilarPairs {
    pub fn update(&mut self, message: Message) -> Option<Output> {
        match message {
            Message::EmbeddingsReady(pairs) => return Some(Output::EmbeddingsReady(pairs)),
        }
    }
}
