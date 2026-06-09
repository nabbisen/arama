//! # arama-ui-layout
//!
//! Application shell layout components for arama.
//!
//! - [`aside::Aside`] — directory tree panel (always visible in the
//!   Explorer page tiling view).
//! - [`header::Header`] — top bar with directory path input and the
//!   similarity-pairs action button.
//! - [`footer::Footer`] — status bar with file/directory counts and
//!   the thumbnail-size slider.
//!
//! These components have no dependency on the AI or cache crates. They
//! emit typed [`message`](aside::message) events that `app` maps onto
//! top-level [`Message`](../app/src/core/message) variants.

pub mod aside;
pub mod footer;
pub mod header;
