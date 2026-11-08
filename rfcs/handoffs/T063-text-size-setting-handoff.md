> **AUTHORIZED 2026-11-04 — owner:** *"Yes. Authorized. Proceed on the
> improvements and enhancement."* Written 2026-11-04 by the architect.
>
> **Implements [RFC 052](../proposed/052-text-size-and-reach.md) §2**,
> accepted the same day. Read the RFC first — §1 explains why the obvious
> approach produces a half-scaled interface, which is worse than no setting.
>
> **Independent of T061.** Different files. RFC 052 §4 is *deliberately* held
> behind T061 and is **not** part of this task — do not touch the Similar Pairs
> button's disabled styling.
>
> **Moved 2026-11-08** from `.git-exclude/tasks/dev-team/` into `rfcs/handoffs/`, at the owner's direction, and completed against the 17-item Developer Handoff structure. The authorization above is unchanged.

> **Added 2026-11-09 — wait for T070 (snora 0.52 → 0.57).** From snora 0.55,
> snora's icons and click targets grow with the text size (*"a control is at
> least max(24 px, 1.5 × its text, `Tokens::targets.min`)"*). This task's
> acceptance — the layout survives Larger, in both locales — measured on 0.52
> would not hold on 0.57. **Do not start, or pause where you are, until T070
> lands**; if you have work in progress, report what exists. The authorization
> above is unchanged.

# T063 Handoff — a text-size setting: Default / Large / Larger

Companion to [RFC 052](../proposed/052-text-size-and-reach.md) §2.

## 1. Why the obvious approach is wrong

Theme switching works at runtime through a global (`set_theme`,
`app/src/core/update/component.rs:312`). **Copying that pattern for text size
produces a broken result**, and the measurement is the reason:

| Route | Who reads it | Runtime? |
|---|---|---|
| `iced::Settings.default_text_size` | arama's own unannotated `text(...)` | **No** — startup only |
| `arama_theme::tokens()` | snora's chrome on the `design` path | Yes |

**118 of 120 non-test `text(` sites inherit `default_text_size`.** Only 2 carry
an explicit size. So scaling tokens at runtime resizes snora's chrome and two
call sites and leaves 118 at 16 px — a half-scaled interface, harder to read
than a small consistent one.

## 2. What to build — RFC 052 §2.1

**One `text_scale` setting, feeding both routes from one source, at startup.**
Scale the token set once; derive `iced::Settings.default_text_size`
(`app/src/core/settings.rs:16`) from the *scaled* `body` role, and hand snora
the same scaled tokens. Everything then moves together because everything
derives from one scaled set.

**Three named steps: Default / Large / Larger.** Not a free-form number — three
choices bound what the layout has to survive, and they match the precedent
snora's own 0.52 migration guide cites (orbok's Default / Large / Larger).

**Pick the factors and say why.** The RFC does not fix them. Whatever you
choose, state the resulting `body` px for each step in your package.

**Place it in Settings → General, beside theme and language.** New keys in
`crates/i18n/src/{en,ja}.rs`; the `Untranslated strings` CI job enforces both.

## 3. It needs a restart, and must say so — §2.2

`Settings` is read once at application start — RFC 043 §3.1's own note at
`app/src/core/settings.rs:9` says so.

**A setting that silently does nothing until restart is the confusion the
owner's second principle forbids.** So the control states plainly, in both
locales, that the change applies after restarting. Visible at the point of
choosing, not buried in a tooltip — tooltips are exactly what RFC 052 §3 is
removing elsewhere for being hover-gated.

## 4. The layout has to survive Larger — §2.3

**This is the part most likely to go wrong**, and we have the precedent: T055
fixed a footer that clipped its own text in Japanese because content exceeded
its allotted share. Scaling text in a window that does not grow is the same
cause.

**Check the narrow case at the largest step, in both locales**, and the footer
specifically. The objective check review 148 used still applies: ink in the
bottom rows of a capture should be 0 for a view with no modal.

## 5. Evidence — read the rules first

**the local GUI rules file (`.git-exclude/rules/project-instructions-gui-common.md` — in the working copy, not the repository) governs this**,
including its "GUI Evidence and Automation Rules". Prefer recorded state and
machine comparison over pictures; a picture only where appearance is the claim.

Here appearance **is** partly the claim — "nothing clips at Larger" is visual —
so captures are legitimate for §4. When you take them: fixed window size
obtained **at launch**, never by resizing a running window; isolated
`XDG_CONFIG_HOME` **and** `XDG_STATE_HOME`; one instance chosen by pid; state
the backend; abort at the first anomaly. **Report the window size and the file
dimensions separately** and say which is which.

For everything else prefer assertions — `iced_test::Simulator` is already used
in `app/src/core/tests.rs`, and "the setting round-trips through
`settings.json`" is a test, not a screenshot.

## 6. Change scope

- `env/src/config/settings.rs` — the new field, with `#[serde(default)]` so
  existing settings files load (the convention the other optional fields use)
- `app/src/core/settings.rs` — derive `default_text_size` from the scaled set
- `crates/theme/src/lib.rs` — the scaled token set
- the Settings → General tab under `crates/ui/widgets/src/dialog/settings_dialog/`
- `crates/i18n/src/en.rs`, `crates/i18n/src/ja.rs`

## 7. Non-change scope

- the Similar Pairs button's disabled styling — RFC 052 §4, held behind T061
- colour tokens or target sizes — RFC 052 §5 non-goals, both measured fine
- `CHANGELOG.md` or any `Cargo.toml` version field

## 8. Documentation updates

- **`docs/src/users/settings.md`** — describe the new control, its three steps, and that it takes effect after a restart.
- `CHANGELOG.md` is written at the release cut, not here.

## 9. Compatibility constraints

**Settings files must keep loading in both directions.**

- *Forward:* the new field takes `#[serde(default)]`, so an existing `settings.json` without it still loads — the convention the other optional fields already follow.
- *Backward:* `Settings` has **no `deny_unknown_fields`**, so serde ignores fields it does not recognise, and an **older** arama reads a newer file by skipping the new field. **Do not add `deny_unknown_fields`**, and confirm the 0.42.0 struct does not use it either — that is what a downgrading user runs.
- No cache, schema or payload change.

## 10. Security constraints

- No new external input: the setting is a three-value choice.
- **An unknown or out-of-range value on load must fall back to Default**, not error. A hand-edited `settings.json` must not stop arama from starting.

## 11. Known risks

- Scaling only the runtime token route leaves 118 of 120 `text(` sites at 16 px — a half-scaled interface (§1).
- Text clipping at Larger — T055's footer is the precedent (§4).
- A restart notice that is easy to miss makes the setting look broken (§3).
- Japanese strings are wider; check both locales at the largest step.
- T064's labels must also survive Larger; coordinate if both land close together.

## 12. Acceptance criteria

RFC 052 §6, as it applies to §2:

- Three steps in Settings → General, beside theme and language, in both locales.
- After a restart, **both** arama's own text and snora's chrome change, consistently — nothing left at the old size.
- The restart notice renders, visibly, in both locales.
- At the largest step, at a narrow window, in both locales: nothing clips, the footer in particular.
- The setting round-trips through `settings.json`.
- `cargo build`, `test`, `fmt --check` and `clippy -D warnings` clean; `Untranslated strings` passes.

## 13. Required deliverables

Review request with `## Entry point`. Include the chosen factors and the
resulting `body` px per step, the round-trip test, §4's narrow-window check at
Larger in both locales, and confirmation that the restart notice renders in
both.

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
| Applicable RFC and requirements | Authorization banner and companion line, §2 |
| Change scope | §6 |
| Explicit non-change scope | §7 |
| Required implementation | §2, §3 |
| Required tests | §5 |
| Required documentation updates | §8 |
| Acceptance criteria | §12 |
| Prohibited shortcuts | §1, §7 |
| Relevant code or module boundaries | §6 |
| Compatibility constraints | §9 |
| Security constraints | §10 |
| Known risks | §11 |
| Required evidence | §4, §5 |
| Required review-request format | §13 |
