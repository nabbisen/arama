#[derive(Debug, thiserror::Error)]
pub enum ReprError {
    #[error("blob length {0} is not a multiple of 4")]
    CodecInvalidLength(usize),
}
