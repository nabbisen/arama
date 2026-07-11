# Roadmap

This roadmap records near-term project themes that are specific enough to
guide planning but not yet implementation promises. Non-trivial themes move
through the RFC process before code changes begin.

## Current focus

### Audit warning burn-down

**Status.** Maintenance pass in progress.

**Why now.** `cargo audit` currently reports allowed warnings beyond the scoped
`quick-xml` ignores. Advisory policy should stay narrow: patch-level fixes that
are available now should be taken, while warnings blocked by upstream direct
dependencies should be recorded with dependency owners instead of hidden behind
broad ignores.

**Planned maintenance questions.**

- Which warnings can be resolved with compatible patch updates?
- Which warnings are blocked by upstream crates that are already on their latest
  compatible release?
- Which warnings are transient lockfile residue rather than active workspace
  dependency paths?
- Which remaining advisory exceptions need release-gate policy changes versus
  simple tracking notes?

## Recently implemented, pending owner-managed lifecycle

### Cache lifecycle

**Status.** RFC 015 implementation reviewed; release/lifecycle transition
pending owner action.

**Why now.** RFC 002 moved arama from the old `file-feature-cache` engine to
`localcache` in v0.23.0. RFC 015 retires the temporary v1 migration path and
keeps cache-size/disk-pressure management split into a separate design.

**Follow-up status.** RFC 016 implementation reviewed; release/lifecycle
transition pending owner action.

### Visible recoverable error UX

**Status.** RFC 017 implementation reviewed; release/lifecycle transition
pending owner action.

### AI and video pipeline resilience

**Status.** RFC 018 implementation reviewed; release/lifecycle transition
pending owner action.

### Startup fatal-boundary resilience

**Status.** RFC 019 implementation reviewed; release/lifecycle transition
pending owner action.

## Later candidates

### Dependency modernization

Direct dependency minor/major updates remain design or implementation work when
they change public behavior, build requirements, or compatibility risk. The
audit warning burn-down note should feed future modernization candidates when an
upstream crate exposes a compatible fix.

### Release prep

Release prep remains owner-driven. The roadmap does not make a release point;
it only identifies when a coherent reviewed batch may be ready for release
consideration.
