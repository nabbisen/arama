# RFC 044 Handoff — Keyboard operability

Companion to [RFC 044](../proposed/044-keyboard-operability.md), requested by
the project owner 2026-08-18. It stays in `rfcs/proposed/` until it ships, per
[RFC 000](../done/000-rfc-lifecycle-policy.md).

**Do [RFC 043](../proposed/043-typography-roles-and-prose-readability.md)
first.** Both are visual-evidence work and running them together puts two sets
of captures in flight at once. Neither blocks the other technically.

**Read the RFC first.** This handoff settles what it left open and names the
traps.

## 1. Design authority

1. [RFC 044](../proposed/044-keyboard-operability.md);
2. **snora's own review of that RFC**, `.git-exclude/tmp/review-arama-rfc044/`
   — they answered five questions and volunteered a consequence we had not
   asked about. Read it; it is shorter than the RFC;
3. [RFC 017](../done/017-visible-recoverable-error-ux.md) — nothing here
   surfaces errors, but the tier model governs if anything does;
4. [`dialog-card-edge-contrast-measured`](../notes/dialog-card-edge-contrast-measured.md)
   — the measurement discipline §5 expects, and the two ways it went wrong.

## 2. The three pieces, in dependency order. The order is not negotiable.

**2.1 Escape dismisses modals.** The smallest and oldest gap.
`snora::keyboard::dismiss_on_escape` has existed since snora 0.25.0 and arama
has never called it, because arama installs no keyboard subscription at all —
`app/src/core/subscription.rs` is eight lines and carries only the toast sweep.

arama already has both messages it needs, wired at
`app/src/core/view.rs:147-148` (`Message::CloseMenus`, `Message::DialogClose`).
This is a subscription and a match arm.

**Modal-before-menu precedence is snora's and is already correct.** Do not
re-derive it.

**2.2 Zone cycling.** `ZonePresence::none().side_bar(true).footer(true)` —
arama composes its header into `body` (`app/src/core/view.rs:128`), so `Header`
is absent by construction and the cycle is `SideBar → Body → Footer`.

snora confirmed both readings: `ZonePresence` describes **slot occupancy, not
visual presence**, and `next_zone` **deliberately ignores** `has_menu`, so
arama's `context_menu` cannot suspend cycling while its `dialog` does.

Requires new `App` state: which zone holds focus.

**2.3 Visible focus. This gates 2.2 — do not ship cycling without it.**

Moving focus invisibly is worse than not moving it: the user loses their place
with no way to recover it. If 2.3 turns out to be harder than expected, **ship
2.1 alone and stop**, rather than shipping 2.2 blind.

## 3. Settled — F6 plus a footer hint

The owner accepted F6 / Shift+F6 **with a visible affordance in the footer**
(RFC §3.1, settled 2026-08-20), rather than documentation alone.

**Why it is not just documentation:** snora's F6 recommendation is
**convention-based, not evidence-based** — their words. Nobody has shipped it:
apimokka declined it, tekstide uses Tab because they own their whole shell,
orbok has not adopted. **arama would be first, with no downstream data from
anyone.** A binding nobody can find is close to no binding.

**Two things the owner did not settle, and neither should be settled in prose:**

- **What the hint says.** It competes for space in a footer that already carries
  the thumbnail slider and a file count.
- **Whether it is permanent or appears on first focus movement.**

**Decide both against a rendered capture.** Try one, look at it, try the other.
This is the cheapest possible experiment and arguing about it costs more than
running it.

## 4. Phase 0 — one question left, and it is cheaper than the RFC says

**0.1 What does the keyboard do today?** Still open. But `Simulator::tap_key`
returns an `iced::event::Status` — `Captured` or `Ignored` — so *"does anything
consume this key today?"* is assertable in-process, per key, **without running
the application.** Do that first; run the real application only for what it
cannot answer.

**0.2 Can input be delivered?** **Closed, twice over.** Two working routes:

- **`iced_test::Simulator`, in-process.** `tap_key`, `press_key`, `typewrite`,
  and `snapshot(&Theme)`. No window, no compositor.
- **Forced XWayland**, arama itself as the launched process:
  `env -u WAYLAND_DISPLAY DISPLAY=:1`. Delivers pointer **and** keyboard to the
  real application — proven across packages 116 and 117, including typed text
  and `Return`. Renders identically in size to native Wayland.

**The trap that made this look impossible for months:** running arama natively
while pointing `xdotool` at XWayland cannot work, because XTEST delivers only
to X clients. **arama must be the X client.** Check `xdotool search --name`
finds a window before concluding anything from a null result.

## 5. Traps

- **`FocusTokens.ring_offset` has no `iced::Border` equivalent.** `Border`
  carries `color`, `width`, `radius` — no offset. **So the ring is *inset*
  unless you add padding or a nested bordered container.** Inset rings read
  differently against dense content, and arama's densest content is a thumbnail
  grid. Decide deliberately; snora flagged this unprompted so it would not be
  found mid-implementation.

- **Use two channels, colour *and* width.** tekstide forbids a colour-only
  indicator, snora agrees, so do I. A colour-only ring fails the users most
  likely to need it.

- **F6 will never stop on arama's header**, because it lives inside `body`.
  Everything in it — including directory navigation — is reachable only once
  focus is in `Body`. **Recorded as RFC §3.5 with a recommendation not to
  reshape the layout for it.** If a user reports it, the answer should be a
  decision we made, not one we never saw.

- **Do not take Tab.** snora's reasoning, adopted wholesale: Tab means "next
  control" to iced and to users, and arama has **five `text_input`s**. If
  Phase 0.1 shows iced provides no traversal at all, `operation::focus_next()`
  is reachable without the `advanced` feature — but that is a separate decision,
  not a consequence of this one.

- **Modal focus trapping is out of scope**, and arama is on record with snora
  as explicitly not wanting it. Do not add it opportunistically.

## 6. Non-change scope

- Arrow-key navigation of the gallery grid. A real feature, a separate one.
- Accelerators for actions — delete, prune, compare.
- Custom `FocusTokens` values. arama takes snora's presets, as everywhere.
- Screen-reader support. iced 0.14 does not make it answerable and pretending
  otherwise would be dishonest.
- RFC 043. Independent, and it precedes this.

## 7. Verification — three tiers, and only the last needs a person

**Tier 1 — pure.** `next_zone`'s cycle order, wrapping, slot-skipping and modal
suspension; Escape routing through `dismiss_on_escape`. Unit tests over pure
functions, no renderer. snora made the decision half pure deliberately and it is
why this RFC is cheap.

**Tier 2 — in-process, headless.** `Simulator::tap_key` plus an assertion on
arama's own focus state: *F6 moved focus to the expected zone.*

**Tier 3 — rendered.** `Simulator::snapshot(&theme)` per preset as the
regression guard, **subject to two footguns that make a suite pass while testing
nothing:**

- **A missing reference auto-passes.** `matches_image` returns `Ok(true)` *and
  creates* the PNG when the path does not exist (`iced_test-0.14.0/src/
  simulator.rs:265-306`); `matches_hash` does the same. **Assert the reference
  existed.** A test that cannot fail on a missing baseline is not a test.
- **Reference filenames embed the renderer** — `{stem}-{renderer}`. A GPU-less
  runner falls back to `tiny-skia`, finds no reference under that name, creates
  one, and passes. **Pin or record the renderer.**

**What still needs a person: whether the ring *reads*.** A pixel comparison
proves it is present and unchanged. It cannot say a 2 px inset ring is findable
against a dense thumbnail grid on `high_contrast_dark`. Keep one rendered
capture per preset for that judgement.

**Derive before you measure.** Both capture failures in Task 028 came from
sampling pixels that were not the thing being measured. Work out what the value
*should* be from the known constants first, then check the pixel matches.

## 8. Acceptance criteria

- Escape closes a dialog, and a menu, with snora's precedence.
- `next_zone` integration covered by Tier 1 tests including the modal-suspended
  and menu-unaffected cases.
- F6 moves focus and **the focus indicator is visible**, on every preset, shown
  by captures.
- The footer hint exists, and the two §3 questions are answered with a capture
  rather than an argument.
- No `text_input` regression from Tab.
- Tier 3, if used, cannot pass with a missing reference.
- Gates clean.
- **A `CHANGELOG.md` `[Unreleased]` entry**, written for a user deciding whether
  this affects them.

## 9. What we owe snora — gather it, do not send it

They asked for two things and both fall out of this work:

1. **Which key we bound, and whether the four-zone model fit** our layout.
2. **Whether `iced_test`'s snapshot path works for a focus indicator.** They
   have never run it and said that if it works they would document the route —
   crediting arama — rather than let three teams discover it separately.

Add a third, unprompted: **whether the footer hint was worth it.** They have no
data on F6 discoverability from anyone and said so.

**Do not send anything to snora.** Outward communication is the owner's.

## 10. Required deliverables

A review request under `.git-exclude/review-request/NNN-<slug>/` opening with an
`## Entry point` section giving the pinned commit, the exact `git show` command,
and plain paths to every file. Include the Phase 0.1 result, the captures, and
§9's three items.

Report observed output; a check not run is recorded as not run.
