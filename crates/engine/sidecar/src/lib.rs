//! # arama-sidecar
//!
//! ffmpeg binary management for arama.
//!
//! Handles download URL selection (GitHub CDN via `yt-dlp/FFmpeg-Builds`
//! for Linux and Windows; evermeet.cx / osxexperts.net for macOS),
//! archive extraction, and spawning `ffmpeg` / `ffprobe` processes.
//!
//! The binary is stored at `.arama-local/bin/ffmpeg[.exe]` relative to
//! the application executable. [`VideoEngine::ready`] checks for the
//! binary both in the local directory and on `PATH`.

pub mod media;
