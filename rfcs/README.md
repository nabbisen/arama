# arama RFCs

Design documents for arama, managed under the lifecycle policy
defined in [RFC 000](./done/000-rfc-lifecycle-policy.md):
folders are the source of truth for state; numbers are
permanent; implemented and archived RFCs are never deleted.

## Proposed

| ID | Title | Priority |
|----|-------|----------|

*None currently proposed.*

## Implemented

| ID | Title | Shipped in |
|----|-------|------------|
| 000 | [RFC lifecycle policy](./done/000-rfc-lifecycle-policy.md) | adopted with this directory |
| 001 | [Migrate the UI layer to the snora framework (v0.8)](./done/001-migrate-ui-to-snora.md) | 0.22.0 |
| 002 | [Replace the in-house cache engine with localcache](./done/002-replace-cache-engine-with-localcache.md) | 0.23.0 |
| 003 | [Side-nav shell redesign](./done/003-side-nav-shell.md) | 0.24.0 |
| 004 | [Cache control page](./done/004-cache-control-page.md) | 0.25.0 |
| 005 | [Configurable similarity threshold + ffmpeg re-download](./done/005-threshold-and-ffmpeg-redownload.md) | 0.26.0 |
| 006 | [Multilingual GUI — i18n foundation](./done/006-i18n-foundation.md) | 0.27.0 |
| 007 | [i18n Phase 2 sweep](./done/007-i18n-phase2.md) | 0.28.0 |
| 008 | [Gallery filter, AI debug cleanup, error handling](./done/008-gallery-filter-cleanup.md) | 0.29.0 |
| 009 | [Replace custom DirTree with iced-swdir-tree](./done/009-iced-swdir-tree.md) | 0.30.0 |
| 010 | [Adopt the Snora Design system (token-driven button styling)](./done/010-snora-design-system.md) | 0.32.0 |
| 011 | [Application theme setting (light / dark / high-contrast)](./done/011-theme-setting.md) | 0.33.0 |
| 012 | [Workspace housekeeping (manifest inheritance, orphan removal, changelog & doc reconciliation)](./done/012-workspace-housekeeping.md) | 0.35.0 |
| 013 | [ELOC splits: update.rs and cache integration tests](./done/013-eloc-splits.md) | 0.36.0 |
| 014 | [Explorer aside tree toggle](./done/014-aside-tree-toggle.md) | 0.36.1 |
| 015 | [Cache lifecycle: retire v1 migration and define cache-capacity direction](./done/015-cache-lifecycle.md) | 0.37.0 |
| 016 | [Cache capacity and disk-pressure management](./done/016-cache-capacity.md) | 0.37.0 |
| 017 | [Visible recoverable error UX](./done/017-visible-recoverable-error-ux.md) | 0.37.0 |
| 018 | [AI/video pipeline resilience](./done/018-ai-video-pipeline-resilience.md) | 0.37.0 |
| 019 | [Startup fatal-boundary resilience](./done/019-startup-fatal-boundary-resilience.md) | 0.37.0 |
| 020 | [Dependency modernization: Candle and sidecar archive stack](./done/020-dependency-modernization.md) | 0.37.0 |
| 021 | [CLIP SafeTensors source strategy](./done/021-clip-safetensors-source-strategy.md) | 0.37.0 |
| 022 | [Image similarity search dependency strategy](./done/022-image-similarity-search-dependency.md) | 0.37.0 |
| 023 | [Cache serialization dependency strategy](./done/023-cache-serialization-dependency.md) | 0.37.0 |
| 024 | [Image codec dependency minimization](./done/024-image-codec-dependency-minimization.md) | 0.37.0 |
| 025 | [Release smoke checklist](./done/025-release-smoke-checklist.md) | 0.37.0 |
| 026 | [Explorer tree maintenance and scan ownership](./done/026-explorer-tree-maintenance.md) | 0.37.0 |
| 027 | [Audit warning ledger refresh](./done/027-audit-warning-ledger-refresh.md) | 0.37.0 |
| 028 | [Source TODO hygiene and orphan cleanup](./done/028-source-todo-hygiene.md) | 0.37.0 |
| 029 | [Release smoke evidence template](./done/029-release-smoke-evidence-template.md) | 0.37.0 |
| 030 | [Distribution and version contract reconciliation](./done/030-distribution-and-version-contracts.md) | 0.37.0 |
| 032 | [Cross-platform external FFmpeg](./done/032-cross-platform-external-ffmpeg.md) | 0.37.0 |
| 033 | [Cache dependency correction and Rust source-build baseline](./done/033-cache-dependency-and-rust-baseline.md) | 0.37.0 |
| 034 | [Release workflow reliability](./done/034-release-workflow-reliability.md) | 0.38.0 |
| 035 | [Similarity-dialog cache-error routing](./done/035-similarity-dialog-error-routing.md) | 0.38.0 |
| 036 | [Similarity-dialog absence states](./done/036-similarity-dialog-absence-states.md) | 0.39.0 |
| 037 | [Release publication atomicity](./done/037-release-publication-atomicity.md) | 0.39.0 |
| 038 | [Native smoke on CI runners](./done/038-native-smoke-on-ci-runners.md) | 0.39.1 |
| 039 | [Windows `PATH` search reachability](./done/039-windows-path-search-reachability.md) | 0.39.1 |
| 040 | [snora 0.29 upgrade and dialog surface](./done/040-snora-0.29-upgrade-and-dialog-surface.md) | 0.39.1 |

## Archive

| ID | Title | Reason |
|----|-------|--------|
| 031 | [macOS ffmpeg trust boundary](./archive/031-macos-ffmpeg-trust-boundary.md) | Superseded by RFC 032 |

## Notes

One-off investigation records and decision notes that are not design
proposals. They do not go through the proposed → implemented lifecycle
and are not numbered, but are kept here as permanent project records.

| File | Subject |
|------|---------|
| [dep-migration-lucide-icons](./notes/dep-migration-lucide-icons.md) | lucide-icons 0.576 → 1.17: API diff and safe-to-update confirmation |
| [dep-migration-candle](./notes/dep-migration-candle.md) | candle-{core,nn,transformers} 0.9 → 0.10: symbol audit and safe-to-update confirmation |
| [dep-migration-snora](./notes/dep-migration-snora.md) | snora 0.8 → 0.18: API diff across ten minor versions, safe-to-update confirmation |
| [snora-recipe-theme-custom](./notes/snora-recipe-theme-custom.md) | `Theme::custom` from Snora Design tokens, written in snora's own RFC-033 recipe format (contribution to snora; unrelated to arama RFC 033) |
| [dep-fix-pt2safetensors](./notes/dep-fix-pt2safetensors.md) | `pt2safetensors` 0.1.2 build break against candle-core 0.10 + safetensors ≥ 0.5; root cause, workspace patch, and upstream fix instructions |
| [audit-warning-burn-down](./notes/audit-warning-burn-down.md) | RustSec warning burn-down: patched actionable lockfile warnings and recorded remaining transitive owners |
| [clip-safetensors-source-decision](./notes/clip-safetensors-source-decision.md) | RFC 021 implementation decision: retain CLIP runtime conversion until a trustworthy pinned SafeTensors source exists |
| [cache-serialization-dependency-decision](./notes/cache-serialization-dependency-decision.md) | RFC 023 implementation decision: retain current localcache/bincode cache serialization until a bincode-free localcache route exists |
| [native-smoke-risk-acceptance](./notes/native-smoke-risk-acceptance.md) | RFC 032 release checkpoint: owner risk acceptance for unexecuted Windows/macOS native smoke, with the specific residual risks and the Linux evidence that was executed |
| [snora-dialog-overlay-card](./notes/snora-dialog-overlay-card.md) | Upstream report, sent 2026-08-10: snora 0.25's dialog overlay documents "the centered modal card" but draws no card, so dialog text is legible only where it lands on neutral background (contribution to snora) |


## Handoffs

Per-RFC developer handoff documents (Implementation Handoff, Task
Breakdown / PR Plan, Acceptance / QA Checklist) live under
[`handoffs/`](./handoffs/). Like the notes, they are companion documents
outside the numbered RFC lifecycle. See [`handoffs/README.md`](./handoffs/README.md)
for the index.
