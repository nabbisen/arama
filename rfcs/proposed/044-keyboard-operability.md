# RFC 044: Keyboard operability

**Status.** Proposed — **requested by the project owner 2026-08-18**, after
snora 0.35.0 shipped the frame-level half of it.
**Tracks.** Make arama operable from the keyboard: dismiss modals, move between
regions, and see where focus is.
**Touches.** `app/src/core/subscription.rs`, `app/src/core.rs` state, `view.rs`
styling. Possibly `crates/theme`. No product logic, no engine, no cache.
**Depends on.** Task 028 (snora 0.33 → 0.35). Nothing else.

## Summary

**arama has no keyboard interaction at all.**

`app/src/core/subscription.rs` is eight lines and installs exactly one
subscription — snora's toast sweep. There is no `iced::keyboard::listen()`
anywhere in `app/src` or `crates/ui`, and no call to any focus operation.

`AppLayout::on_close_modals` is wired to the **modal backdrop's `mouse_area`**
(`snora-0.33.0/src/render.rs:15`) — a click handler. The Escape half is a
separate pure function, `snora::keyboard::dismiss_on_escape`
(`snora-0.33.0/src/keyboard.rs:55`), which the application must call from its own
subscription. **arama has never called it**, and it has been available since
snora 0.25.0 — before arama's first snora version.

So a user who cannot or does not use a mouse cannot dismiss a dialog in arama.

arama renders **47 interactive controls** — 37 `button`, 5 `text_input`, 2
`slider`, 2 `checkbox`, 1 `pick_list` — and reaches none of them deliberately
from the keyboard.

## Why this is a capability question, not a polish question

WCAG 2.1 SC 2.1.1 (Keyboard) requires all functionality to be operable through a
keyboard interface. arama does not meet it. Neither SC 2.4.7 (Focus Visible),
because nothing renders a focus indicator, nor SC 2.1.2 (No Keyboard Trap),
which cannot even be evaluated without focus movement to trap.

This is not a shortfall against a standard nobody asked for. **arama is a local
photo-library tool whose whole argument is that a user's files never leave their
machine** — a trust argument aimed squarely at people who care how software
treats them. Shipping something a keyboard-only user cannot dismiss a dialog in
is inconsistent with that.

**RFC 043 and this RFC are the two halves of the same goal.** Typography is
whether text can be *read*; this is whether arama can be *used*. The second is
the larger gap, and it was invisible until snora 0.35.0 made the mechanism
available and prompted the look.

## Phase 0 — two questions, both answered by running, not reading. Blocking.

**Do not design past this section.** Both questions are cheap, and I have been
wrong about the second one before in this exact project.

### 0.1 What does the keyboard do in arama today?

I have established what arama *installs*: nothing. I have **not** established
what iced 0.14 does on arama's behalf without being asked. `text_input` is
focusable in iced, and whether Tab traverses anything by default is a property
of the framework, not of arama's code.

**Run arama and press keys.** Tab, Shift+Tab, Escape, Enter, arrows, F6. Record
what each does on the gallery, in a dialog, and in the settings page. That
observation is this RFC's baseline and there is no substitute for it — a
reading-only answer would report "nothing happens" and might be wrong.

### 0.2 Can keyboard events be injected into arama at all?

**This is the one that decides the shape of the work**, and this project already
has a scar on it.

RFC 040 needed synthetic input. Native Wayland injection did not work; the
Xwayland path was tried; **motion was delivered and button press/release was
not** (review 096 addendum). I had asserted that path would work, having verified
its preconditions rather than its capability. That was my error and it cost a
cycle.

**Keys are a different code path from buttons.** `xdotool key` and `xdotool
click` are not the same mechanism, and the bridge that dropped one may carry the
other. **Nobody knows, and it is one command to find out.**

- **If key injection works**, most of this RFC verifies automatically and the
  evidence is cheap.
- **If it does not**, every acceptance criterion below needs a human at a real
  keyboard, and the scope should be cut accordingly — see §5.

**Report the answer before implementing.** "Cannot be established" is a real
result, exactly as it was for Task 026.

## 1. What snora supplies, and what it explicitly does not

From the 0.34.0/0.35.0 release notes. **The 0.35.0 API below is quoted from
those notes and is not verified against the crate** — this repository's registry
holds nothing newer than 0.33.0. Task 028 resolves that; **verify before
relying.**

**Supplied:**

- `snora_core::focus::next_zone(current, Cycle, ZonePresence, has_modal, has_menu)`
  → `Option<FocusZone>`. Pure, iced-free, unit-testable without a renderer.
- Cycle order `Header → SideBar → Body → Footer`, wrapping, skipping absent
  slots. Not direction-mirrored under RTL, deliberately.
- Returns `None` while a modal is open — cycling suspended. A menu alone does
  not suspend it.
- `snora::keyboard::cycle_zones` maps F6 / Shift+F6 to a `Cycle`.
- `snora::keyboard::dismiss_on_escape` — already present since 0.25.0.
- `FocusTokens { ring_width, ring_offset, ring_color }`
  (`snora-design-0.33.0/src/focus.rs:19`), already reachable today.

**Explicitly not supplied, and these are the interesting ones:**

- **snora captures no keyboard events.** No subscription is installed on
  arama's behalf. Wiring `iced::keyboard::listen()` is arama's job.
- **snora does not take Tab.** Their reasoning is sound and worth adopting
  rather than re-deriving: Tab means "next control" to iced and to users, and a
  framework claiming it for region cycling breaks in-pane navigation for any
  application with a form or a text input. arama has five text inputs.
- **No modal focus trapping.** Cycling is *suspended* in a modal but focus is
  not *contained* — nothing stops Tab walking out of a dialog into the chrome
  behind it. snora probed iced 0.14 and found `operation::focus_next()` /
  `focus_previous()` reachable, but `focusable::find_focused()` — needed to
  detect the boundary — requires iced's `advanced` feature, which snora does not
  enable. **They shipped the half that required no bet**, and asked consumers to
  say if containment matters to them.

## 2. Scope, in dependency order

**2.1 Escape dismisses modals.** The smallest, oldest, and most clearly missing
piece. `dismiss_on_escape` already exists and arama already has the two messages
it needs (`Message::DialogClose`, `Message::CloseMenus`, wired at
`app/src/core/view.rs:147-148`). This is a subscription and a match arm.

Modal-before-menu precedence is snora's, already correct, and is not arama's to
re-decide.

**2.2 Zone cycling.** `ZonePresence::none().side_bar(true).footer(true)` — arama
composes its header into `body` (`app/src/core/view.rs:128`), so `Header` is
absent by construction and the cycle is `SideBar → Body → Footer`. arama's
`context_menu` is a **menu**, not a modal, so it does not suspend cycling; its
`dialog` does.

Requires new App state: which zone holds focus.

**2.3 Visible focus.** *This is the part that must not be skipped.* Moving focus
invisibly is worse than not moving it — the user loses their place with no way
to recover it. **2.2 must not ship without 2.3.**

snora corrected their own documentation here: they had said a focus ring "cannot
be rendered" on iced 0.14, which was over-scoped. The accurate constraint is that
iced cannot tell a style closure that a widget **iced** owns is focused. An
application that owns focus as its own state can style it today — a `container`
style closure is an arbitrary `Fn(&Theme) -> Style`, and anything arama knows is
available inside it. arama owns the state from 2.2, so `FocusTokens` applies.

## 3. Design questions

**3.1 Which key cycles zones?** snora recommends F6 / Shift+F6 and supplies
`cycle_zones` for it, but takes a direction rather than a key, so the choice is
arama's. F6 is the Windows convention and costs nothing; it is also a key many
users have never pressed. **Recommendation: F6, and document it** — an
undiscoverable binding is close to no binding.

**3.2 Does Tab do anything?** Phase 0.1 answers what it does today. If iced
provides no traversal, arama can call `operation::focus_next()`, which snora
confirmed is reachable without the `advanced` feature. **Do not decide this
before Phase 0 reports.**

**3.3 Is focus containment in modals required here?** snora left it to
consumers and asked for evidence. arama's modals are `MediaFocusDialog` and
`SimilarPairsDialog` — neither handles credentials or destructive confirmation,
so containment is a usability property rather than a security one.
**Recommendation: out of scope for this RFC**, and report that reasoning to
snora, since they asked for exactly this input.

**3.4 Does the focus indicator need a fifth theme preset dimension?** No —
`FocusTokens` is already in every preset. But whether the ring reads adequately
on all four presets is a rendering question, not an assertion, and belongs in
verification.

## 4. Non-goals

- **Full keyboard control of the gallery grid.** Arrow-key navigation between
  thumbnails is a real feature and a separate one. This RFC makes arama
  operable; it does not make it efficient.
- **Shortcuts for actions.** No accelerators for delete, prune, or compare.
- **Modal focus trapping.** §3.3.
- **Custom `FocusTokens` values.** arama takes snora's presets, as everywhere.
- **Screen-reader support.** A much larger question that iced 0.14 does not
  currently make answerable, and pretending otherwise would be dishonest.
- **RFC 043.** Concurrent, independent, and it precedes this.

## 5. Verification, and its dependency on Phase 0.2

**Most of this RFC is verifiable without input injection**, because snora
deliberately made the decision half pure:

- `next_zone`'s cycle order, wrapping, slot-skipping and modal suspension are
  **unit tests over a pure function**. No renderer.
- Escape routing through `dismiss_on_escape` is likewise pure.
- `iced_test::Simulator` can drive widgets programmatically — not a substitute
  for rendered evidence over a real gallery (review 096 established that), but
  entirely adequate for *decision* logic.

**Exactly one thing needs a real key press: that the focus indicator appears,
on the right zone, and is visible on all four presets.** That is a rendered
capture with a focused state, and reaching it requires either working key
injection (Phase 0.2) or a human.

**If Phase 0.2 fails**, say so and reduce the claim rather than working around
it: ship 2.1 and 2.2 with unit-test evidence, and mark 2.3's rendered evidence
as manual and pending. **Do not assert a focus ring is visible on evidence that
does not show it.**

## 6. Risks

- **Invisible focus movement.** The single worst outcome, and the reason 2.3
  gates 2.2. A user who presses F6 and sees nothing has lost their place.
- **Tab collision with text inputs.** arama has five. This is why snora refuses
  Tab and why arama should too, unless Phase 0.1 shows something surprising.
- **The evidence method may not exist.** §5. Named as a risk rather than assumed
  away, because the same assumption cost RFC 040 a cycle.
- **The 0.35.0 API is unverified here.** §1. Everything quoted comes from
  release notes, and this project's standing instruction is to treat an upstream
  claim as a claim.
- **Scope creep toward grid navigation.** Arrow keys in the gallery will feel
  like the obvious next step while the code is open. It is a separate RFC.

## 7. Open questions

- **Does anything in arama already trap the keyboard by accident?** Phase 0.1
  should notice if, for example, a `text_input` swallows Escape.
- **Should the F6 binding be discoverable in-app**, or only in documentation? A
  keyboard-shortcuts line in the settings About tab is cheap; deciding it needs
  the feature to exist first.
- **Does zone focus survive a page switch** between Gallery, Cache and Settings?
  Not answerable before 2.2's state shape exists.
