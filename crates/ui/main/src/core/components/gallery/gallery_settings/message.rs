use arama_env::target_media_type::TargetMediaType;

#[derive(Debug, Clone)]
pub enum Message {
    TargetMediaTypeChanged(TargetMediaType),
    SubDirDepthLimitChanged(u8),
    SimilarPairsOpen,
}
