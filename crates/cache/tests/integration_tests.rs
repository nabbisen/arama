//! `arama-cache` integration tests.
//!
//! These tests define the facade's public contract. They were written
//! against the v1 (`file-feature-cache`) implementation and pass
//! unchanged — modulo the import below — against the v2 (`localcache`)
//! implementation, which is the API-compatibility proof for RFC 002.
//!
//! Test coverage is split across sibling files by namespace:
//! - `helpers`  — shared fixtures compiled into every test binary via `#[path]`
//! - `image`    — image-namespace tests
//! - `video`    — video-namespace tests
//! - `cross`    — cross-namespace / session / parallel / dir-summary tests
//!
//! Each sibling (`image.rs`, `video.rs`, `cross.rs`) is its own Cargo integration
//! test binary and includes `helpers.rs` via `#[path]`.
