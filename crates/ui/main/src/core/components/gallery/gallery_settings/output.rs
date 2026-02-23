use super::media_type::MediaType;

#[derive(Debug, Clone)]
pub enum Output {
    MediaTypeChange(MediaType),
}
