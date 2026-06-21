# Handoff — RFC 001: Migrate the UI layer to the snora framework

**RFC.** [`rfcs/done/001-migrate-ui-to-snora.md`](../done/001-migrate-ui-to-snora.md)
**Shipped in.** v0.22.0

---

## 1. Implementation Handoff

### Goal
Replace arama's hand-rolled application chrome (layout composition, modal
overlay + dim backdrop, context-menu positioning) with snora v0.8's
`AppLayout` + `render`, and adopt snora toasts for the many
`// todo: error handling` sites.

### Key principle — re-host, don't rewrite
Every arama view component (header, aside, footer, gallery, dialogs) is
already a plain `Element`-producing function — exactly what snora's slots
accept. The migration moves these elements into snora slots; it does **not**
rewrite them.

### What snora replaces
- The bespoke `column![header, row![aside, body], footer]` skeleton →
  `AppLayout` slots.
- `crates/ui/widgets/src/dialog.rs` `overlay()` / `backdrop()` and the
  `stack!`-based layering in `view.rs` → snora `render`'s layered stack
  (skeleton → menu backdrop → menus → modal backdrop → dialog → toasts).
- Context-menu `space()` spacer positioning → snora's context-menu surface.

### What snora adds
Toasts (with logical-edge / RTL support aligning with the i18n requirement),
giving a real target for the error-handling todos (later realised in RFC 008).

---

## 2. Task Breakdown / PR Plan

### PR 1 — Adopt AppLayout skeleton
1. Add `snora = "0.8"`; build the `AppLayout` via its builder, feeding the
   existing header/aside/footer/body elements into slots.
2. Replace the manual `column!/row!/stack!` composition in `view.rs`.

### PR 2 — Overlays + toasts
3. Route dialog + context-menu through snora's overlay surfaces; delete the
   bespoke `overlay`/`backdrop` helpers.
4. Add `toasts` + `toast_position`; wire `toast::subscription` and
   `toast::sweep_expired`.

---

## 3. Acceptance / QA Checklist

### Automated
- [ ] `cargo check --workspace` — zero errors, zero warnings (watch for the
      now-deleted overlay/backdrop helpers leaving unused imports).

### Manual — layout parity
- [ ] Header, directory tree (aside), gallery, and footer render in the same
      positions as before.
- [ ] Opening a settings/focus/pairs dialog dims the backdrop and centers the
      modal; clicking the backdrop closes it.
- [ ] The context menu opens at the cursor and closes on outside click.

### Manual — toasts
- [ ] A toast appears, stacks if multiple, and auto-dismisses after its TTL
      (`toast::sweep_expired` via the subscription).
- [ ] Toast position matches `toast_position`.

### Regression
- [ ] No bespoke `overlay`/`backdrop` code remains in
      `crates/ui/widgets/src/dialog.rs`.
- [ ] All pre-existing view components behave identically (re-host, not
      rewrite).
