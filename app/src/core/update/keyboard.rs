use iced::Task;
use iced::keyboard::{Key, Modifiers};
use snora_core::focus::{Cycle, FocusZone};

use super::super::{App, message::Message};

/// RFC 044 §2.2: the zone `F6` / `Shift+F6` should move focus to, given
/// arama's own static [`App::zone_presence`] and the two overlay flags -
/// pure, so Tier 1 can assert wrapping, slot-skipping and modal
/// suspension without constructing an `App`.
pub(crate) fn zone_after_cycle(
    current: FocusZone,
    cycle: Cycle,
    has_modal: bool,
    has_menu: bool,
) -> Option<FocusZone> {
    snora_core::focus::next_zone(current, cycle, App::zone_presence(), has_modal, has_menu)
}

impl App {
    /// RFC 044 §2.1/§2.2: routes a key `iced::keyboard::listen()` reports
    /// as unclaimed (Phase 0.1 found every key arama cares about is
    /// unclaimed today) through snora's pure helpers, modal-dismissal
    /// first - the same precedence order `next_zone` enforces internally
    /// for cycling, so a key that could mean either never does both.
    pub(super) fn handle_key_pressed(&mut self, key: Key, modifiers: Modifiers) -> Task<Message> {
        let has_modal = self.dialog.is_some();
        let has_menu = self.context_menu.is_open();

        if let Some(message) = snora::keyboard::dismiss_on_escape(
            has_modal,
            has_menu,
            Some(Message::DialogClose),
            Some(Message::CloseMenus),
            key.clone(),
        ) {
            return self.update(message);
        }

        if let Some(cycle) = snora::keyboard::cycle_zones(key, modifiers)
            && let Some(zone) = zone_after_cycle(self.focus_zone, cycle, has_modal, has_menu)
        {
            self.focus_zone = zone;
            // First real cycle: start rendering the ring. Never reset -
            // once a keyboard user exists, hiding the ring again on a
            // later mouse click would be its own invisible-focus defect.
            self.focus_visible = true;
        }

        Task::none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zone_presence_matches_aramas_static_layout() {
        // Header lives inside `body` (view.rs) - never its own slot.
        // SideBar and Footer are always populated.
        assert_eq!(
            App::zone_presence(),
            snora_core::focus::ZonePresence::none()
                .side_bar(true)
                .footer(true)
        );
    }

    #[test]
    fn f6_cycles_forward_through_aramas_three_zones() {
        assert_eq!(
            zone_after_cycle(FocusZone::SideBar, Cycle::Forward, false, false),
            Some(FocusZone::Body)
        );
        assert_eq!(
            zone_after_cycle(FocusZone::Body, Cycle::Forward, false, false),
            Some(FocusZone::Footer)
        );
        assert_eq!(
            zone_after_cycle(FocusZone::Footer, Cycle::Forward, false, false),
            Some(FocusZone::SideBar),
            "must wrap past the absent Header slot straight back to SideBar"
        );
    }

    #[test]
    fn shift_f6_cycles_backward() {
        assert_eq!(
            zone_after_cycle(FocusZone::Body, Cycle::Backward, false, false),
            Some(FocusZone::SideBar)
        );
        assert_eq!(
            zone_after_cycle(FocusZone::SideBar, Cycle::Backward, false, false),
            Some(FocusZone::Footer),
            "must wrap past the absent Header slot straight back to Footer"
        );
    }

    #[test]
    fn a_modal_suspends_cycling_regardless_of_menu() {
        assert_eq!(
            zone_after_cycle(FocusZone::Body, Cycle::Forward, true, false),
            None
        );
        assert_eq!(
            zone_after_cycle(FocusZone::Body, Cycle::Forward, true, true),
            None,
            "modal takes priority even with a menu also open"
        );
    }

    #[test]
    fn a_menu_alone_does_not_suspend_cycling() {
        assert_eq!(
            zone_after_cycle(FocusZone::Body, Cycle::Forward, false, true),
            Some(FocusZone::Footer)
        );
    }

    #[test]
    fn header_never_appears_because_it_is_never_present() {
        for start in [FocusZone::SideBar, FocusZone::Body, FocusZone::Footer] {
            for cycle in [Cycle::Forward, Cycle::Backward] {
                let next = zone_after_cycle(start, cycle, false, false)
                    .expect("Body is always present, so a next zone always exists");
                assert_ne!(
                    next,
                    FocusZone::Header,
                    "F6 must never stop on Header - it lives inside Body by construction"
                );
            }
        }
    }

    #[test]
    fn escape_dismisses_a_modal_before_a_menu() {
        let result = snora::keyboard::dismiss_on_escape(
            true,
            true,
            Some(Message::DialogClose),
            Some(Message::CloseMenus),
            Key::Named(iced::keyboard::key::Named::Escape),
        );
        assert!(matches!(result, Some(Message::DialogClose)));
    }

    #[test]
    fn escape_dismisses_a_menu_when_no_modal_is_open() {
        let result = snora::keyboard::dismiss_on_escape(
            false,
            true,
            Some(Message::DialogClose),
            Some(Message::CloseMenus),
            Key::Named(iced::keyboard::key::Named::Escape),
        );
        assert!(matches!(result, Some(Message::CloseMenus)));
    }

    #[test]
    fn escape_does_nothing_with_no_overlay_open() {
        let result = snora::keyboard::dismiss_on_escape(
            false,
            false,
            Some(Message::DialogClose),
            Some(Message::CloseMenus),
            Key::Named(iced::keyboard::key::Named::Escape),
        );
        assert!(result.is_none());
    }

    #[test]
    fn a_non_escape_key_never_dismisses_anything() {
        let result = snora::keyboard::dismiss_on_escape(
            true,
            true,
            Some(Message::DialogClose),
            Some(Message::CloseMenus),
            Key::Named(iced::keyboard::key::Named::Enter),
        );
        assert!(result.is_none());
    }
}
