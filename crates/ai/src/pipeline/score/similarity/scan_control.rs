use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// Shared cancellation and progress state for an O(N^2) similarity scan
/// (RFC 047 S3.3; review 161 #2 and #3 of the first delivery).
///
/// Review 161 #2: `Task::abortable()` around a `tokio::task::spawn_blocking`
/// call has no effect on the blocking closure itself -
/// `tokio-1.54.1/src/task/blocking.rs:106-110` documents this explicitly.
/// So the only way to actually stop the O(N^2) work once it has started is
/// a flag the work itself checks, the same principle `run_thumbnail_phase`
/// already applies for the reason spawned work keeps running regardless of
/// whether the `Task` around it is still polled.
///
/// Review 161 #3: RFC 047 S6 ("reports progress") asks for more than S3.3's
/// own, weaker "reports that it is running" - a determinate count, not a
/// static label. `compared`/`total` are the same mechanism as the flag,
/// not a separate one: one instance per scan, checked and advanced from
/// inside the scoring loop that is being cancelled.
///
/// One instance covers a whole scan even when it spans more than one
/// call (images, then videos): `total` is set once, before scoring
/// starts; `compared` advances by one per outer row actually scored,
/// across however many calls contribute to it.
#[derive(Debug, Default)]
pub struct ScanControl {
    cancelled: AtomicBool,
    compared: AtomicUsize,
    total: AtomicUsize,
}

impl ScanControl {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set once, before scoring starts: the number of outer rows the
    /// whole scan will visit across every media type together.
    pub fn set_total(&self, total: usize) {
        self.total.store(total, Ordering::Relaxed);
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }

    /// Call once per outer row actually scored - not once per row
    /// skipped after cancellation, so `compared` never overstates how
    /// much real comparison work happened.
    pub fn advance(&self) {
        self.compared.fetch_add(1, Ordering::Relaxed);
    }

    /// `(compared, total)` as of now. A snapshot, not a stream - read it
    /// as often as is useful (a subscription tick, a test), never
    /// pushed.
    pub fn snapshot(&self) -> (usize, usize) {
        (
            self.compared.load(Ordering::Relaxed),
            self.total.load(Ordering::Relaxed),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::ScanControl;

    #[test]
    fn starts_at_zero_and_zero_and_not_cancelled() {
        let control = ScanControl::new();
        assert_eq!(control.snapshot(), (0, 0));
        assert!(!control.is_cancelled());
    }

    #[test]
    fn advance_and_set_total_are_independent_and_observable() {
        let control = ScanControl::new();
        control.set_total(42);
        control.advance();
        control.advance();
        assert_eq!(control.snapshot(), (2, 42));
    }

    #[test]
    fn cancel_is_observable_and_does_not_touch_progress() {
        let control = ScanControl::new();
        control.set_total(10);
        control.advance();
        control.cancel();
        assert!(control.is_cancelled());
        assert_eq!(control.snapshot(), (1, 10));
    }
}
