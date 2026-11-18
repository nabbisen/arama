use iced::Task;
use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};
use snora::focus::Cycle;

use super::super::{App, message::Message};

/// What a key press means, independent of any widget tree (RFC 054 §3.1).
/// Exists only because `snora::keyboard::traverse`/`focus_zone` return an
/// opaque `Task`, so a routing decision cannot be asserted by calling them
/// directly - this is the pure half, tested without constructing an `App`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum KeyRoute {
    /// Tab / Shift+Tab: move between the controls that can take focus.
    Traverse,
    /// F6 / Shift+F6: move focus into the next/previous region.
    Zone(Cycle),
}

/// Plain Tab or Shift+Tab only - not `Ctrl+Tab`, `Alt+Tab` or `Logo+Tab`,
/// which mean other things (switching tabs, switching windows) and must
/// fall through rather than move focus. Mirrors the gate
/// `snora::keyboard::traverse` applies internally; duplicated here only
/// because that gate is not otherwise observable (`traverse` returns an
/// opaque `Task`, not a reason).
fn is_plain_tab(key: &Key, modifiers: Modifiers) -> bool {
    *key == Key::Named(Named::Tab) && !modifiers.control() && !modifiers.alt() && !modifiers.logo()
}

/// Tab is tested first, then F6 (RFC 054 §3.1, §6.2) - the two never
/// overlap, but the order matches the handoff's own description and
/// `handle_key_pressed`'s dispatch below.
pub(crate) fn route_key(key: &Key, modifiers: Modifiers) -> Option<KeyRoute> {
    if is_plain_tab(key, modifiers) {
        return Some(KeyRoute::Traverse);
    }
    snora::keyboard::cycle_zones(key.clone(), modifiers).map(KeyRoute::Zone)
}

impl App {
    /// RFC 054 §3.1: routes Tab/Shift+Tab to `snora::keyboard::traverse`
    /// and F6/Shift+F6 to `snora::keyboard::focus_zone`. arama keeps no
    /// zone state and passes no modal/menu flags: snora already contains
    /// focus to an open dialog, sheet or menu-with-backdrop on its own
    /// (RFC 054 §2, "Containment in a dialog"), so there is nothing
    /// special to do here while one is open. Every other key is
    /// unclaimed and returns `Task::none()`, as before.
    pub(super) fn handle_key_pressed(&mut self, key: Key, modifiers: Modifiers) -> Task<Message> {
        match route_key(&key, modifiers) {
            Some(KeyRoute::Traverse) => {
                snora::keyboard::traverse(key, modifiers).unwrap_or_else(Task::none)
            }
            Some(KeyRoute::Zone(cycle)) => snora::keyboard::focus_zone(cycle),
            None => Task::none(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TAB: Key = Key::Named(Named::Tab);
    const F6: Key = Key::Named(Named::F6);

    #[test]
    fn plain_tab_routes_to_traverse() {
        assert_eq!(
            route_key(&TAB, Modifiers::empty()),
            Some(KeyRoute::Traverse)
        );
    }

    #[test]
    fn shift_tab_also_routes_to_traverse() {
        assert_eq!(route_key(&TAB, Modifiers::SHIFT), Some(KeyRoute::Traverse));
    }

    #[test]
    fn ctrl_tab_alt_tab_and_logo_tab_are_not_tab() {
        for modifiers in [Modifiers::CTRL, Modifiers::ALT, Modifiers::LOGO] {
            assert_eq!(
                route_key(&TAB, modifiers),
                None,
                "Tab with {modifiers:?} must not route to Traverse"
            );
        }
    }

    #[test]
    fn plain_f6_routes_to_zone_forward() {
        assert_eq!(
            route_key(&F6, Modifiers::empty()),
            Some(KeyRoute::Zone(Cycle::Forward))
        );
    }

    #[test]
    fn shift_f6_routes_to_zone_backward() {
        assert_eq!(
            route_key(&F6, Modifiers::SHIFT),
            Some(KeyRoute::Zone(Cycle::Backward))
        );
    }

    #[test]
    fn other_keys_route_nowhere() {
        assert_eq!(
            route_key(&Key::Named(Named::Enter), Modifiers::empty()),
            None
        );
        assert_eq!(
            route_key(&Key::Named(Named::Escape), Modifiers::empty()),
            None
        );
        assert_eq!(
            route_key(&Key::Character("a".into()), Modifiers::empty()),
            None
        );
    }
}
