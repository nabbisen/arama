/// Raw video frame data in RGB24 HWC layout.
pub struct RawVideoFrame {
    pub timestamp_secs: f64,
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}
