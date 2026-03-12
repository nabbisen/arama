use arama_ai::model::model_container::{clip, wav2vec2};
use arama_sidecar::media::video::video_engine::{FfmpegStatus, VideoEngine};

pub fn ready() -> bool {
    clip::model().ready().is_ok_and(|x| x)
        && wav2vec2::model().ready().is_ok_and(|x| x)
        && VideoEngine::ready() != FfmpegStatus::NotExists
}
