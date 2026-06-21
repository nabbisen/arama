# Handoff — RFC 010: Adopt the Snora Design system (token-driven button styling)

**RFC.** [`rfcs/done/010-snora-design-system.md`](../done/010-snora-design-system.md)
**Shipped in.** v0.32.0
**Migration note.** [`rfcs/notes/dep-migration-snora.md`](../notes/dep-migration-snora.md)

---

## 1. Implementation Handoff

### Goal
Update snora 0.18.1 → 0.25.0 and adopt its `design` feature so arama's
buttons use token-driven styles with WCAG AA-verified contrast, replacing
iced's built-in button styles.

### Why it's low-risk
- The version bump is drop-in: every snora symbol arama uses (`AppLayout` +
  builder, `Toast`, `ToastIntent`, `ToastPosition`, `render`,
  `toast::subscription`, `toast::sweep_expired`) is unchanged. The one
  breaking change in the range (`Palette::roles()` → test-only in 0.24.0) is
  not used by arama.
- `snora-design` is iced-free with **zero external dependencies**; enabling
  `design` adds the style-bridge code but no new crates.

### Key mechanic — the global-tokens pattern
snora's design style functions take `&Tokens` explicitly. Rather than thread
`&Tokens` through every `view()` signature in three UI crates, a new
`arama-theme` crate holds the tokens globally (consistent with how arama
handles i18n) and exposes **drop-in** style functions with iced's exact
`fn(&Theme, button::Status) -> button::Style` shape. Call sites change only
the function path: `button::primary` → `arama_theme::primary`.

> Note: RFC 010 used a write-once `OnceLock<Tokens>` fixed to
> `Tokens::light()`. RFC 011 later replaced this with a mutable preset
> global. A developer reading this handoff for historical context should be
> aware the `OnceLock` form is superseded.

### Style mapping
| was (iced built-in) | now (arama-theme) |
|---|---|
| `button::primary` | `arama_theme::primary` |
| `button::text` | `arama_theme::ghost` |
| `button::secondary` | `arama_theme::secondary` |
| `button::danger` | `arama_theme::danger` |

---

## 2. Task Breakdown / PR Plan

Recommend **two PRs**.

### PR 1 — Version bump + design feature (mechanical, low-risk)
1. Workspace `Cargo.toml`: `snora` 0.18 → 0.25, add
   `features = ["widgets", "design"]`.
2. `cargo update -p snora -p snora-core`; `cargo check --workspace`.
- **Acceptance:** builds clean against 0.25; no behaviour change yet
  (buttons still use iced built-ins until PR 2).

### PR 2 — arama-theme crate + button migration
3. New `crates/theme/` (manifest + `lib.rs` with the four style functions);
   register as workspace member + path dep.
4. Add `arama-theme` dep to `app`, `crates/ui/main`, `crates/ui/widgets`.
5. Swap the four call sites:
   - `app/src/core/view.rs` — nav rail (active = primary, inactive = ghost)
   - `crates/ui/widgets/.../general_settings/view.rs` — locale selector
   - `crates/ui/main/.../cache_page/view.rs` — stop button (danger)
   - `crates/ui/main/.../setup/view.rs` — skip button (secondary)
6. Docs: add `arama-theme` to `docs/src/dev/workspace.md`; addendum to the
   snora migration note.

---

## 3. Acceptance / QA Checklist

### Automated
- [ ] `cargo check --workspace` — zero errors, zero warnings.
- [ ] `cargo test -p arama-cache -p arama-i18n` — all pass (no test
      regressions from the snora bump).
- [ ] `arama-theme` compiles against `snora` with `design` enabled.

### Manual — visual
- [ ] Nav rail: the active page's icon button is accent-styled (primary);
      inactive ones are transparent (ghost). Hover/press states render.
- [ ] Locale selector buttons: active locale = primary, others = ghost.
- [ ] Cache page stop button (⏳ row) renders in the danger style.
- [ ] Setup wizard "Skip" button renders in the secondary style.
- [ ] Button label/foreground contrast is visibly adequate (the design
      tokens are contrast-verified; this is a sanity check, not a measurement).

### Regression
- [ ] Toasts still appear and auto-dismiss (snora bump did not change toast
      behaviour beyond the documented ordering fix).
- [ ] `AppLayout` skeleton (side-bar / body / footer / dialog overlay)
      renders unchanged.
- [ ] No new transitive crates beyond `snora-design` entered `Cargo.lock`.
