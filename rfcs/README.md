# arama RFCs

Design documents for arama, managed under the lifecycle policy
defined in [RFC 000](./done/000-rfc-lifecycle-policy.md):
folders are the source of truth for state; numbers are
permanent; implemented and archived RFCs are never deleted.

## Proposed

| ID | Title | Priority |
|----|-------|----------|
| — | (none currently) | |

## Implemented

| ID | Title | Shipped in |
|----|-------|------------|
| 000 | [RFC lifecycle policy](./done/000-rfc-lifecycle-policy.md) | adopted with this directory |
| 001 | [Migrate the UI layer to the snora framework (v0.8)](./done/001-migrate-ui-to-snora.md) | v0.22.0 |
| 002 | [Replace the in-house cache engine with localcache](./done/002-replace-cache-engine-with-localcache.md) | v0.23.0 |
| 003 | [Side-nav shell redesign](./done/003-side-nav-shell.md) | v0.24.0 |
| 004 | [Cache control page](./done/004-cache-control-page.md) | v0.25.0 |
| 005 | [Configurable similarity threshold + ffmpeg re-download](./done/005-threshold-and-ffmpeg-redownload.md) | v0.26.0 |
| 006 | [Multilingual GUI — i18n foundation](./done/006-i18n-foundation.md) | v0.27.0 |
| 007 | [i18n Phase 2 sweep](./done/007-i18n-phase2.md) | v0.28.0 |
| 008 | [Gallery filter, AI debug cleanup, error handling](./done/008-gallery-filter-cleanup.md) | v0.29.0 |

## Archive

| ID | Title | Reason |
|----|-------|--------|
| — | (none yet) | |

## Notes

One-off investigation records and decision notes that are not design
proposals. They do not go through the proposed → implemented lifecycle
and are not numbered, but are kept here as permanent project records.

| File | Subject |
|------|---------|
| [dep-migration-lucide-icons](./notes/dep-migration-lucide-icons.md) | lucide-icons 0.576 → 1.17: API diff and safe-to-update confirmation |
| [dep-migration-candle](./notes/dep-migration-candle.md) | candle-{core,nn,transformers} 0.9 → 0.10: symbol audit and safe-to-update confirmation |
