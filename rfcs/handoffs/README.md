# RFC Implementation Handoffs

Developer-facing handoff documents for selected feature RFCs. A handoff is
added when implementation or closeout is complex enough to benefit from a
separate execution companion. Each handoff distils its RFC into sections aimed
at a developer picking up, reviewing, or regression-checking the work:

1. **Implementation Handoff** — the goal, the key mechanics, and the
   non-obvious pitfalls to avoid.
2. **Task Breakdown / PR Plan** — a suggested decomposition into
   independently reviewable pull requests.
3. **Acceptance / QA Checklist** — automated and manual checks that
   define "done", reusable as a regression pass.

These are derived from their corresponding RFCs and are kept in sync with
them. They are **not** part of the numbered RFC lifecycle; they are companion
documents (like the migration notes in [`../notes/`](../notes/)). Their status
is inherited from the corresponding RFC, so a handoff may accompany proposed
work when it is useful for implementation or closeout.

RFC 000 (the RFC lifecycle policy) is a meta-policy, not implementable
feature work, so it has no handoff.

## Index

| RFC | Handoff | RFC status / shipped |
|----|---------|----------------------|
| 001 | [Migrate UI to snora](./001-migrate-ui-to-snora-handoff.md) | 0.22.0 |
| 002 | [Replace cache engine with localcache](./002-replace-cache-engine-with-localcache-handoff.md) | 0.23.0 |
| 003 | [Side-nav shell redesign](./003-side-nav-shell-handoff.md) | 0.24.0 |
| 004 | [Cache control page](./004-cache-control-page-handoff.md) | 0.25.0 |
| 005 | [Configurable threshold + ffmpeg re-download](./005-threshold-and-ffmpeg-redownload-handoff.md) | 0.26.0 |
| 006 | [i18n foundation](./006-i18n-foundation-handoff.md) | 0.27.0 |
| 007 | [i18n Phase 2 sweep](./007-i18n-phase2-handoff.md) | 0.28.0 |
| 008 | [Gallery filter, AI cleanup, error handling](./008-gallery-filter-cleanup-handoff.md) | 0.29.0 |
| 009 | [Replace custom DirTree with iced-swdir-tree](./009-iced-swdir-tree-handoff.md) | 0.30.0 |
| 010 | [Adopt the Snora Design system](./010-snora-design-system-handoff.md) | 0.32.0 |
| 011 | [Application theme setting](./011-theme-setting-handoff.md) | 0.33.0 |
| 012 | [Workspace housekeeping](./012-workspace-housekeeping-handoff.md) | 0.35.0 |
| 013 | [ELOC splits](./013-eloc-splits-handoff.md) | 0.36.0 |
| 014 | [Explorer aside tree toggle](./014-aside-tree-toggle-handoff.md) | 0.36.1 |
| 032 | [Cross-platform external FFmpeg](./032-cross-platform-external-ffmpeg-handoff.md) | Proposed; closeout pending |
| 033 | [Cache dependency and Rust baseline](./033-cache-dependency-and-rust-baseline-handoff.md) | Proposed; accepted for implementation |
