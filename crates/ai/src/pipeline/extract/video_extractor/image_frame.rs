/// 映像フレームの生データ（RGB24 HWC）
pub struct RawVideoFrame {
    pub timestamp_secs: f64,
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}
