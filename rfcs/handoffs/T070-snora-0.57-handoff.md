> **NOT AUTHORIZED YET.** Written 2026-11-09 by the architect. Do not start
> until this banner is replaced by the owner's authorization.
>
> **Order, once authorized:** after T061's rework lands (both edit
> `app/src/core/view.rs`), and **before T063 and T064 continue** — snora 0.55
> changes how snora's icons and targets follow the text size (T063's subject),
> and 0.54 gives `app_side_bar` visible labels (T064's subject). Verifying
> either task on 0.52 and then upgrading would invalidate its evidence.

# T070 Handoff — snora 0.52 → 0.57

No RFC. Dependency maintenance, the same kind as T052 (snora 0.42 → 0.52).
Constrained by [RFC 044](../done/044-keyboard-operability.md) (keyboard), which
this upgrade must not regress, and by
[RFC 049](../proposed/049-advisory-maintenance-policy.md) §3 (the audit
ignore list), which §2.6 touches.

## 1. Purpose

Move arama from snora 0.52.0 to **0.57.0**, the current release, so that:

- the removed and deprecated APIs arama calls are replaced while the
  replacement is a one-line change, not after 0.60 removes the second one;
- T063 and T064 are built and verified against the snora they will ship with.

**Do not stop at 0.55.0 or 0.56.0.** 0.55.0 gave every snora control keyboard
focus on a *click*: a focus ring after every press, and Space or Enter pressing
the clicked control again. 0.56.0 fixed it. Go straight to 0.57.0.

## 2. Background, and what to do

snora published one migration guide per minor. **Read all five, in order,
before editing anything** — each opens with a *Who is affected* table; follow
only the rows that match arama:

- <https://nabbisen.github.io/snora/guides/migration-0.52-to-0.53.html>
- <https://nabbisen.github.io/snora/guides/migration-0.53-to-0.54.html>
- <https://nabbisen.github.io/snora/guides/migration-0.54-to-0.55.html>
- <https://nabbisen.github.io/snora/guides/migration-0.55-to-0.56.html>
- <https://nabbisen.github.io/snora/guides/migration-0.56-to-0.57.html>

The architect has matched them against arama's source. What follows is that
match; **confirm each row yourself** — a table in a handoff is a claim until
reproduced.

### 2.1 The version line

`Cargo.toml` (workspace): `snora = { version = "0.52", … }` → `"0.57"`, features
unchanged (`default-features = false, features = ["design"]`).

**Since 0.54, `default-features = false` on snora also turns off iced's
defaults through snora.** arama's own `iced` line keeps its defaults, so this
does not break the build — the guide's one failing case needs both. **Do not
change the `iced` line.**

### 2.2 Toasts — removed API, and a new hold

`snora::toast::sweep_expired` is **removed in 0.57**. arama calls it once, in
`handle_toast_sweep` (`app/src/core/update/ui.rs`).

- Add `holds: snora::toast::Holds` to `App`; the sweep becomes
  `self.holds.sweep(&mut self.toasts, Instant::now())`.
- **Wire the attention hold** — the end state, not the deprecated one:
  `on_toast_attention(Message::ToastAttention)` on the layout, and
  `Message::ToastAttention(id, attention) => self.holds.attend(id, attention,
  &mut self.toasts, now)`. A toast then waits while the pointer rests on it or
  the keyboard is on its ×. **Not** `on_toast_hover` / `ToastHover` /
  `Holds::hover`: those are deprecated in 0.56 and removed in 0.59.
- `ToastLifetime::DEFAULT` is also removed; arama does not use it. Confirm.

**Behaviour that changes with no code change:** `Toast::new` with
`ToastIntent::Error` is now **persistent** — it stays until dismissed — and
other toasts stay as long as their text takes to read (≥ 6 s, was 4 s). That
is right for arama's readers. **Audit every `ToastIntent::Error` site** (they
go through `push_error_toast` / `push_toast` in `app/src/core.rs`, plus the
startup notices) **for any that can fire repeatedly within one operation** — a
per-file loop, a retry. Before 0.54 a burst expired; now it piles up and must be
dismissed one by one. **List what you find. If any such path exists, stop and
report it** rather than redesigning it here.

### 2.3 Escape — deprecated API that already does nothing

`snora::keyboard::dismiss_on_escape` is **deprecated in 0.57** (removed in
0.60). The workspace builds clippy with `-D warnings`, so it must go.

**From 0.55, snora's engine captures Escape** when `on_close_modals` /
`on_close_menus` are set, and arama's `iced::keyboard::listen()` no longer
receives it. `app/src/core/view.rs` sets both. So the call in
`handle_key_pressed` (`app/src/core/update/keyboard.rs`) is dead code on 0.57:
**remove it**, and keep the zone cycling after it unchanged.

- Do **not** add `AppLayout::escape(...)` unless you find an overlay that must
  close on Escape but **not** on a click outside. Today there is none; if you
  find one, say which.
- The four tests in `app/src/core/update/keyboard.rs` that call
  `dismiss_on_escape` directly test snora's helper, not arama. Remove them with
  the helper.
- **`tier_2_escape_closes_a_real_open_dialog`** (`app/src/core/tests.rs`) sends
  `Message::KeyPressed(Escape)` into `App::update`. **Its premise is what 0.55
  changed:** on 0.57 that message never arrives while an overlay is open — the
  engine takes the key first — so once the helper is gone the test fails, and
  it should. **Re-point it, do not delete it:** press Escape through
  `iced_test::Simulator` on the rendered view with a dialog open, assert the
  close message (`Message::DialogClose`) is produced, and add the same for an
  open context menu (`Message::CloseMenus`). Keep its first half — Escape with
  nothing open closes nothing. If the Simulator cannot deliver a key to snora's
  engine, **stop and report** what you tried; do not fall back to calling
  `App::update` with the message directly, which proves nothing about Escape.

### 2.4 Keyboard — no regression, and no new scope

RFC 044's F6 zone cycling and focus ring (`snora::keyboard::cycle_zones`, the
`tier_2_*` and `phase_0_*` tests in `app/src/core/tests.rs`) **must keep working
unchanged** — every one of those tests passes unmodified, **with two
exceptions whose premise snora changed:** `tier_2_escape_closes_a_real_open_dialog`
(§2.3), and `phase_0_1_keyboard_baseline_on_gallery_and_settings`, which records
which keys reach arama unclaimed — Escape, and possibly Enter or Space on a
focused snora control, may now be claimed. For that one, **report the before and
after result per key**, and change its expectations only where a guide row
explains the change; name the row. Several of those tests reach into how snora identifies regions
(`tier_2_focus_ring_zones_resolve_to_snoras_own_region_ids`,
`is_focused_selector_cannot_see_aramas_zone_ring_containers`). 0.55 moved
snora's focus machinery into a new crate (`snora-interaction`, named
`snora-control` in 0.55–0.56). **If one of those tests fails, stop and report
— do not rewrite its assertions to pass.** A changed assertion there is a
keyboard-accessibility decision, not a migration step.

### 2.5 Appearance — what moves, and how to evidence it

On arama's path (the `design` feature, the four presets in
`crates/theme/src/lib.rs`):

| Change | Version | Expected on arama |
|---|---|---|
| Modal dim: light veil on dark themes, slightly darker on light | 0.53 | Visible with a dialog open |
| Ancestor breadcrumbs underlined, row +1 px | 0.53 | None — arama renders no breadcrumbs (no `crumb` in its source). Confirm |
| Toast × 15.75 px (was 18), target unchanged | 0.55 | Visible on any toast |
| Toast text colours | 0.53 | Unchanged on arama's presets — the guide pins stock and preset colours |

**Evidence follows the GUI rules** (`.git-exclude/rules/project-instructions-gui-common.md`
— a working-copy file, not in the repository): prefer recorded state; capture
only where appearance is the claim. Here it is, for two pictures: a dialog open
(dim) and a toast visible, on Light and on Dark, before and after, same backend.
Fixed window size at launch, isolated `XDG_CONFIG_HOME` and `XDG_STATE_HOME`,
one instance by pid, `niri msg action screenshot-window` with an absolute path.
**Abort at the first anomaly.**

### 2.6 One ignore this upgrade can retire — as its own commit

snora's 0.53 → 0.54 guide (§5) notes that `wasmtimer` 0.4.5 no longer depends
on `pin-utils`. arama's lockfile has `wasmtimer` 0.4.3 — and
`cargo update -p wasmtimer --dry-run` today reports:

```
Removing pin-utils v0.1.0
Updating wasmtimer v0.4.3 -> v0.4.5
```

So **`RUSTSEC-2026-0441`, ignored by T068 on 2026-11-08, is retirable by a
lockfile update.** The architect accepted that ignore without checking whether
an update removed the crate — exactly the case RFC 049 §4's detector (T066)
exists to catch. Under RFC 049 the retirement is:

- `cargo update -p wasmtimer`;
- remove `RUSTSEC-2026-0441` from `.cargo/audit.toml`;
- move its ledger entry in `rfcs/notes/audit-warning-burn-down.md` to retired,
  dated, with the commit;

**in one commit of its own, before the snora commit**, so each lockfile change
is attributable. `cargo audit --deny warnings` must still exit 0 afterwards.

## 3. Change scope

- `Cargo.toml` (workspace, the `snora` line)
- `Cargo.lock`
- `app/src/core.rs` (`App` gains `holds`)
- `app/src/core/message.rs` (`ToastAttention`)
- `app/src/core/update.rs` (dispatch)
- `app/src/core/update/ui.rs` (sweep, attend)
- `app/src/core/update/keyboard.rs` (Escape helper removed, tests replaced)
- `app/src/core/view.rs` (`on_toast_attention`)
- `app/src/core/tests.rs` (Escape through the real path)
- `.cargo/audit.toml` and `rfcs/notes/audit-warning-burn-down.md` (§2.6 only)
- documentation named in §6

## 4. Non-change scope

- **Tab routing** (`snora::keyboard::traverse`). It would make snora's controls
  reachable and leave arama's own iced buttons — the navigation among them —
  unreachable: half a Tab order, which is worse than a consistent one. That is
  an RFC decision, not a migration step.
- **Shortcuts** (`AppLayout::shortcuts`, 0.57).
- **`app_side_bar` / `SideBarLabels`** for arama's navigation — T064's question,
  decided after this lands.
- **The AAA presets** (`Tokens::light_aaa`, `dark_aaa`) and **`Tokens::targets`**
  — RFC 052's territory.
- **`design::button::link`**.
- The `iced` line, iced's feature set, and the zbus advisories behind
  `linux-theme-detection`.
- `CHANGELOG.md` and every `Cargo.toml` `version` field.

## 5. Required tests

- Every existing test passes; **the RFC 044 keyboard tests unmodified**, except
  the two §2.4 names, each changed as §2.3 and §2.4 say.
- Escape closes a dialog and a menu, pressed through the Simulator (§2.3).
- The toast hold: a test that a toast with attention held survives a sweep
  past its lifetime, and expires after attention ends — through
  `App::update` with `Message::ToastAttention`, not by calling `Holds` directly.
- Gates: `cargo build`, `cargo test`, `cargo fmt --check`,
  `cargo clippy --all-targets -- -D warnings` (all `--workspace --locked`),
  and `cargo audit --deny warnings`.

## 6. Documentation updates

- `docs/src/` — any page that states how long a toast stays or that error
  toasts disappear on their own. Search for it; list what you changed, or say
  none matched.
- Wherever `docs/src/users/` describes Escape (start with
  `docs/src/users/using-arama.md`): Escape still closes dialogs and menus —
  confirm the text still matches, change nothing that does.
- `rfcs/notes/audit-warning-burn-down.md` (§2.6).

## 7. Module boundaries

- arama depends on **`snora` only**. Do not add `snora-interaction`,
  `snora-widgets`, `snora-core` or any other snora crate to any `Cargo.toml`.
- The toast hold lives in `App` beside `toasts`; no new module.

## 8. Compatibility constraints

- **No settings, cache or payload change.** A user's settings file and cache
  are untouched.
- **MSRV:** snora 0.57.0 declares `rust-version = 1.88`, below the workspace's.
  The `MSRV` workflow must stay green.
- **Lockfile:** list every package that moves. Expect the `snora-*` crates to
  0.57.0 and a new `snora-interaction`; anything else, explain or stop.

## 9. Security constraints

- No new network, filesystem or subprocess surface.
- `cargo audit --deny warnings` exits 0 after each commit.
- §2.6 removes an ignore: the ledger records why, with the commit, per RFC 049.

## 10. Known risks

- **Fresh releases.** 0.54 through 0.57 were published within two days
  (2026-11-07 and 2026-11-08), and 0.55.0 needed a correction within a day.
  If you find a defect in 0.57.0, stop and report it; the architect writes to
  snora.
- **RFC 044 tests coupled to snora internals** (§2.4).
- **Escape now handled by the engine** — any arama code that reacted to Escape
  for something else while an overlay is open no longer sees it. Search for
  `Named::Escape` outside the removed helper and report what you find.
- **Persistent error toasts piling up** (§2.2).
- **T061 rework and this task both edit `app/src/core/view.rs`.** T061 goes
  first.

## 11. Acceptance criteria

- `snora` at 0.57.0 in `Cargo.lock`; no 0.55.x or 0.56.x anywhere.
- No call to `sweep_expired`, `dismiss_on_escape`, `on_toast_hover`,
  `ToastHover` or `Holds::hover` remains.
- The toast attention hold works through `App::update` (§5).
- Escape closes dialogs and menus, pressed through the Simulator (§2.3).
- RFC 044's keyboard tests pass unmodified, except the two named in §2.4,
  changed only as §2.3 and §2.4 allow, each change tied to a guide row.
- The error-toast audit (§2.2) is reported.
- `RUSTSEC-2026-0441` retired in its own commit (§2.6); `cargo audit --deny
  warnings` exits 0.
- All gates green.

## 12. Prohibited shortcuts

- `#[allow(deprecated)]` anywhere, to keep a deprecated snora call.
- Stopping at 0.55.x or 0.56.x.
- Editing an RFC 044 test's assertions to make it pass, beyond the two §2.4
  names — and those only as §2.3 and §2.4 say.
- Testing Escape by sending `Message::KeyPressed(Escape)` or `Message::DialogClose`
  straight into `App::update`.
- Wiring `on_toast_hover` instead of `on_toast_attention`.
- Depending on a snora sub-crate directly.
- Folding §2.6 into the snora commit.

## 13. Required evidence

- The lockfile diff summary per commit (packages moved, added, removed).
- The output of each gate, observed.
- The two before/after capture pairs (§2.5), with backend and window size.
- The error-toast audit (§2.2) and the `Named::Escape` search (§10).
- Your reproduction of §2's matching — which guide rows applied, and which did
  not.

## 14. Review-request format

A package under `.git-exclude/review-request/` (working copy, not in the
repository) with a `## Entry point` section: the commit(s), the `git show`
commands, and full repo-relative paths. Report observed output; a check not run
is recorded as not run. You cannot push: report the commits, and the architect
pushes and watches every workflow on that SHA — **the Windows job included**.

## 15. Handoff checklist

The 17 items the architect's operating instructions require of every Developer
Handoff, and where each is covered here.

| Required item | Covered in |
|---|---|
| Task title | Heading |
| Purpose | §1 |
| Background | §2 |
| Applicable RFC and requirements | Companion line (RFC 044, RFC 049), §2.4, §2.6 |
| Change scope | §3 |
| Explicit non-change scope | §4 |
| Required implementation | §2.1–§2.6 |
| Required tests | §5 |
| Required documentation updates | §6 |
| Acceptance criteria | §11 |
| Prohibited shortcuts | §12 |
| Relevant code or module boundaries | §3, §7 |
| Compatibility constraints | §8 |
| Security constraints | §9 |
| Known risks | §10 |
| Required evidence | §13 |
| Required review-request format | §14 |
