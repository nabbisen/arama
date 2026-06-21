use std::path::PathBuf;

use iced::Task;
use iced::widget::Id;
use iced::widget::operation::{RelativeOffset, snap_to};
use iced_swdir_tree::{DirectoryFilter, DirectoryTree, DirectoryTreeEvent, SelectionMode};

use message::{Internal, Message};

pub mod message;
mod update;
pub mod view;

/// Stable Id for the aside's outer scrollable — used to snap the
/// viewport to the selected directory after the expand cascade completes.
pub(crate) const SCROLLABLE_ID: &str = "aside-tree-scroll";

pub struct Aside {
    pub(crate) tree: DirectoryTree,
    pub(crate) processing: bool,
    /// Remaining ancestor paths to expand, outermost-first.
    /// Populated by `expand_to`; drained one entry per `Loaded` event.
    pub(crate) expand_queue: Vec<PathBuf>,
    /// The directory the current cascade is targeting, stored so
    /// `finish_expand` can select and scroll to it when the queue empties.
    pub(crate) expand_target: Option<PathBuf>,
}

impl Aside {
    /// Create a new `Aside` rooted at the filesystem root so all
    /// parent directories are always visible.
    pub fn new(processing: bool) -> Self {
        let root = filesystem_root();
        let tree = DirectoryTree::new(root).with_filter(DirectoryFilter::FoldersOnly);
        Self {
            tree,
            processing,
            expand_queue: Vec::new(),
            expand_target: None,
        }
    }

    /// Expand the tree from the root down to `target`, one async level
    /// at a time, then select and scroll to it.
    ///
    /// Issues `Toggled(root)` immediately; each `Loaded` event advances
    /// the cascade one level; when the queue is empty `ExpandDone` fires,
    /// selecting the target row and snapping the scroll to it.
    pub fn expand_to(&mut self, target: &PathBuf) -> Task<Message> {
        self.expand_queue.clear();
        self.expand_target = Some(target.clone());

        let root = self.tree.root_path().to_path_buf();

        let Ok(rel) = target.strip_prefix(&root) else {
            return Task::none();
        };

        // Build the outermost-first list of ancestor paths:
        // [root/a, root/a/b, …, target]
        // `rel.ancestors()` yields deepest-first relative suffixes;
        // filter the empty one, collect, then reverse.
        let mut ancestors: Vec<PathBuf> = rel
            .ancestors()
            .filter(|s| !s.as_os_str().is_empty())
            .map(|suffix| root.join(suffix))
            .collect();
        ancestors.reverse();

        self.expand_queue = ancestors;

        self.tree
            .update(DirectoryTreeEvent::Toggled(root))
            .map(|e| Message::Internal(Internal::TreeEvent(e)))
    }

    /// Pop and issue the next queued `Toggled`. When the queue empties,
    /// emit `ExpandDone` so the handler can select + scroll to the target.
    pub fn advance_expand(&mut self) -> Task<Message> {
        if let Some(next) = self.expand_queue.first().cloned() {
            self.expand_queue.remove(0);
            return self
                .tree
                .update(DirectoryTreeEvent::Toggled(next))
                .map(|e| Message::Internal(Internal::TreeEvent(e)));
        }
        Task::done(Message::Internal(Internal::ExpandDone))
    }

    /// Called on `ExpandDone`: select the target row and snap the outer
    /// scrollable's viewport to the end so the selected row is visible.
    pub fn finish_expand(&mut self) -> Task<Message> {
        if let Some(target) = self.expand_target.take() {
            let _ = self.tree.update(DirectoryTreeEvent::Selected(
                target,
                true,
                SelectionMode::Replace,
            ));
            let snap: Task<Message> = snap_to(Id::new(SCROLLABLE_ID), RelativeOffset::END);
            return snap;
        }
        Task::none()
    }

    /// Rebuild the tree (re-rooted at the filesystem root) and expand
    /// to `path`. Called when the user navigates via header or file-picker.
    pub fn update_dir_tree(&mut self, path: &PathBuf) -> Task<Message> {
        let root = filesystem_root();
        self.tree = DirectoryTree::new(root).with_filter(DirectoryFilter::FoldersOnly);
        self.expand_to(path)
    }

    pub fn set_processing(&mut self, processing: bool) {
        self.processing = processing;
    }
}

/// Returns the filesystem root: `/` on Unix, `C:\` on Windows.
fn filesystem_root() -> PathBuf {
    #[cfg(unix)]
    {
        PathBuf::from("/")
    }
    #[cfg(windows)]
    {
        PathBuf::from("C:\\")
    }
}
