//! # arama-sidecar
//!
//! External ffmpeg toolchain integration for arama.
//!
//! Handles paired `ffmpeg` / `ffprobe` discovery, bounded validation, and
//! command creation. Acquisition is user-managed on every supported platform.
//! Discovery validates both tools as one compatible pair before returning an
//! [`FfmpegToolchain`].
//!
//! [`FfmpegToolchain`]: crate::media::video::video_engine::FfmpegToolchain

pub mod media;
