use super::hash_strategy::HashStrategy;

pub enum Mode {
    Full,
    Partial { partial_bytes: usize },
}

pub fn effective_mode(file_size: u64, strategy: &HashStrategy) -> Mode {
    match strategy {
        HashStrategy::Full => Mode::Full,
        HashStrategy::SizeAdaptive {
            threshold_bytes,
            partial_bytes,
        } => {
            if file_size < *threshold_bytes {
                Mode::Full
            } else {
                Mode::Partial {
                    partial_bytes: *partial_bytes,
                }
            }
        }
    }
}
