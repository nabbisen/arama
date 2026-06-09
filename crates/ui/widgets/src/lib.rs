//! # arama-ui-widgets
//!
//! Self-contained reusable UI widgets for arama.
//!
//! - [`dir_tree::DirTree`] — interactive directory tree with per-node
//!   processing-state spinners; emits `DirClick` events.
//! - [`context_menu::ContextMenu`] — right-click menu for gallery cells
//!   (open, show in folder, trash).
//! - [`dialog::media_focus_dialog`] — modal that shows media similar to
//!   the currently focused file, sorted by cosine similarity.
//! - [`dialog::similar_pairs_dialog`] — modal that finds all
//!   near-duplicate pairs across the indexed directory.
//! - [`dialog::settings_dialog`] — tabbed settings panel (General, AI,
//!   File system, About); used as the Settings page in the side-nav shell.

pub mod context_menu;
pub mod dialog;
pub mod dir_tree;
mod similarity_badge;
pub use similarity_badge::view::similarity_badge;
