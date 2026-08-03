# RFC 029 - Release smoke evidence template

**Status.** Implemented (0.37.0)
**Tracks.** Maintenance follow-up: make owner-managed release-smoke evidence
repeatable and comparable across releases.
**Touches.** `docs/src/dev/testing.md`,
`docs/src/dev/release-smoke-evidence-template.md`,
`docs/src/dev/release.md`, `docs/src/SUMMARY.md`, `ROADMAP.md`,
`rfcs/README.md`.

## Summary

RFC 025 added the current release-smoke checklist to `docs/src/dev/testing.md`.
The checklist is shared and useful, but it is still coarse: future release
owners and reviewers cannot easily record which items were run, skipped, or
environment-dependent for a specific release.

This RFC proposes a small documentation refinement:

1. Assign stable IDs to the existing release-smoke checklist areas and bullets.
2. Add a reusable owner evidence template for release-smoke runs.
3. Keep manual GUI smoke owner-managed and separate from release mechanics.
4. Do not add a desktop UI automation harness in this RFC.

## Why

The project now has a strong release-readiness workflow for automated gates and
review packages, but manual GUI smoke evidence remains hard to compare across
releases. A stable template would improve maintenance because it gives owners
and reviewers a durable place to record:

- what release build/profile was used;
- which smoke items passed, failed, or were not run;
- which checks were skipped because they require network, clean local state, or
  owner-specific setup;
- whether follow-up work belongs in docs, tests, or a future automation RFC.

This avoids two bad outcomes:

- treating broad smoke guidance as if it were recorded release evidence;
- jumping directly to brittle GUI automation before the repeatable evidence
  shape is clear.

## Proposal

### Part A - Add stable smoke IDs

Update the existing release-smoke section in `docs/src/dev/testing.md` so each
check has a stable ID. The IDs should be descriptive and grouped by surface, for
example:

- `SMOKE-SETUP-*`
- `SMOKE-GALLERY-*`
- `SMOKE-SIMILARITY-*`
- `SMOKE-CACHE-*`
- `SMOKE-SETTINGS-*`
- `SMOKE-RESTART-*`

The existing checklist scope should remain intact. The implementation may adjust
wording for clarity, but it should not expand the checklist into exhaustive QA.

### Part B - Add an owner evidence template

Add a reusable Markdown template under `docs/src/dev/`, tentatively:

```text
docs/src/dev/release-smoke-evidence-template.md
```

The template should include:

- release/version under consideration;
- date;
- platform;
- build command or binary used;
- profile/local-state description;
- fixture directory description;
- network/download availability;
- a result table keyed by the stable smoke IDs;
- result values such as `pass`, `fail`, `not run`, and
  `environment-dependent`;
- notes and follow-up links.

This template is for owner-run evidence. It is not a command script and does not
perform release actions.

### Part C - Link from release docs

Update `docs/src/dev/release.md` to point to the evidence template when a release
includes UI, setup, cache, first-run, or recoverable-error changes.

The wording should keep the current boundary:

- automated gates remain the baseline;
- GUI smoke remains owner-managed;
- release mechanics remain owner-managed and separate.

### Part D - Leave automation for later

Do not add automation hooks in this RFC. A later RFC can propose a narrow
`--headless-smoke` or similar hook after the evidence template proves which
checks are stable enough to automate.

## Non-goals

- No UI automation framework.
- No `--headless-smoke` implementation.
- No new test dependency.
- No product behavior change.
- No requirement that every release environment has network access.
- No requirement to delete or corrupt the owner's real local state.
- No release action, version bump, changelog finalization, archive, tag,
  publish, or push.

## Risks

- The evidence template could become too heavy. Mitigation: keep result fields
  short and allow `not run` for environment-dependent checks.
- Stable IDs could create false precision if the underlying checks are vague.
  Mitigation: keep IDs tied to existing checklist bullets and refine wording only
  where necessary.
- Reviewers could treat missing owner smoke as an automated gate failure.
  Mitigation: keep release docs explicit that manual GUI smoke is owner-managed
  confidence evidence, not a replacement for automated gates.

## Acceptance criteria

- `docs/src/dev/testing.md` keeps the RFC 025 release-smoke scope and gives each
  smoke item a stable ID.
- A reusable owner evidence template exists under `docs/src/dev/`.
- `docs/src/dev/release.md` links to the evidence template without making it an
  automated gate.
- The template separates `pass`, `fail`, `not run`, and
  `environment-dependent` results.
- No release mechanics are performed.
- Documentation builds cleanly.

## Review evidence

Required for proposal review:

```sh
mdbook build docs
git diff --check
```

Required for implementation review:

```sh
mdbook build docs
git diff --check
```

## Implementation notes

The implementation assigned stable IDs to all 17 RFC 025 smoke checks, added a
reusable owner evidence template with the four accepted result values, and
linked the template from the testing guide, release process, and mdBook
navigation. It did not add automation, dependencies, product behavior, or
release mechanics.
