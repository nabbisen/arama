/// 音声セグメントの生データ（f32LE モノラル PCM）
pub struct RawAudioSegment {
    pub start_secs: f64,
    pub sample_rate: u32,
    pub samples: Vec<f32>,
}

/// AudioEncoder に渡すゼロコピービュー
pub struct AudioSegmentView<'a> {
    pub start_secs: f64,
    pub sample_rate: u32,
    pub samples: &'a [f32],
}
