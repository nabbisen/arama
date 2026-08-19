# RFC 044: Keyboard operability

**Status.** Proposed — **requested by the project owner 2026-08-18**, after
snora 0.35.0 shipped the frame-level half of it. **Revised the same day** on
snora's review of the draft: Phase 0.2's blocking unknown is closed, three
implementation constraints were added, and the F6 recommendation is weaker than
it was. See §0.2, §2.2, §2.3, §3.1.
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

## Phase 0 — answered by running, not reading. Partly closed.

**0.2 is closed and the answer was better than the question**; 0.1 and 0.2b
remain. I was wrong about 0.2's premise — I framed it as "can we inject OS-level
input", having been burned on exactly that in RFC 040, and the answer is that
arama never needed to.

### 0.1 What does the keyboard do in arama today?

I have established what arama *installs*: nothing. I have **not** established
what iced 0.14 does on arama's behalf without being asked. `text_input` is
focusable in iced, and whether Tab traverses anything by default is a property
of the framework, not of arama's code.

**Press keys and record what happens.** Tab, Shift+Tab, Escape, Enter, arrows,
F6 — on the gallery, in a dialog, and on the settings page. That observation is
this RFC's baseline and a reading-only answer would report "nothing happens" and
might be wrong.

**0.2's answer makes this cheaper than written.** `Simulator::tap_key` returns
an `iced::event::Status` — `Captured` or `Ignored` — so "does anything consume
this key today?" is assertable in-process, per key, without running the
application at all. Do that first; run the real application only for whatever it
cannot answer.

### 0.2 ~~Can keyboard events be injected into arama at all?~~ — answered

> **Answered 2026-08-18 by snora's review, then verified against the crate.**
> **The premise was wrong: no OS-level injection is needed.**
>
> `iced_test 0.14`'s `Simulator` drives keyboard events **in-process** — no
> window, no compositor, no injection path to fail. Verified in
> `iced_test-0.14.0/src/simulator.rs`, not taken on report:
>
> | API | Line | Purpose |
> |---|---|---|
> | `tap_key(impl Into<keyboard::Key>)` | `:164` | press + release |
> | `press_key` / `release_key` | `:370` / `:390` | held modifiers |
> | `typewrite(&str)` | `:172` | text entry |
> | `snapshot(&Theme) -> Result<Snapshot, Error>` | `:199` | **renders to pixels** |
> | `Snapshot::matches_image(path)` | `:265` | compare against a reference PNG |
> | `Snapshot::matches_hash(path)` | `:307` | compare a SHA-256 of the RGBA |
>
> So the one thing §5 said needed a human — *the indicator appears, on the
> right zone, and reads on all four presets* — is a loop over four snapshots.
>
> **snora flagged their own limits honestly:** they use the simulator for
> composition assertions only and have **never used `snapshot`** (their
> RFC-011-D chose semantic over pixel testing deliberately), their
> `render_semantics` carries a *"CI hardware; may OOM locally"* note, and
> reference images are a maintenance surface — their border repair and the
> 0.37.0 dim change would each have invalidated ours.

**Two footguns neither of us knew about, found by reading the crate. Both make a
snapshot suite pass while testing nothing.**

**(a) A missing reference auto-passes.** `matches_image` returns `Ok(true)` and
*creates* the PNG when the path does not exist (`simulator.rs:265-306`;
`matches_hash` does the same). A first run always passes. So does a run after
someone deletes a reference — and whatever the code produces at that moment
silently becomes the baseline, regression included.

**(b) The reference filename embeds the renderer.** `path()` builds
`{stem}-{renderer}.{ext}` (`simulator.rs`, `fn path`). A `wgpu` reference and a
`tiny-skia` reference are different files.

**They compound.** A CI runner with no GPU falls back to `tiny-skia`, looks for
a reference that does not exist under that name, creates it, and **passes**. The
suite is green and inert, and nothing says so.

**Required if the snapshot route is taken:** assert the reference existed. A
test that cannot fail on a missing baseline is not a test, and this project has
shipped one of those before — the 0.38.0 release published zero assets while
every check was green.

### 0.2b What is left to establish

- **Does `snapshot` work at all here**, on this hardware, for a focus ring?
  snora has never run it and wants the answer as much as we do.
- **Which renderer** the run resolves, and whether it is stable between a
  developer machine and CI.
- **Whether a focus ring is even visible at snapshot resolution.** A 2 px ring
  is a small signal; a pixel comparison will detect it, but *"reads correctly"*
  is a human judgement a hash cannot make. Expect to keep one rendered capture
  for the judgement and use snapshots for the regression guard.

> **Second route confirmed 2026-08-19, and it was not asked for.** Driving arama
> under forced XWayland (`env -u WAYLAND_DISPLAY DISPLAY=:1`, arama itself as
> the launched process) delivers **both pointer and keyboard** events to the
> real application — verified across packages 116 and 117, including typed text
> and `Return`. Renders identically to native Wayland at the same size.
>
> So **0.1 is now runnable against the real application**, not only through
> `iced_test`; **Tier 3's focus-indicator captures have a second viable route**;
> and once this RFC ships, **Escape and F6 become directly testable the same
> way.** Two captures of the same state from independent launches came back
> md5-identical, so this route supports pixel controls rather than only
> eyeballing.
>
> The earlier negative — pointer motion but no buttons — was a *different
> configuration*: arama running native-Wayland while `xdotool` talked to
> XWayland, which cannot work because XTEST delivers only to X clients.

**The fallback, if the in-process route fails.** snora reports another adopter
has working native-Wayland keyboard injection needing no root, no daemon and no
`uinput`; details are pending that team's permission. They also passed on a trap
worth recording whether or not we ever need it:

> **Verify delivery by observing the client, not by triggering a compositor
> keybinding.** A compositor can decline to act on its own binding while still
> forwarding the key — so that test reports failure when delivery works.

That is precisely the shape of RFC 040's Xwayland error: a negative result from
the wrong observation point.

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
absent by construction and the cycle is `SideBar → Body → Footer`.

snora confirmed both readings. `ZonePresence` describes **slot occupancy, not
visual presence**, so "a header that exists visually but not as a slot" is the
case the field was written for. And `next_zone` **deliberately ignores**
`has_menu` — `let _ = has_menu;` in their source, with the reason in the doc
comment — so arama's `context_menu` cannot suspend cycling by construction
rather than by coincidence. Its `dialog` does.

> **The consequence snora volunteered, which I had not asked about and should
> have:** because arama's header lives inside `body`, **F6 will never stop on
> the header.** Everything in it — including directory navigation — is reachable
> only once focus is already in `Body`.
>
> Making the header its own stop means moving it into `AppLayout::header`. That
> is a **layout** change, not a keyboard one. snora explicitly does not
> recommend it and neither do I: it would reshape a working layout to serve a
> cycle. **But it must be a decision rather than a discovery**, so it is recorded
> as §3.5.

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
snora confirmed this reading, and tekstide ships it today.

**But `FocusTokens` does not map cleanly onto a container border, and this
changes what the indicator looks like:**

| `FocusTokens` | `iced::Border` | |
|---|---|---|
| `ring_color` | `color` | ✅ |
| `ring_width` | `width` | ✅ |
| **`ring_offset`** | — | ❌ **no equivalent** |

`iced::Border` carries `color`, `width` and `radius` only. **A ring drawn
*outside* the control's edge is not expressible**, so the indicator is an
**inset** ring unless padding or a nested bordered container is added.

Inset rings read differently against dense content — and arama's densest content
is a thumbnail grid. **Decide this deliberately rather than discovering it**:
honour colour and width and accept the inset, or add the structure. snora
flagged it unprompted specifically so it would not be found mid-implementation.

**Two channels, colour *and* width.** tekstide forbids a colour-only indicator
and snora agrees; so do I. A colour-only ring fails for the users most likely to
need it.

## 3. Design questions

**3.1 Which key cycles zones, and how does anyone find out?**

snora recommends F6 / Shift+F6, but `next_zone` takes a direction rather than a
key, so the binding is entirely arama's.

> **snora's recommendation is weaker than it reads, and they said so
> unprompted:** it is **convention-based, not evidence-based** — chosen because
> F6 is the desktop convention for pane cycling and because Tab was unavailable
> to a framework, *not* because anyone knows users find it.
>
> **Nobody has shipped it.** orbok has not adopted zone navigation; apimokka
> **declined** — two zones is a toggle, not worth a binding a user must learn;
> tekstide uses Tab because they own their entire shell. arama would be first,
> with no downstream data from anyone.
>
> They also said the discoverability concern is sound and they cannot refute it
> with data.

So this splits into two questions, and the second is the real one:

- **Which key?** F6. No reason to diverge from the convention, and diverging
  would be a second undiscoverable binding rather than a discoverable one.
- **How does a user learn it exists?** Documentation is the floor and is close
  to useless on its own — nobody reads a keyboard-shortcuts page for a photo
  browser. snora invited an in-app affordance as a better answer than the
  convention and asked to hear it.

**Recommendation: F6, plus a visible affordance in the footer.** arama already
has a footer that is a live zone. A hint costs one line and is seen by the
people who need it, which a docs page is not.

> **Settled 2026-08-20 — owner accepted.** F6 / Shift+F6, **with a hint in the
> footer.** So the binding is discoverable in the application rather than only
> in documentation, and arama becomes the first consumer to ship zone
> navigation with an affordance rather than a convention alone.
>
> Two things the implementation must decide, and neither is settled here:
> **what the hint says** — it is competing for space in a footer that already
> carries the thumbnail slider and a file count — and **whether it is
> permanent or appears on first focus movement.** Both are cheap to try and
> should be decided against a rendered capture, not in prose.
>
> This is also the piece snora asked to hear about: they have no downstream
> data on F6 from anyone, and said so.

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

**3.5 Should the header become a real slot?** §2.2's consequence: as composed
today it can never be an F6 stop. **Recommendation: no, not in this RFC.**
Reshaping a working layout to serve a cycle is the tail wagging the dog, and the
header's contents stay reachable through `Body`. Recorded so that if a user
reports the header as unreachable, the answer is a decision we made rather than
one we never saw.

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

## 5. Verification — three tiers, and only the last needs a person

Phase 0.2 changed this section from "one thing needs a human" to "one *judgement*
does". The work splits cleanly:

**Tier 1 — pure, no renderer.** `next_zone`'s cycle order, wrapping,
slot-skipping and modal suspension; Escape routing through `dismiss_on_escape`.
Unit tests over pure functions. snora made the decision half pure deliberately
and it is the reason this RFC is cheap.

**Tier 2 — in-process, headless.** `Simulator::tap_key` + assert arama's own
focus state: *F6 moved focus to the expected zone.* No window, no compositor,
no injection. This is the tier that was thought impossible when the RFC was
written.

**Tier 3 — rendered.** `Simulator::snapshot(&theme)` per preset, as the
regression guard, **subject to Phase 0.2b** and to the two footguns in 0.2:

- **Assert the reference existed.** `matches_image` and `matches_hash` create a
  missing reference and return `true`. A suite that cannot fail on a missing
  baseline is not a suite.
- **Pin or record the renderer.** References are named `{stem}-{renderer}`, so a
  GPU-less runner silently starts a fresh, always-passing set.
- Prefer `matches_hash` where a visual diff is not needed — a 64-byte digest
  instead of a PNG answers snora's maintenance-surface objection, at the cost of
  telling you nothing about *how* a failure differs. Use images where a human
  will need to look.

**What still needs a person: whether the ring *reads*.** A pixel comparison
proves it is present and unchanged. It cannot say a 2 px inset ring is findable
against a dense thumbnail grid on `high_contrast_dark`. **Keep one rendered
capture per preset for that judgement** and let snapshots guard the regression.

**If Tier 3 does not work here**, say so and reduce the claim: ship 2.1 and 2.2
on Tier 1 and 2 evidence, and mark 2.3's rendered evidence manual and pending.
**Do not assert a focus ring is visible on evidence that does not show it.**

**snora wants Tier 3's answer.** They have never run `snapshot`, and if it works
for a focus indicator they intend to document the route — crediting arama —
"rather than let three teams each discover it separately." Whichever way it
goes, it is worth reporting.

## 6. Risks

- **Invisible focus movement.** The single worst outcome, and the reason 2.3
  gates 2.2. A user who presses F6 and sees nothing has lost their place.
- **Tab collision with text inputs.** arama has five. This is why snora refuses
  Tab and why arama should too, unless Phase 0.1 shows something surprising.
- **A green snapshot suite that tests nothing.** Now the sharpest risk on this
  list, and it replaces "the evidence method may not exist". Missing references
  auto-pass and are auto-created; reference names embed the renderer. Together
  they mean a GPU-less runner produces a fresh, always-passing set and reports
  success. This project has shipped exactly this shape before — 0.38.0
  published a release with zero assets while every check was green.
- **The 0.35.0 API is unverified here.** §1. Everything quoted comes from
  release notes, and this project's standing instruction is to treat an upstream
  claim as a claim. *(The `iced_test` claims in Phase 0.2 were treated that way
  and verified against the crate — which is how the two footguns were found.
  snora had not mentioned them because they had never run that path either.)*
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
