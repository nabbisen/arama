use std::time::Duration;

use iced::Subscription;

use super::{App, message::Message};

/// RFC 047 S6 / review 161 #3: how often the similar-pairs dialog's
/// progress line is refreshed while a scan is running. A snapshot read,
/// not a push - cheap enough at this interval that the exact value is
/// not load-bearing, just frequent enough to read as "live" rather than
/// stuck.
const SIMILAR_PAIRS_PROGRESS_INTERVAL: Duration = Duration::from_millis(150);

impl App {
    pub fn subscription(&self) -> Subscription<Message> {
        // RFC 044: `iced::keyboard::listen()` only fires for keys no
        // widget already claimed (it filters on
        // `event::Status::Ignored`) - Phase 0.1 confirmed every key arama
        // cares about (Escape, F6, Tab) is ignored today, so this is the
        // one subscription that reaches all of them.
        let key_sub = iced::keyboard::listen().map(|event| match event {
            iced::keyboard::Event::KeyPressed { key, modifiers, .. } => {
                Message::KeyPressed(key, modifiers)
            }
            _ => Message::NoOp,
        });

        // RFC 047 S6 / review 161 #3: only present while a scan is
        // actually in flight. The `.map` closure must not capture
        // anything (`Subscription::map`'s own requirement - confirmed
        // by hitting its const-eval panic when it first did), so this
        // emits a bare tick; `handle_similar_pairs_progress_tick` is
        // where the live `ScanControl` snapshot is actually read, with
        // `&self` access `subscription()` does not need here. The same
        // pattern `snora::toast::subscription` already uses for
        // `ToastSweep` reading `self.toasts` inside `update` instead.
        let similar_pairs_progress_sub = if self.similar_pairs_scan.is_some() {
            iced::time::every(SIMILAR_PAIRS_PROGRESS_INTERVAL)
                .map(|_| Message::SimilarPairsProgressTick)
        } else {
            Subscription::none()
        };

        Subscription::batch([
            snora::toast::subscription(&self.toasts, || Message::ToastSweep),
            key_sub,
            similar_pairs_progress_sub,
        ])
    }
}
