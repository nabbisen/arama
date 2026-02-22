use crate::aside::dir_tree;

use super::{Aside, message::Message, output::Output};

impl Aside {
    pub fn update(&mut self, message: Message) -> Option<Output> {
        match message {
            Message::DirTreeMessage(message) => {
                let output = self.dir_tree.update(message.clone());

                match output {
                    Some(dir_tree::output::Output::DirClick(path)) => {
                        return Some(Output::DirClick(path));
                    }
                    _ => (),
                }
            }
        }
        None
    }
}
