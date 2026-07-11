use std::path::PathBuf;

use iced::Task;

use super::{
    CachePage,
    message::{Event, Internal, Message},
    parse_mib_target,
};

impl CachePage {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Event(_) => Task::none(),
            Message::Internal(message) => match message {
                Internal::FilterInput(s) => {
                    self.filter = s;
                    Task::none()
                }
                Internal::DirInput(s) => {
                    self.dir_input = s;
                    Task::none()
                }
                Internal::PruneTargetInput(s) => {
                    self.prune_target_input = s;
                    Task::none()
                }
                Internal::RefreshPressed => self.load_task(),
                Internal::CachePressed => {
                    let path = PathBuf::from(self.dir_input.trim());
                    // The app validates further; the page only emits the
                    // request for non-empty input.
                    if path.as_os_str().is_empty() {
                        Task::none()
                    } else {
                        Task::done(Message::Event(Event::CacheRequest(path)))
                    }
                }
                Internal::PrunePressed => match parse_mib_target(&self.prune_target_input) {
                    Some(max_bytes) => {
                        self.prune_busy = true;
                        Task::done(Message::Event(Event::PruneRequest(max_bytes)))
                    }
                    None => Task::none(),
                },
                Internal::RowsLoaded(Ok(load)) => {
                    self.rows = load.rows;
                    self.footprint = load.footprint;
                    self.load_error = None;
                    self.busy = false;
                    self.loaded = true;
                    Task::none()
                }
                Internal::RowsLoaded(Err(err)) => {
                    self.load_error = Some(err.message);
                    self.busy = false;
                    self.loaded = true;
                    Task::none()
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::views::cache_page::{CacheLoad, CacheLoadError, DirRow};

    #[test]
    fn rows_loaded_success_clears_prior_error() {
        let mut page = CachePage {
            load_error: Some("old error".to_owned()),
            busy: true,
            ..CachePage::default()
        };

        let load = CacheLoad {
            rows: vec![DirRow {
                dir_path: "/tmp/images".to_owned(),
                file_count: 2,
                total_size: 128,
                latest_cached_at: 42,
            }],
            footprint: None,
        };

        let _ = page.update(Message::Internal(Internal::RowsLoaded(Ok(load))));

        assert!(page.load_error.is_none());
        assert!(!page.busy);
        assert!(page.loaded);
        assert_eq!(page.rows.len(), 1);
    }

    #[test]
    fn rows_loaded_failure_preserves_stale_rows_and_sets_error() {
        let stale = DirRow {
            dir_path: "/tmp/stale".to_owned(),
            file_count: 1,
            total_size: 64,
            latest_cached_at: 7,
        };
        let mut page = CachePage {
            rows: vec![stale.clone()],
            busy: true,
            loaded: true,
            ..CachePage::default()
        };

        let _ = page.update(Message::Internal(Internal::RowsLoaded(Err(
            CacheLoadError {
                message: "cache unavailable".to_owned(),
            },
        ))));

        assert_eq!(page.load_error.as_deref(), Some("cache unavailable"));
        assert!(!page.busy);
        assert!(page.loaded);
        assert_eq!(page.rows.len(), 1);
        assert_eq!(page.rows[0].dir_path, stale.dir_path);
    }
}
