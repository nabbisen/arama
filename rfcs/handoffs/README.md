# RFC Implementation Handoffs

Developer-facing handoff documents, one per feature RFC. Each handoff
distils its RFC into three sections aimed at a developer picking up (or
reviewing, or regression-checking) the work:

1. **Implementation Handoff** — the goal, the key mechanics, and the
   non-obvious pitfalls to avoid.
2. **Task Breakdown / PR Plan** — a suggested decomposition into
   independently reviewable pull requests.
3. **Acceptance / QA Checklist** — automated and manual checks that
   define "done", reusable as a regression pass.

These are derived from the RFCs in [`../done/`](../done/) and are kept in
sync with them. They are **not** part of the numbered RFC lifecycle; they
are companion documents (like the migration notes in [`../notes/`](../notes/)).

RFC 000 (the RFC lifecycle policy) is a meta-policy, not implementable
feature work, so it has no handoff.

## Index

| RFC | Handoff | Shipped |
|----|---------|---------|
| 001 | [Migrate UI to snora](./001-migrate-ui-to-snora-handoff.md) | v0.22.0 |
| 002 | [Replace cache engine with localcache](./002-replace-cache-engine-with-localcache-handoff.md) | v0.23.0 |
| 003 | [Side-nav shell redesign](./003-side-nav-shell-handoff.md) | v0.24.0 |
| 004 | [Cache control page](./004-cache-control-page-handoff.md) | v0.25.0 |
| 005 | [Configurable threshold + ffmpeg re-download](./005-threshold-and-ffmpeg-redownload-handoff.md) | v0.26.0 |
| 006 | [i18n foundation](./006-i18n-foundation-handoff.md) | v0.27.0 |
| 007 | [i18n Phase 2 sweep](./007-i18n-phase2-handoff.md) | v0.28.0 |
| 008 | [Gallery filter, AI cleanup, error handling](./008-gallery-filter-cleanup-handoff.md) | v0.29.0 |
| 009 | [Replace custom DirTree with iced-swdir-tree](./009-iced-swdir-tree-handoff.md) | v0.30.0 |
| 010 | [Adopt the Snora Design system](./010-snora-design-system-handoff.md) | v0.32.0 |
| 011 | [Application theme setting](./011-theme-setting-handoff.md) | v0.33.0 |
| 012 | [Workspace housekeeping](./012-workspace-housekeeping-handoff.md) | v0.35.0 |
