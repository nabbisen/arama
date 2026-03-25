#[derive(Debug, Clone)]
pub enum Message {
    SubDirDepthLimitChanged(u8),
    SimilarPairsOpen,
}
