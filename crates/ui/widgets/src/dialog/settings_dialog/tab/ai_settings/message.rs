#[derive(Debug, Clone)]
pub enum Message {
    LoadStart,
    Loaded(Option<String>),
    RefreshCapabilities,
    GetWav2vec2Start,
    Wav2vec2Got(u64, Result<(), String>),
    CheckFfmpeg,
    FfmpegRecheckRequested,
    SelectFfmpegDirectory,
    ClearFfmpegSelection,
    FfmpegChecked(bool),
    GetFfmpegStart,
    FfmpegGot(Option<String>),
}
