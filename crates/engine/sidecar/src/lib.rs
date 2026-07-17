//! # arama-sidecar
//!
//! ffmpeg binary management for arama.
//!
//! Handles paired `ffmpeg` / `ffprobe` discovery and command creation. Linux
//! and Windows may install digest-authenticated archives from the GitHub CDN
//! via `yt-dlp/FFmpeg-Builds`; macOS executable acquisition is user-managed.
//!
//! Managed binaries are stored in `.arama-local/bin` relative to the
//! application executable. Discovery validates both tools as one compatible
//! pair before returning an [`FfmpegToolchain`].

//! [`FfmpegToolchain`]: crate::media::video::video_engine::FfmpegToolchain

pub mod media;
