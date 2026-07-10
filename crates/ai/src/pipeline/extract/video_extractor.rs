use std::{path::Path, process::Stdio};

use anyhow::{Context, anyhow};
use arama_sidecar::media::video::video_engine::VideoEngine;

use crate::{
    config::video_similarity_config::VideoSimilarityConfig,
    pipeline::extract::video_extractor::{
        audio_segment::RawAudioSegment, image_frame::RawVideoFrame,
    },
};

pub mod audio_segment;
pub mod image_frame;

pub struct VideoExtractor {
    cfg: VideoSimilarityConfig,
}

impl VideoExtractor {
    pub fn new(cfg: VideoSimilarityConfig) -> Self {
        Self { cfg }
    }

    // Video duration.

    /// Get video duration in seconds with ffprobe.
    ///
    /// Uses the ffprobe binary managed by arama, so no separate installation is required.
    pub fn get_duration(&self, path: &Path) -> anyhow::Result<f64> {
        let output = VideoEngine::ffprobe()
            .with_context(|| "failed to get ffprobe")?
            .args([
                "-v",
                "error",
                "-show_entries",
                "format=duration",
                "-of",
                "default=noprint_wrappers=1:nokey=1",
                path.to_string_lossy().as_ref(),
            ])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let duration: f64 = stdout
            .trim()
            .parse()
            .with_context(|| format!("Failed to parse duration: '{}'", stdout.trim()))?;

        Ok(duration)
    }

    // Video frames.

    /// Extract frames at the requested timestamps using individual seeks.
    ///
    /// With input-side `-ss`, ffmpeg performs GOP-level fast seeks, so decode
    /// cost is roughly constant regardless of whether the timestamp is near the
    /// beginning or middle of a long video.
    pub fn extract_video_frames(
        &self,
        path: &Path,
        timestamps: &[f64],
    ) -> anyhow::Result<Vec<RawVideoFrame>> {
        let size = self.cfg.clip_image_size;
        let mut frames = Vec::with_capacity(timestamps.len());

        for &ts in timestamps {
            match self.seek_single_frame(path, ts, size) {
                Ok(Some(f)) => frames.push(f),
                Ok(None) => eprintln!("arama: no frame at t={:.1}s in {:?}", ts, path),
                Err(e) => eprintln!("arama: frame extract error at t={:.1}s: {}", ts, e),
            }
        }

        Ok(frames)
    }

    fn seek_single_frame(
        &self,
        path: &Path,
        timestamp: f64,
        size: usize,
    ) -> anyhow::Result<Option<RawVideoFrame>> {
        let scale = format!("{}:{}", size, size);

        let output = VideoEngine::ffmpeg()
            .with_context(|| "failed to get ffmpeg")?
            .args([
                "-ss",
                &timestamp.to_string(),
                "-i",
                path.to_string_lossy().as_ref(),
                "-vframes",
                "1",
                "-vf",
                &format!("scale={}", scale),
                "-f",
                "rawvideo",
                "-pix_fmt",
                "rgb24",
                "pipe:1",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .context("ffmpeg failed")?;

        if !output.status.success() {
            return Err(anyhow!(format!(
                "ffmepg failed to seek single frame: {:?}",
                output
            )));
        }

        let data: Vec<u8> = output.stdout;

        if data.is_empty() {
            return Ok(None);
        }

        Ok(Some(RawVideoFrame {
            timestamp_secs: timestamp,
            width: size as u32,
            height: size as u32,
            data,
        }))
    }

    // Audio segments.

    /// Extract audio segments by seeking directly to each timestamp.
    ///
    /// This decodes only the needed seconds. It starts ffmpeg once per segment,
    /// but each invocation decodes a small window, which is cheaper than
    /// decoding every window in one large pass.
    pub fn extract_audio_segments_direct(
        &self,
        path: &Path,
        start_times: &[f64],
        duration_secs: f64,
        sample_rate: u32,
    ) -> anyhow::Result<Vec<RawAudioSegment>> {
        start_times
            .iter()
            .map(|&start| self.extract_one_audio_segment(path, start, duration_secs, sample_rate))
            .collect()
    }

    fn extract_one_audio_segment(
        &self,
        path: &Path,
        start: f64,
        duration: f64,
        sample_rate: u32,
    ) -> anyhow::Result<RawAudioSegment> {
        let output = VideoEngine::ffmpeg()
            .with_context(|| "failed to get ffmpeg")?
            .args([
                "-ss",
                &start.to_string(),
                "-i",
                path.to_string_lossy().as_ref(),
                "-t",
                &duration.to_string(),
                "-vn", // Ignore the video track.
                "-acodec",
                "pcm_f32le", // Convert directly to f32LE PCM.
                "-ar",
                &sample_rate.to_string(),
                "-ac",
                "1", // Mono.
                "-f",
                "f32le",
                "pipe:1",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .context("ffmpeg failed")?;

        if !output.status.success() {
            return Err(anyhow!(format!(
                "ffmepg failed to extract one audio segment: {:?}",
                output
            )));
        }

        let data: Vec<u8> = output.stdout;

        if data.is_empty() {
            return Ok(RawAudioSegment {
                start_secs: 0.0,
                sample_rate: 0,
                samples: vec![],
            });
        }

        let samples: Vec<f32> = data
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes(b.try_into().unwrap()))
            .collect();

        Ok(RawAudioSegment {
            start_secs: start,
            sample_rate,
            samples,
        })
    }
}
