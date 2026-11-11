# Developer Handoffs

**The architect's work orders to the dev team live here.** A handoff converts a
decision — an accepted RFC, or maintenance an accepted policy calls for — into
implementation-ready work, specific enough that the implementer does not have to
invent product or architecture decisions.

There are two kinds, told apart by their names:

- **`NNN-…-handoff.md` — an RFC companion**, keyed by RFC number. It distils one
  RFC for whoever implements, reviews or regression-checks it, and its status is
  inherited from that RFC.
- **`T0NN-…-handoff.md` — a task.** One independently reviewable unit of work.
  It may implement part of an RFC, or carry out maintenance an accepted RFC
  governs (advisory work under RFC 049 §6, for instance). The `T` prefix keeps
  task numbers from colliding with RFC numbers — the two series are independent.

**Every task handoff opens with an authorization banner**, recording the owner's
grant in their own words. Accepting an RFC does not authorize a task, and a task
without a grant in its banner is not to be started.

**Every task handoff covers the 17 items** the architect's operating
instructions require — purpose, background, applicable RFC, change and
non-change scope, required implementation, tests, documentation updates,
acceptance criteria, prohibited shortcuts, module boundaries, compatibility and
security constraints, known risks, required evidence, and the review-request
format — and closes with a checklist mapping each item to its section.

**Required evidence is an evidence plan** (from 2026-11-11): a table of the
questions the work must answer, the cheapest level each is checked at — logic
and state, headless UI, or the real UI — and the captures that remain. A capture
is the worst case, plus one baseline when something is compared. The plan is
stated before work starts; a question found later is added to it, with the
reason, before any capture it implies.

**References to `.git-exclude/…`** point at files in the working copy that are
not part of the repository: the local rules, the architect's schedule, and the
review records. They are written out so the dev team can find them; a public
reader will not.

RFC 000 (the RFC lifecycle policy) is a meta-policy, not implementable work, so
it has no handoff.

## Index

### RFC companions

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
| 032 | [Cross-platform external FFmpeg](./032-cross-platform-external-ffmpeg-handoff.md) | 0.37.0 |
| 033 | [Cache dependency and Rust baseline](./033-cache-dependency-and-rust-baseline-handoff.md) | 0.37.0 |
| 034 | [Release workflow reliability](./034-release-workflow-reliability-handoff.md) | 0.38.0 |
| 035 | [Similarity-dialog cache-error routing](./035-similarity-dialog-error-routing-handoff.md) | 0.38.0 |
| 036 | [Similarity-dialog absence states](./036-similarity-dialog-absence-states-handoff.md) | 0.39.0 |
| 037 | [Release publication atomicity](./037-release-publication-atomicity-handoff.md) | 0.39.0 |
| 038 | [Native smoke on CI runners](./038-native-smoke-on-ci-runners-handoff.md) | 0.39.1 |
| 039 | [Windows `PATH` search reachability](./039-windows-path-search-reachability-handoff.md) | 0.39.1 |
| 040 | [snora 0.29 upgrade and dialog surface](./040-snora-0.29-upgrade-and-dialog-surface-handoff.md) | 0.39.1 |
| 041 | [Application data locations](./041-application-data-locations-handoff.md) | 0.40.0 |
| 042 | [Windows Store distribution, phase one](./042-windows-store-distribution-handoff.md) | Proposed |
| 043 | [Typography roles and prose readability](./043-typography-roles-and-prose-readability-handoff.md) | 0.41.0 |
| 044 | [Keyboard operability](./044-keyboard-operability-handoff.md) | 0.41.0 |
| 045 | [Automated Store submission — Half A](./045-automated-store-submission-handoff.md) | Proposed; accepted 2026-08-25 |

### Tasks

| Task | Handoff | Governing document | Authorized |
|----|---------|--------------------|-----------|
| T061 | [Indexing that can actually be stopped](./T061-indexing-cancellation-handoff.md) | RFC 047 | 2026-10-21 |
| T063 | [Text-size setting: Default / Large / Larger](./T063-text-size-setting-handoff.md) | RFC 052 §2 | 2026-11-04 |
| T064 | [Visible navigation labels](./T064-visible-nav-labels-handoff.md) | RFC 052 §3 | 2026-11-04 |
| T066 | [Detect an ignore whose condition is already met](./T066-retirable-ignore-detection-handoff.md) | RFC 049 §4, §5, §8 | 2026-11-04 |
| T068 | [RUSTSEC-2026-0441, `pin-utils`: wasm32-only](./T068-pin-utils-wasm32-only-handoff.md) | RFC 049 | 2026-11-08 |
| T069 | [Thumbnail phase swallows an unopenable cache](./T069-thumbnail-phase-silent-cache-failure-handoff.md) | RFC 017 | 2026-11-08 |
| T070 | [snora 0.52 → 0.57](./T070-snora-0.57-handoff.md) | — (dependency maintenance; RFC 044, RFC 049 §3) | 2026-11-09 |
| T071 | [Model downloads refuse plain HTTP, redirects included](./T071-https-only-model-downloads-handoff.md) | RFC 021 | 2026-11-09 |

**Tasks written before 2026-11-08 are not here.** Until then they were kept in
the working copy's `.git-exclude/tasks/dev-team/` — numbered `001`–`047`, then
`T048` onward once the `T` prefix was introduced. On 2026-11-08 the owner
directed that dev-team work belongs in this directory. The completed tasks stay
where they were, as history; **the six above were still open**, so they were
moved here and completed against the 17-item structure.
