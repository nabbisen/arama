> **ACCEPTED 2026-11-16 — review 180.** Published as `d17229c`.

> **AUTHORIZED 2026-11-04 — owner:** *"Yes. Authorized. Proceed on the
> improvements and enhancement."* Written 2026-11-04 by the architect.
>
> **Implements [RFC 052](../done/052-text-size-and-reach.md) §3**,
> accepted the same day.
>
> **Independent of T063 and T061**, but it touches `app/src/core/view.rs`, which
> T061 also touches. **Coordinate the order with the architect** rather than
> both editing it at once.
>
> **Moved 2026-11-08** from `.git-exclude/tasks/dev-team/` into `rfcs/handoffs/`, at the owner's direction, and completed against the 17-item Developer Handoff structure. The authorization above is unchanged.

> **Added 2026-11-09 — wait for T070 (snora 0.52 → 0.57).** snora 0.54's
> `app_side_bar` shows each item's name on the rail (`SideBarLabels::Beneath`),
> its controls are keyboard-focusable from 0.55, and its active item clears
> 4.5:1. arama's navigation is three hand-built iced buttons with tooltips,
> which iced 0.14 cannot focus. **Whether to adopt snora's sidebar instead of
> showing labels on arama's own buttons is a design question the architect will
> settle in this handoff after T070 lands.** Do not start, or pause where you
> are; if you have work in progress, report what exists. The authorization above
> is unchanged.

> **DECIDED 2026-11-16 — the architect, as §2 below promised after T070.**
> T070 landed on 2026-11-15; this decision should have followed it, and did not
> until the dev team asked. **Start now.** The decision replaces §2, and
> amends §5 and §6. Everything else stands.
>
> **Adopt snora's own sidebar, names beneath the icons:** the `design`-path
> `app_side_bar_with(side_bar, LayoutDirection::Ltr, SideBarLabels::Beneath, …)`
> (`snora-widgets` 0.57.0, `src/design/widget.rs:208-230`, re-exported through
> `snora`), in place of the three hand-built `tooltip(button(…))` items in
> `app/src/core/view.rs`. `SideBar { items, active: self.nav_page }` with
> `NavPage` as the view id; each item's `tooltip` field carries
> `t("nav.explorer")` / `t("nav.cache")` / `t("nav.settings")` (it is the
> drawn name in this form, not a tooltip); `on_press: Message::NavTo(…)`.
>
> **Why this, and not the alternatives:**
> - **Labels on arama's own buttons** would rebuild, in arama, a component the
>   design system already solved and measured: naming, wrapping at a cap
>   (240 px, measured across snora's examples), active contrast (snora's
>   `sidebar_button_style_text_meets_aa`), RTL. And iced 0.14 cannot focus a
>   plain `button`, while snora's items are focus targets (`control`). Two rails
>   to maintain is not the final design.
> - **An expand/collapse rail** brings back the discovery problem this task
>   exists to remove (§2, §11), and adds a state to remember.
> - **`Beside`** puts each name next to its icon. That is wider than
>   `Beneath`, at a cost to the gallery on every screen. **`Tooltip`** is
>   today's behaviour.
> - `Beneath` keeps the rail narrow: an icon, with its name under it, the rail
>   only as wide as the longest name, wrapping at the cap.
>
> **Conditions:**
> 1. **Icons:** enable snora's `lucide-icons` feature on the workspace `snora`
>    line, and pass `Icon::Lucide(…)` for the same three glyphs. snora's
>    `lucide-icons` dependency is `"1"`, which resolves to the `lucide-icons`
>    1.21.0 already locked. **List every lockfile change. If any package other
>    than a feature edge moves, stop and report.**
> 2. **Keep RFC 044 as it is:** wrap the returned rail in the same
>    `container(…).style(arama_theme::focus_ring_style(self.is_zone_focused(FocusZone::SideBar)))`,
>    so F6 zone cycling and the zone ring keep working. **The RFC 044 tests pass
>    unmodified.** If one cannot, stop and report. **Tab routing stays out of
>    scope** (as in T070 §4).
> 3. **More than colour (§3, WCAG 1.4.1):** snora marks the active item with a
>    filled background. Report the contrast of that fill against the page
>    background, computed from each of arama's four presets' palettes, **at
>    least 3:1** (WCAG 1.4.11). If any preset falls short, stop and report; the
>    architect writes to snora.
> 4. **Targets:** snora's items are at least 48 × 48 (its own icon box), so §3's
>    44 × 44 holds. State the measured size.
> 5. **Tooltips:** removed. `Beneath` draws the name, and a tooltip repeating it
>    is noise.
> 6. **Evidence (amends §5), cheapest level first:** the T074 layout matrix,
>    all 72 combinations, every state, green with the new rail. That is the
>    proof the names fit at Larger, in both locales, at 760 × 520. Then **two
>    captures** through `scripts/gui-session.sh` (pre-seeded `settings.json`, no
>    clicks): the worst case, `ja` / Larger / 760 × 520, and its baseline, `en` /
>    Default / 1280 × 720. Before and after are not needed; the matrix is the
>    regression proof.
> 7. **Scope (amends §6):** `Cargo.toml` (one feature), `Cargo.lock` (if a
>    feature edge appears), `app/src/core/view.rs`, tests, and
>    `docs/src/users/using-arama.md`.
>
> **Rules from this session that apply here:** plants only in a scratch worktree
> with its own target directory; start an arama binary only through `cargo
> test`, `cargo run` with isolation, or `scripts/gui-session.sh` — never a file
> under `target/` by name or glob.

# T064 Handoff — show the navigation labels without hover

Companion to [RFC 052](../done/052-text-size-and-reach.md) §3.

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
