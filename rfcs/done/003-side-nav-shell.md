# RFC 003 — Side-nav shell redesign

**Status.** Implemented (v0.24.0)
**Tracks.** Top-level navigation structure. Replaces the
header-mounted settings button and the collapsible aside rail with
a snora side-nav shell that hosts two named pages.
**Touches.** `app/src/core/` (new `NavPage` enum, view/update/message),
`crates/ui/layout/src/header/` (settings button removed),
`crates/ui/layout/src/aside/` (toggle removed), `rfcs/README.md`,
`CHANGELOG.md`.

## Summary

The current shell places the directory-nav header, the collapsible
aside (which carries the directory tree), and the gallery side-by-side.
Settings are behind a modal dialog opened from a header button. This
arrangement does not scale: a third navigation destination (the planned
Cache control page) would require another modal, another header button,
or an ad-hoc workaround.

This RFC replaces the arrangement with a snora `side_bar` nav rail and
two named pages — **Explorer** and **Settings** — built on the same
`AppLayout` skeleton already in use.

The Cache control page (RFC tbd) will slot in as a third nav item with
no further structural change.

## Design

### Navigation rail

The `AppLayout.side_bar` slot receives a vertical column of icon
buttons — one per page — built from the Lucide icons already in the
dependency tree. The column is a plain iced element; the snora
`SideBar` / `SideBarItem` data types are not required because
`AppLayout` accepts any `Element` in that slot.

```
side_bar  body
┌───┬─────────────────────┐
│ 📁 │                     │
│ ⚙  │   page content      │
│    │                     │
└───┴─────────────────────┘
      footer
```

Active item uses `button::primary`; inactive items use `button::text`.

### App state

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum NavPage { Explorer, Settings }

pub struct App {
    nav_page: NavPage,            // new
    settings_page: SettingsDialog, // promoted from dialog slot (persistent)
    // header, aside, gallery, footer — unchanged fields
    // Dialog enum: SettingsDialog variant removed
}
```

`SettingsDialog` becomes a persistent field rather than a dynamically
created dialog variant. Its state (selected tab, AI loading message)
is preserved when the user navigates away and back.

### Explorer page

When `nav_page == Explorer`, the body is:

```
column![
    header.view()        ← dir input + similarity-pairs button (full width)
    row![
        aside.view()     ← dir tree (always visible — toggle removed)
        gallery.view()   ← existing gallery
    ]
]
```

The `AppLayout.header` slot is **not used**. The header widget renders
inside the Explorer body so that it is absent while the Settings page
is displayed.

The existing `mouse_area` wrapper for cursor tracking stays around the
tiling row.

### Settings page

When `nav_page == Settings`, the body is:

```
settings_page.view()    ← full body, no surrounding header
```

The settings content currently rendered inside a fixed `600 × 400`
container grows to fill the available area.

### Header cleanup

`settings_nav` submodule and `SettingsOpen` event are removed from
`Header`. The header now surfaces only `DirSelect` and
`SimilarPairsDialogOpen`.

### Aside cleanup

The open/close toggle buttons and `is_open: bool` state are removed
from `Aside`. The directory tree is always visible inside the Explorer
tiling row. The `DirSelect` event path and processing-state propagation
are preserved unchanged.

## Message routing

| Old | New |
|-----|-----|
| `HeaderMessage::Event(SettingsOpen)` → open `Dialog::SettingsDialog` | removed |
| `SettingsDialogMessage` → `dialog` field if `SettingsDialog` variant | `SettingsDialogMessage` → persistent `settings_page` field |
| *(none)* | `NavTo(NavPage)` → sets `self.nav_page` |

## Dependency changes

None. The nav rail is built from existing iced + lucide-icons
primitives; no new crates are required.

## Touches in detail

| File | Change |
|------|--------|
| `app/src/core.rs` | `NavPage` enum; `nav_page` + `settings_page` fields; remove `Dialog::SettingsDialog` |
| `app/src/core/message.rs` | Add `NavTo(NavPage)` |
| `app/src/core/update.rs` | `NavTo` handler; route `SettingsDialogMessage` to persistent field; remove `SettingsOpen` arm |
| `app/src/core/view.rs` | Side bar element; page-switching body; header inside Explorer body; no `AppLayout.header` |
| `crates/ui/layout/src/header.rs` | Remove `settings_nav` field |
| `crates/ui/layout/src/header/view.rs` | Remove settings button from row |
| `crates/ui/layout/src/header/message.rs` | Remove `SettingsOpen` from `Event`; remove `SettingsNavMessage` from `Internal` |
| `crates/ui/layout/src/header/update.rs` | Remove `SettingsNavMessage` arm |
| `crates/ui/layout/src/header/settings_nav*` | Deleted |
| `crates/ui/layout/src/aside.rs` | Remove `is_open` field |
| `crates/ui/layout/src/aside/view.rs` | Always-visible dir tree; toggle removed |
| `crates/ui/layout/src/aside/message.rs` | Remove `Open` / `Close` from `Internal` |
| `crates/ui/layout/src/aside/update.rs` | Remove `Open` / `Close` handlers |

## Open questions

None at time of authoring. The Cache control page is explicitly out of
scope; it will be added as a new `NavPage::Cache` variant in its own RFC
with no structural change to the shell.
