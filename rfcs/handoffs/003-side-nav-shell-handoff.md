# Handoff — RFC 003: Side-nav shell redesign

**RFC.** [`rfcs/done/003-side-nav-shell.md`](../done/003-side-nav-shell.md)
**Shipped in.** v0.24.0
**Depends on.** RFC 001 (snora `AppLayout`)

---

## 1. Implementation Handoff

### Goal
Replace the header-mounted Settings button and the collapsible aside toggle
with a snora `side_bar` nav rail hosting named pages. Ships with **Explorer**
and **Settings**; designed so RFC 004's Cache page slots in as a third item
with no structural change.

### Design
- New `NavPage` enum in `app/src/core/` (initially `Explorer | Settings`,
  later `+ Cache`).
- The `AppLayout.side_bar` slot receives a vertical column of icon buttons
  (one per page), built from Lucide icons already in the tree, each wrapped
  in a tooltip. Active page → primary style, others → text/ghost. The snora
  `SideBar` data types are not required — `AppLayout` accepts any `Element`.
- Settings stops being a modal dialog opened from the header; it becomes a
  full page selected from the rail.
- The header loses its Settings button; the aside loses its collapse toggle.

### Per-page body
The app `view()` switches on `NavPage`: Explorer renders
`column![header, row![aside, gallery]]`; Settings renders the settings page
body. (Cache is added later by RFC 004.)

---

## 2. Task Breakdown / PR Plan

Single cohesive PR (shell restructure), or split:

### PR 1 — NavPage + rail
1. `NavPage` enum, `nav_page` app state, `NavTo` message.
2. Side-bar rail of tooltip'd icon buttons in the `AppLayout.side_bar` slot.
3. Per-`NavPage` body switch in `view()`.

### PR 2 — Remove the old entry points
4. Remove the header Settings button and the aside collapse toggle and their
   messages.

---

## 3. Acceptance / QA Checklist

### Automated
- [ ] `cargo check --workspace` — zero errors, zero warnings (watch for
      now-unused header/aside messages).

### Manual
- [ ] The side-nav rail shows one icon button per page; hovering shows a
      tooltip; the active page's button is highlighted (primary).
- [ ] Clicking a rail item switches the page body.
- [ ] Explorer page shows header + directory tree + gallery as before.
- [ ] Settings is now a full page (no longer a modal), reachable from the
      rail.

### Regression
- [ ] The header no longer has a Settings button; the aside no longer has a
      collapse toggle.
- [ ] Directory navigation, gallery, and dialogs continue to work within the
      Explorer page.
