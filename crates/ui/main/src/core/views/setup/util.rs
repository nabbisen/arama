pub(crate) fn resources_ready(clip_ready: bool) -> bool {
    clip_ready
}

#[cfg(test)]
mod tests {
    use super::resources_ready;

    #[test]
    fn setup_readiness_requires_clip() {
        assert!(resources_ready(true));
        assert!(!resources_ready(false));
    }
}
