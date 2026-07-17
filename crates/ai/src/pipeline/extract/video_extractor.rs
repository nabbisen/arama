use std::{path::Path, process::Stdio};

use anyhow::{Context, anyhow};
use arama_sidecar::media::video::video_engine::FfmpegToolchain;

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
    toolchain: FfmpegToolchain,
}

pub struct VideoFrameExtraction {
    pub frames: Vec<RawVideoFrame>,
    pub failures: usize,
}

pub struct AudioSegmentExtraction {
    pub segments: Vec<RawAudioSegment>,
    pub failures: usize,
}

impl VideoExtractor {
    pub fn new(cfg: VideoSimilarityConfig, toolchain: FfmpegToolchain) -> Self {
        Self { cfg, toolchain }
    }

    pub(crate) fn ffmpeg_path(&self) -> &Path {
        self.toolchain.ffmpeg_path()
    }

    // Video duration.

    /// Get video duration in seconds with ffprobe.
    ///
    /// Uses ffprobe from arama's validated paired toolchain.
    pub fn get_duration(&self, path: &Path) -> anyhow::Result<f64> {
        let output = self
            .toolchain
            .ffprobe()
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
    pub fn extract_video_frames_report(
        &self,
        path: &Path,
        timestamps: &[f64],
    ) -> VideoFrameExtraction {
        let size = self.cfg.clip_image_size;
        let mut frames = Vec::with_capacity(timestamps.len());
        let mut failures = 0;

        for &ts in timestamps {
            match self.seek_single_frame(path, ts, size) {
                Ok(Some(f)) => frames.push(f),
                Ok(None) => failures += 1,
                Err(_) => failures += 1,
            }
        }

        VideoFrameExtraction { frames, failures }
    }

    fn seek_single_frame(
        &self,
        path: &Path,
        timestamp: f64,
        size: usize,
    ) -> anyhow::Result<Option<RawVideoFrame>> {
        let scale = format!("{}:{}", size, size);

        let output = self
            .toolchain
            .ffmpeg()
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
    pub fn extract_audio_segments_direct_report(
        &self,
        path: &Path,
        start_times: &[f64],
        duration_secs: f64,
        sample_rate: u32,
    ) -> AudioSegmentExtraction {
        let mut segments = Vec::with_capacity(start_times.len());
        let mut failures = 0;
        for &start in start_times {
            match self.extract_one_audio_segment(path, start, duration_secs, sample_rate) {
                Ok(segment) if !segment.samples.is_empty() => segments.push(segment),
                Ok(_) => failures += 1,
                Err(_) => failures += 1,
            }
        }
        AudioSegmentExtraction { segments, failures }
    }

    fn extract_one_audio_segment(
        &self,
        path: &Path,
        start: f64,
        duration: f64,
        sample_rate: u32,
    ) -> anyhow::Result<RawAudioSegment> {
        let output = self
            .toolchain
            .ffmpeg()
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
