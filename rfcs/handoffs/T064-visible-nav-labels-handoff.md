> **AUTHORIZED 2026-11-04 — owner:** *"Yes. Authorized. Proceed on the
> improvements and enhancement."* Written 2026-11-04 by the architect.
>
> **Implements [RFC 052](../proposed/052-text-size-and-reach.md) §3**,
> accepted the same day.
>
> **Independent of T063 and T061**, but it touches `app/src/core/view.rs`, which
> T061 also touches. **Coordinate the order with the architect** rather than
> both editing it at once.
>
> **Moved 2026-11-08** from `.git-exclude/tasks/dev-team/` into `rfcs/handoffs/`, at the owner's direction, and completed against the 17-item Developer Handoff structure. The authorization above is unchanged.

# T064 Handoff — show the navigation labels without hover

Companion to [RFC 052](../proposed/052-text-size-and-reach.md) §3.

## 1. The finding

`app/src/core/view.rs:55-82` builds all three navigation buttons the same way:

```rust
let settings = tooltip(
    button(icon_settings()).style(...).on_press(Message::NavTo(NavPage::Settings)),
    text(t("nav.settings")),
    tooltip::Position::Right,
);
```

**The labels already exist and are already translated in both locales** —
`nav.explorer`, `nav.cache`, `nav.settings`. They are shown only on hover.

**Hover is the wrong gate.** It needs a steady pointer, it is slow with tremor,
it does not exist on touch, and it is invisible to a user who does not know to
try. Three glyphs with no persistent text is a recognition task on every visit.

**No new strings. No new translations.** This is layout.

## 2. What to decide, and say why

The RFC does not prescribe the shape. Two reasonable ones:

- **Labels permanently beside the icons** — simplest, always readable, widens
  the rail.
- **An expand/collapse rail**, labels shown when expanded.

**Pick one and justify it in the package.** Weigh it against *"less is more"*:
a permanently wider rail costs horizontal space on every screen, and the gallery
is the point of the application. An expanding rail keeps the default narrow but
reintroduces a discovery problem if the expand control is itself only an icon.

**Whatever you choose, the default state must make the three destinations
legible without hovering.** That is the requirement; the shape is yours.

## 3. What not to break

- **The focus ring.** The rail carries
  `arama_theme::focus_ring_style(self.is_zone_focused(FocusZone::SideBar))`.
  RFC 044 made zone focus visible and F6 cycles between zones — keep both
  working.
- **The active-page styling.** Each button switches between
  `arama_theme::primary` and `arama_theme::ghost` on `self.nav_page`. The
  current page must stay distinguishable **by more than colour** once labels
  are present.
- **Target size.** The buttons measure ≈52 × 76 logical px today, which clears
  WCAG 2.5.5's enhanced 44 × 44. **Do not shrink them** to make room for text.
- **The tooltips.** Keeping them alongside visible labels is redundant; removing
  them is reasonable. Say which you did.

## 4. It must survive T063

T063 adds a text-size setting with a **Larger** step. **Labels that fit at
Default must still fit at Larger** — or the rail must grow. Check it if T063 has
landed by the time you start; if it has not, say so and state the assumption.

## 5. Evidence

**the local GUI rules file (`.git-exclude/rules/project-instructions-gui-common.md` — in the working copy, not the repository) governs this**,
including its "GUI Evidence and Automation Rules".

Appearance **is** the claim here, so a capture is the right evidence — one per
locale, since Japanese `nav.*` labels differ in width from English. Follow the
rules when taking them: size obtained at launch, isolated `XDG_CONFIG_HOME` and
`XDG_STATE_HOME`, one instance by pid, backend stated, **and report window size
and file dimensions separately, saying which is which.**

Keyboard navigation (F6 zone cycling, RFC 044) is a **keyboard** claim, so it
goes through the keyboard path the rules describe — not AT-SPI — and states its
backend.

## 6. Change scope

- `app/src/core/view.rs`
- possibly `crates/theme/src/lib.rs` if the rail needs a style

## 7. Non-change scope

- `crates/i18n/` — the strings exist; this task adds none
- the Similar Pairs button — RFC 052 §4, held behind T061
- `CHANGELOG.md` or any `Cargo.toml` version field

## 8. Documentation updates

- **`docs/src/users/using-arama.md`** — if it describes navigation by its icons, update it to match what users now see.
- No new user-facing strings (§1).

## 9. Compatibility constraints

- No data, settings or cache change.
- **RFC 044's keyboard operability must survive**: F6 / Shift+F6 zone cycling and the visible focus ring on the rail.
- The current page must stay distinguishable **by more than colour** (WCAG 1.4.1) once labels are present.

## 10. Security constraints

None — layout only; no new input, output or data.

## 11. Known risks

- A permanently wider rail costs gallery space on every screen.
- An expand control that is itself only an icon reintroduces the discovery problem this task removes.
- Labels that fit at Default may not fit at T063's Larger (§4).
- **T061 also edits `app/src/core/view.rs`** and has right of way.
- Shrinking the buttons to make room for text would undo the ≈52 × 76 logical px targets that currently clear WCAG 2.5.5.

## 12. Acceptance criteria

- The three destinations are legible **without hovering**, in both locales.
- The focus ring and active-page styling still work; the current page is identifiable by more than colour.
- Targets stay at or above 44 × 44 logical px.
- The labels fit at T063's Larger — or the package states the assumption if T063 has not landed.
- `cargo build`, `test`, `fmt --check` and `clippy -D warnings` clean.

## 13. Required deliverables

Review request with `## Entry point`. Include your §2 choice and its reasoning,
a capture per locale, confirmation that focus ring and active-page styling still
work, and §4's answer about Larger.

You cannot push: report the commit and the architect pushes and watches every
workflow on that SHA.

Report observed output; a check not run is recorded as not run.

## 14. Handoff checklist

The 17 items the architect's operating instructions require of every Developer Handoff, and where each is covered here.

| Required item | Covered in |
|---|---|
| Task title | Heading |
| Purpose | §1 |
| Background | §1 |
| Applicable RFC and requirements | Authorization banner and companion line |
| Change scope | §6 |
| Explicit non-change scope | §7, §3 |
| Required implementation | §2 |
| Required tests | §5 |
| Required documentation updates | §8 |
| Acceptance criteria | §12 |
| Prohibited shortcuts | §3 |
| Relevant code or module boundaries | §6, §3 |
| Compatibility constraints | §9 |
| Security constraints | §10 |
| Known risks | §11 |
| Required evidence | §5 |
| Required review-request format | §13 |
