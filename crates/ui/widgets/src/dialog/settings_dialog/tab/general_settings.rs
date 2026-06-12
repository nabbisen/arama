use arama_env::target_media_type::TargetMediaType;

pub mod message;
mod update;
mod view;

#[derive(Clone, Debug)]
pub struct GeneralSettings {
    target_media_type: TargetMediaType,
    sub_dir_depth_limit: u8,
    similarity_threshold: f32,
}

impl GeneralSettings {
    pub fn new(
        target_media_type: &TargetMediaType,
        sub_dir_depth_limit: u8,
        similarity_threshold: f32,
    ) -> Self {
        Self {
            target_media_type: target_media_type.to_owned(),
            sub_dir_depth_limit,
            similarity_threshold,
        }
    }
}
