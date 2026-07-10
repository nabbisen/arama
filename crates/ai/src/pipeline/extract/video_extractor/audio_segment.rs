/// Raw audio segment data in f32LE mono PCM.
pub struct RawAudioSegment {
    pub start_secs: f64,
    pub sample_rate: u32,
    pub samples: Vec<f32>,
}

/// Zero-copy view passed to `AudioEncoder`.
pub struct AudioSegmentView<'a> {
    pub start_secs: f64,
    pub sample_rate: u32,
    pub samples: &'a [f32],
}
