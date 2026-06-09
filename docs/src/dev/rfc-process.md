# RFC Process

arama uses a lightweight RFC (Request for Comments) process for
non-trivial design decisions. The lifecycle is defined in
[RFC 000](../../rfcs/done/000-rfc-lifecycle-policy.md); this page
summarises the practical steps.

## When to write an RFC

Write an RFC when a change:
- Affects the public API of a crate that other crates depend on
- Introduces a new dependency or removes an existing one
- Rearchitects a significant component (navigation shell, caching
  strategy, AI pipeline)
- Is too large or uncertain to describe adequately in a commit message

Bug fixes and routine refactors do not need RFCs.

## Lifecycle

```
Draft (branch / gist)
      │
      ▼
Proposed  ──  rfcs/proposed/NNN-slug.md
      │
      ▼
Implemented  ──  rfcs/done/NNN-slug.md   (never deleted)
```

Withdrawn or superseded RFCs move to `rfcs/archive/`.

## Writing an RFC

1. Assign the next sequential number (`NNN`) — check `rfcs/` to find the
   current highest.
2. Create `rfcs/proposed/NNN-slug.md` with the header:

   ```markdown
   # RFC NNN — Title

   **Status.** Proposed
   **Tracks.** What this addresses.
   **Touches.** Which files / crates change.
   ```

3. Sections to include: Summary, Why, Design (with ASCII diagrams where
   helpful), Touches in detail, Open questions.

4. Update `rfcs/README.md` to list the RFC in the Proposed table.

## Implementing an RFC

1. Implement the changes in the same branch or as a focused PR.
2. When the work ships:
   - Move the file from `rfcs/proposed/` to `rfcs/done/`.
   - Update the `**Status.**` field to `Implemented (vX.Y.Z)`.
   - If implementation deviated from the design, add an
     `## Implementation notes` section recording the differences.
   - Update `rfcs/README.md` — move the entry from Proposed to
     Implemented.
   - Update `CHANGELOG.md`.

## Rules

- **Numbers are permanent.** RFC 005 stays 005 even if withdrawn.
- **Files are never deleted.** Withdrawn RFCs move to `archive/`.
- **The folder is the source of truth** for status, not the Status
  field. If they disagree, the folder wins.
- Cross-references use relative paths. Fix them in the same commit that
  moves the RFC.

## Current RFCs

See [rfcs/README.md](../../rfcs/README.md) for the full index.
