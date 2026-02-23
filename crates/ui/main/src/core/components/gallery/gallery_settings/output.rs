use super::target_media_type::TargetMediaType;

#[derive(Debug, Clone)]
pub enum Output {
    TargetMediaTypeChange(TargetMediaType),
}
