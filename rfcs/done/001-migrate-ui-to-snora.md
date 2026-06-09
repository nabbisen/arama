# RFC 001 — Migrate the UI layer to the snora framework (v0.8)

**Status.** Implemented (v0.22.0)
**Tracks.** Replacement of arama's hand-rolled iced application
skeleton (layout composition, dialog overlay, context menu,
backdrop handling) with the `snora` crate, v0.8.x.
**Touches.** `app/src/core/view.rs`, `app/src/core/update.rs`,
`app/src/core/message.rs`, `crates/ui/widgets/src/dialog.rs`
(the `overlay` / backdrop helpers), `crates/ui/widgets/src/context_menu/`,
`crates/ui/layout/` (header / aside / footer remain but are
re-hosted as snora slots), workspace `Cargo.toml`
(new dependency `snora = "0.8"`).

## Summary

arama currently composes its application chrome by hand: a
`column![header, row![aside, body], footer]` skeleton, a
custom modal `overlay()` function with a hand-built dim
backdrop, and a context menu positioned with `space()` spacers
inside a `stack!`. snora v0.8 provides exactly this skeleton
(`AppLayout` + `render`) plus the overlay surfaces arama uses
(dialog, context menu) and ones it currently lacks but needs
(toasts for the many `// todo: error handling` sites), with
RTL/logical-edge support that aligns with the project's i18n
requirement.

This RFC replaces the bespoke composition code with snora while
keeping every arama view component (header, aside, footer,
gallery, dialogs) as-is — they are already plain
`Element`-producing functions, which is precisely what snora's
slots accept. The migration is therefore a *re-hosting* of
existing elements, not a rewrite of them.

## Motivation

1. **Less owned plumbing.** `crates/ui/widgets/src/dialog.rs`
   (`overlay`, `backdrop`) and the stacking logic in
   `app/src/core/view.rs` re-implement what snora's `render`
   does: layered stack of skeleton → menu backdrop → menus →
   modal backdrop → dialog → toasts. Owning this code means
   owning its bugs (the comment trail in `dialog.rs` about
   `opaque`/`mouse_area` ordering shows this class of bug has
   already bitten once).
2. **Error surfacing.** The codebase has ~15
   `// todo: error handling` sites and `expect()` calls in the
   GUI path. snora's `Toast` + `subscription` + `sweep_expired`
   gives a framework-managed, non-blocking error channel that
   these sites can target incrementally.
3. **i18n alignment.** The project requires multilingual GUI
   support. snora's `LayoutDirection::Rtl` and logical edges
   (`Edge::Start`/`Edge::End`) make RTL locales a one-setter
   concern instead of a future layout rewrite.
4. **Same foundations.** snora targets iced `0.14` and uses
   `lucide-icons` — both already arama workspace dependencies.
   No version skew is introduced.
5. **Shared maintenance.** snora is maintained by the same
   author (nabbisen); fixes and feature requests flow upstream
   instead of accumulating as arama-local patches.

## Current state (as-is analysis)

### Skeleton (`app/src/core/view.rs`)

```rust
let layout = mouse_area(column![
    container(header).height(60),
    container(row![aside, content]).height(Fill).padding([0, 20]),
    container(footer).height(40)
])
.on_move(Message::CursorMove);
```

- Fixed heights: header 60 px, footer 40 px.
- A `mouse_area(...).on_move(Message::CursorMove)` wraps the
  whole skeleton; the cursor position is stored on `App` and
  later used to place the context menu.

### Modal dialogs

- `App.dialog: Option<Dialog>` with three variants
  (`MediaFocusDialog`, `SimilarPairsDialog`, `SettingsDialog`).
- `arama_ui_widgets::dialog::overlay(base, dialog, Some(Message::DialogClose))`
  builds: opaque base → dim backdrop (`a = 0.90`, click =
  `DialogClose`) → centered opaque dialog card.

### Context menu

- `ContextMenu` widget positions itself with
  `space().height(point.y)` / `space().width(point.x)` inside a
  `stack!` layer above the skeleton.
- Dismissal is handled ad hoc (clicks elsewhere; no dedicated
  transparent backdrop, so dismissal behaviour is inconsistent
  with the dialog layer).

### Gaps

- No toast/notification surface; errors go to `eprintln!` or
  panic via `expect()`.
- No RTL support; all edges are physical (left/right).

## Target design (to-be)

### External design

The user-visible behaviour is unchanged except:

- Context-menu dismissal becomes consistent: any click outside
  the menu closes it (snora's transparent menu backdrop wired
  to `on_close_menus`).
- A toast area appears (proposed: `ToastPosition::BottomEnd`)
  for non-fatal errors (cache write failures, model-load
  failures, ffmpeg errors). Toast intents map: `Error` for
  failures, `Info` for long-running status (optional, phase 2).
- Dialog backdrop click-to-close behaviour is preserved
  (`on_close_modals(Message::DialogClose)` replaces the current
  `overlay(.., Some(Message::DialogClose))`).

### Internal design

#### Dependency

```toml
# workspace Cargo.toml
snora = "0.8"
```

`app` and (if context-menu code moves) `crates/ui/widgets`
depend on it. `iced` stays at `0.14` (workspace pin unchanged).

#### `App` state changes (`app/src/core.rs`)

| Field | Change |
|---|---|
| `context_menu: ContextMenu` | kept; its `view()` keeps producing the positioned `Element`, now fed to `AppLayout::context_menu` |
| `dialog: Option<Dialog>` | kept; mapped to `AppLayout::dialog(snora::Dialog::new(elem))` |
| `toasts: Vec<snora::Toast<Message>>` | **new** |

#### `Message` changes (`app/src/core/message.rs`)

```rust
enum Message {
    // existing variants unchanged, plus:
    CloseMenus,                 // sink for snora on_close_menus
    ToastDismiss(usize),        // manual toast dismissal
    ToastSweep(std::time::Instant), // from snora::toast::subscription
    // Message::DialogClose already exists and is reused for on_close_modals
}
```

#### `view()` rewrite (`app/src/core/view.rs`)

```rust
use snora::{AppLayout, Dialog as SnoraDialog, render};

pub fn view(&self) -> Element<'_, Message> {
    if !self.setup.finished && !setup::util::ready() {
        return self.setup.view().map(Message::SetupMessage);
    }

    let body = self.gallery.view(self.footer.thumbnail_size())
        .map(Message::GalleryMessage);

    let mut layout = AppLayout::new(body)
        .header(self.header.view().map(Message::HeaderMessage))
        .side_bar(self.aside.view().map(Message::AsideMessage))
        .footer(self.footer.view().map(Message::FooterMessage))
        .on_close_menus(Message::CloseMenus)
        .on_close_modals(Message::DialogClose)
        .toasts(self.toasts.clone())
        .toast_position(ToastPosition::BottomEnd);

    if self.context_menu.is_open() {
        layout = layout.context_menu(
            self.context_menu.view().map(Message::ContextMenuMessage));
    }
    if let Some(dialog) = &self.dialog {
        layout = layout.dialog(SnoraDialog::new(dialog_element(dialog)));
    }

    render(layout)
}
```

Notes:

- `mouse_area(...).on_move(Message::CursorMove)` wraps the
  *body* element (gallery), not the whole skeleton. The cursor
  point is only consumed for context-menu placement over
  gallery cells, so narrowing the capture area is acceptable
  and avoids wrapping snora's output. If full-window tracking
  proves necessary, fall back to wrapping `render(layout)` in
  the `mouse_area` — snora's output is an ordinary `Element`,
  so this composes.
- Header/footer fixed heights (60/40 px) move into the
  header/footer components' own `view()` (wrap in
  `container(..).height(..)`), keeping pixel parity. Whether
  snora's skeleton imposes its own slot sizing is **open
  question Q1** for the snora author.

#### Code deletions

- `crates/ui/widgets/src/dialog.rs`: `overlay()` and
  `backdrop()` are deleted. `card_style()` is kept (snora is
  explicitly "skeleton, not styling"; the dialog card look
  remains arama's).
- `app/src/core/view.rs`: `stack!` composition removed.
- Context menu spacer-positioning code is retained for now
  (iced 0.14 has no absolute-position primitive; snora's own
  context-menu example uses the same manual technique), but
  moves behind the `AppLayout::context_menu` slot so dismissal
  is framework-managed.

#### Toast lifecycle wiring

```rust
// subscription (app/src/core/subscription.rs)
snora::toast::subscription(&self.toasts, Message::ToastSweep)

// update (app/src/core/update.rs)
Message::ToastSweep(now) => {
    snora::toast::sweep_expired(&mut self.toasts, now);
    Task::none()
}
```

Phase 1 converts only the existing silent failure points in
`update.rs` (`ThumbnailCacheFinished` error branch, settings
load failure) to `ToastIntent::Error` toasts. Remaining
`expect()` sites are tracked as follow-up work, not in this
RFC's scope.

### Program design — file-level plan

| File | Action |
|---|---|
| `Cargo.toml` (workspace) | add `snora = "0.8"` |
| `app/Cargo.toml` | add `snora = { workspace = true }` |
| `app/src/core.rs` | add `toasts` field; init in `new()` |
| `app/src/core/view.rs` | rewrite per above (~60 → ~50 lines) |
| `app/src/core/message.rs` | add 3 variants |
| `app/src/core/update.rs` | handle new variants; toast pushes at 2 error sites |
| `app/src/core/subscription.rs` | merge toast subscription with existing ones |
| `crates/ui/widgets/src/dialog.rs` | delete `overlay`/`backdrop`; keep `card_style` |
| `crates/ui/widgets/src/context_menu/*` | add `is_open()`; remove ad-hoc dismissal |
| `crates/ui/layout/src/{header,footer}/view.rs` | absorb fixed heights |

All touched files stay well under the 300-ELOC guideline.

## Migration plan (phased)

1. **Phase A — skeleton + dialogs.** `AppLayout` with header /
   side_bar / footer / dialog / `on_close_modals`. Delete
   `overlay()`. Pixel-parity check by manual run of the three
   dialogs.
2. **Phase B — context menu.** Move under
   `AppLayout::context_menu` + `on_close_menus`. Verify
   open-at-cursor and click-outside-dismiss.
3. **Phase C — toasts.** Add toast state, subscription, and the
   two initial error-site conversions.

Each phase is one reviewable commit series; the app builds and
runs after each phase.

## Testing

- Per project testing guidelines, tests validate the design
  spec, not the code: the spec items are (a) dialog opens
  centered over a dim backdrop and backdrop click emits
  `DialogClose`; (b) context menu renders at the stored point
  and outside click emits `CloseMenus`; (c) expired toasts are
  removed by `ToastSweep`.
- (a)/(b) are layout-composition behaviours owned by snora and
  covered by snora's own suite; arama's tests cover the
  *message wiring*: unit tests on `App::update` transitions
  (dialog field cleared on `DialogClose`, context-menu state
  cleared on `CloseMenus`, toast vec shrinks on sweep). These
  live in `app/src/core/tests.rs` per the test-organization
  guideline.
- Manual smoke checklist (until an iced headless harness
  exists): the three dialogs, context menu on an image cell,
  toast on a forced cache error.

## Risks and mitigations

| Risk | Mitigation |
|---|---|
| snora's backdrop dim differs from current `a = 0.90` look | acceptable cosmetic change; if not, request a dim-style knob upstream (Q2) |
| Cursor tracking regression after narrowing `mouse_area` | fall back to wrapping `render()` output |
| snora API churn (pre-1.0) | pin `=0.8.x` in workspace; author is reachable for migration notes |
| Layered z-order conflicts with gallery internal stacks | snora layers are additive over an opaque skeleton (verified in `render.rs`); gallery never escapes the body slot |

## Alternatives considered

- **Keep the hand-rolled composition.** Rejected: duplicated
  effort with upstream, and the toast/RTL gaps would still need
  in-house builds.
- **Adopt snora-widgets prefabs (`app_header`, `app_side_bar`)
  too.** Deferred: arama's header/aside/footer are functional
  and styled; replacing them is churn without user benefit.
  Re-evaluate when the i18n work (separate RFC) lands, since
  the prefabs are direction-aware out of the box.
- **Other iced component libraries (e.g. iced_aw).** Rejected
  for this scope: they provide widgets, not the application
  skeleton + overlay layering that is arama's actual
  duplication.

## Questions for the snora author

- **Q1.** Does `build_skeleton` constrain header/footer slot
  heights, or does the slot adopt its child's intrinsic height?
  arama needs 60 px / 40 px parity.
- **Q2.** Is the modal dim color/alpha configurable, or fixed?
  arama currently uses `rgba(0,0,0,0.90)`.
- **Q3.** Any plan for a cursor-anchored overlay primitive for
  context menus (the example notes iced 0.14 lacks one)? arama
  carries spacer-based positioning that would gladly move
  upstream.

## Open questions

- Whether `Message::CursorMove` tracking should move to an
  `iced::mouse::Event` subscription (as snora's example
  suggests) instead of a `mouse_area` wrapper. Out of scope
  here; candidate follow-up RFC.

## Out of scope

- i18n string externalization (own RFC; this RFC only ensures
  the layout layer won't block RTL).
- Replacing arama's header/aside/footer internals with
  snora-widgets prefabs.
- Full `expect()`-to-toast conversion across the codebase.
