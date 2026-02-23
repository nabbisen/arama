#[derive(Clone, Debug)]
pub struct MediaType {
    pub include_image: bool,
    pub include_video: bool,
}

impl Default for MediaType {
    fn default() -> Self {
        Self {
            include_image: true,
            include_video: false,
        }
    }
}
