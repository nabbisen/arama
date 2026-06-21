# Migration report: snora 0.8.0 → 0.18.0

**Verdict: drop-in for arama. No source changes required.**

## Release summary (0.8 → 0.18, ten minor versions)

| Version | Key changes |
|---|---|
| 0.8.0 | mdBook docs, GitHub conventions |
| 0.9.0 | Doctest coverage in `snora-core` |
| 0.10.0 | Binary-size budget tracking |
| 0.11.0 | CI workflow; render-semantics test harness; `AppLayout` marked `#[non_exhaustive]`; toast ordering **fixed** |
| 0.12.0 | ABDD compliance checklist; workbench example; compile-time tracking |
| 0.13.0 | Anchored popover design study; API freeze review |
| 0.14.0 | `snora::keyboard::dismiss_on_escape` helper (new) |
| 0.15.0 | Starter application example; versioning policy; migration guide template |
| 0.16.0 | Performance envelope reference; alternate-engine boundary doc |
| 0.17.0 | `Icon` implements `PartialEq`; two RTL render-semantics tests; build-cost data |
| 0.18.0 | Contributing overview; API freeze review updated (7 of 10 1.0 gates satisfied) |

## API changes that affect arama

### Breaking: `AppLayout` is now `#[non_exhaustive]` (0.11.0)

Struct-literal construction (`AppLayout { body, side_bar, ... }`) is
no longer permitted outside `snora-core`.

**arama is unaffected.** The app constructs `AppLayout` exclusively
via the builder chain:

```rust
AppLayout::new(body)
    .side_bar(side_bar)
    .footer(footer)
    .on_close_menus(Message::CloseMenus)
    .on_close_modals(Message::DialogClose)
    .toasts(self.toasts.clone())
    .toast_position(ToastPosition::BottomEnd)
```

All builder methods present in 0.8.0 are present with identical
signatures in 0.18.0.

### Behavioral: toast ordering fix (0.11.0)

The newest toast now appears closest to the configured anchor edge
(previously inverted due to an `is_bottom()` vs `is_top()` predicate
bug). With `ToastPosition::BottomEnd`, the newest toast now appears
at the bottom of the stack rather than the top.

This is a bug fix. The visual stacking order changes, but the API
(`toasts`, `toast_position`) is unchanged.

### Additive (no migration needed)

- `snora::keyboard::dismiss_on_escape` — new helper (0.14.0).
- `Icon: PartialEq` — new trait impl (0.17.0).
- `Tab`, `TabBar`, `TabAction`, `Crumb`, `BreadcrumbAction`,
  `app_tab_bar`, `app_breadcrumb` — new navigation widgets (0.7.0,
  already past arama's 0.8.0 baseline — this is a reminder that the
  tab bar and breadcrumb widgets are available if arama ever needs them).

## Symbols arama uses — 0.18.0 verification

| Symbol | Present in 0.18.0 | Notes |
|---|---|---|
| `AppLayout` | ✓ | `#[non_exhaustive]`; builder unchanged |
| `AppLayout::new` | ✓ | |
| `.side_bar()` | ✓ | |
| `.footer()` | ✓ | |
| `.on_close_menus()` | ✓ | |
| `.on_close_modals()` | ✓ | |
| `.toasts()` | ✓ | |
| `.toast_position()` | ✓ | |
| `.context_menu()` | ✓ | |
| `Dialog` | ✓ | |
| `Toast` | ✓ | |
| `ToastIntent` | ✓ | |
| `ToastPosition` | ✓ | |
| `render` | ✓ | |
| `toast::subscription` | ✓ | |
| `toast::sweep_expired` | ✓ | |

## How to apply

In `Cargo.toml` (workspace):

```toml
# was:
snora = { version = "0.8", default-features = false }

# becomes:
snora = { version = "0.18", default-features = false }
```

Then `cargo update -p snora -p snora-core`.
No source changes required.


---

## Addendum: 0.18.1 → 0.25.0 (RFC 010)

Updated again when adopting the Snora Design system. The 0.18 → 0.25
range adds the design system across versions 0.19–0.25:

| Version | Design-system milestone |
|---|---|
| 0.19.0 | `snora-design` token crate introduced (groundwork, `publish = false`) |
| 0.20.0 | `snora-design` published; `design` feature live (opt-in); pilot button + card helpers |
| 0.21.0 | Notice, chip, progress primitives |
| 0.22.0 | Chip style refactor + tests |
| 0.23.0 | Four design recipes |
| 0.24.0 | **Breaking:** `Palette::roles()` → test-only; chip selected-state contrast fixed to WCAG AA |
| 0.25.0 | Measurement methodology fixes; size-probe crates |

**Non-design API:** unchanged across the whole range. Every symbol arama
uses from the snora facade (`AppLayout` + builder, `Toast`, `ToastIntent`,
`ToastPosition`, `render`, `toast::subscription`, `toast::sweep_expired`)
is present and signature-identical in 0.25.0.

**Only breaking change (0.24.0):** `Palette::roles()` made `#[cfg(test)]
pub(crate)`. arama does not use `Palette::roles()` — unaffected.

**Design feature cost:** `snora-design` is iced-free with **zero external
dependencies**; the `design` feature adds the style-bridge code but no new
crates. Build-cost and binary-size impact are minimal.

arama enables `features = ["widgets", "design"]` and consumes the design
button style functions via the new `arama-theme` crate. See RFC 010.
