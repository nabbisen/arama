# Handoff — RFC 007: i18n Phase 2 sweep

**RFC.** [`rfcs/done/007-i18n-phase2.md`](../done/007-i18n-phase2.md)
**Shipped in.** v0.28.0
**Depends on.** RFC 006 (i18n foundation)

---

## 1. Implementation Handoff

### Goal
Complete the i18n string sweep RFC 006 deferred: replace remaining hardcoded
English literals with `t(key)` across six views, and ship the en/ja table
entries.

### Surfaces
- `setup/view.rs` — Download / Skip / no-space message
- `downloader/view.rs` — status labels, component names, disk-space text
- `media_focus_dialog/view.rs` — "Cache lookup strategy", "Close"
- `similar_pairs_dialog/view.rs` — "No valid pairs."
- `header/dir_nav/view.rs` — "Folder" picker button
- `gallery/view.rs` — "No file to render." empty state

### Bundled fixes
- Typo "No enough space" → "Not enough space" (now a translated key).
- `panic!("Unknown donwload config")` in `state_name()` →
  `eprintln!` + graceful fallback (removes a panic from a non-fatal UI path;
  also fixes the "donwload" typo).
- Japanese comments in `gallery/view.rs` → English (project convention).

### Watch out for
- `button(t("..."))` fails to compile — `t()` returns `String`, and iced
  `button` needs an `Element`; wrap in `text(t("..."))`.
- Japanese full-width punctuation in `ja.rs` must use brace unicode escapes
  (`\u{ff1a}`, `\u{ff08}`), not bare `\uff1a` — the bare form is an invalid
  Rust escape.

---

## 2. Task Breakdown / PR Plan

Single PR (cohesive sweep), or split table vs. call-sites:

1. Add ~20 keys to `crates/i18n/src/en.rs` and `ja.rs`.
2. Apply `t()` across the six view files.
3. Apply the three bundled fixes (typos, panic removal, comment language).
4. Build, verify zero warnings.

---

## 3. Acceptance / QA Checklist

### Automated
- [ ] `cargo check --workspace` — zero errors, zero warnings.
- [ ] `cargo check -p arama-i18n` — locale tables compile (catches the
      unicode-escape pitfall).
- [ ] `grep` confirms no remaining hardcoded English in the six target views.

### Manual — translation coverage (switch locale to 日本語 in Settings)
- [ ] Setup wizard: buttons and component names render in Japanese.
- [ ] Downloader: status labels and disk-space text render in Japanese.
- [ ] Focus dialog: "Cache lookup strategy" and "Close" translate.
- [ ] Similar-pairs dialog empty state translates.
- [ ] Header "Folder" button translates.
- [ ] Gallery empty state translates.
- [ ] Switching back to EN restores English with no restart.

### Regression
- [ ] Unknown keys still fall back gracefully (current → English → raw key).
- [ ] The setup downloader handles an unrecognised model config without
      panicking (logs via `eprintln!`, falls back to the CLIP label).
