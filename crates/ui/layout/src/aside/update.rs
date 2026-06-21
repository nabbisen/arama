use iced::Task;
use iced_swdir_tree::DirectoryTreeEvent;

use super::{
    Aside,
    message::{Event, Internal, Message},
};

impl Aside {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Event(_) => Task::none(),

            Message::Internal(Internal::ExpandDone) => self.finish_expand(),

            Message::Internal(Internal::TreeEvent(event)) => {
                // While indexing is active, let in-flight scan results and
                // drag-state transitions complete so the tree stays consistent,
                // but block user-initiated navigation (folder expand, selection).
                if self.processing {
                    return match &event {
                        DirectoryTreeEvent::Loaded(_) | DirectoryTreeEvent::Drag(_) => self
                            .tree
                            .update(event)
                            .map(|e| Message::Internal(Internal::TreeEvent(e))),
                        _ => Task::none(),
                    };
                }

                // When an async scan completes, advance the expand cascade
                // (if one is in progress).
                if let DirectoryTreeEvent::Loaded(_) = &event {
                    let widget_task = self
                        .tree
                        .update(event)
                        .map(|e| Message::Internal(Internal::TreeEvent(e)));
                    let expand_task = self.advance_expand();
                    return Task::batch([widget_task, expand_task]);
                }

                // Emit DirSelect when the user selects a directory row.
                if let DirectoryTreeEvent::Selected(path, true, _) = &event {
                    let path = path.clone();
                    let scan_task = self
                        .tree
                        .update(event)
                        .map(|e| Message::Internal(Internal::TreeEvent(e)));
                    return Task::batch([
                        scan_task,
                        Task::done(Message::Event(Event::DirSelect(path))),
                    ]);
                }

                // All other variants (Toggled, Drag, DragCompleted,
                // Selected-file) route straight through to the widget.
                self.tree
                    .update(event)
                    .map(|e| Message::Internal(Internal::TreeEvent(e)))
            }
        }
    }
}
