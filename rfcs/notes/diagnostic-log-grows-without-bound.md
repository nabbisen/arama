# `diagnostic.log` grows without bound on Windows release builds

**Found:** 2026-08-25, while reviewing Task 038 (review 133). **Not** a Task 038
defect — that work only depends on the behaviour, and correctly so.

**Shipped in:** 0.41.2, with Task 037.

## The behaviour

`env/src/diagnostic.rs` opens the file in append mode and never rotates,
truncates or caps it:

```rust
OpenOptions::new().create(true).append(true).open(path)
```

`app/src/core.rs:142` calls `diagnostic()` **unconditionally at every startup**,
on both arms of the match — resolved locations on one, the unresolved message on
the other. On a Windows release build `diagnostic()` writes to the file rather
than stderr (Task 037: there is no console).

**So every launch appends at least one line, forever.** Nothing deletes it,
nothing bounds it, and the user has no in-app way to see or clear it.

## Why it is small, and why it is still worth recording

One line per launch is trivial in bytes; this will not fill a disk. The reasons
it is worth a note anyway:

1. **It is unbounded, which is a different property from "large".** The size
   depends on how long a user has had arama installed, and nothing in the
   codebase bounds it.
2. **It is invisible.** The file lives in the platform data directory (RFC 041),
   and under packaging it is redirected again into the package's `LocalCache`.
   A user will not find it and cannot clear it from the application.
3. **The file is the only diagnostic channel Windows release builds have.** If
   it ever does become a problem, the fix cannot be "stop writing it".

## What this is not

**Not** an argument against Task 037's design. Routing diagnostics to a file was
correct — a Windows GUI build has no stderr, and RFC 041 §7 requires resolved
locations to be discoverable without a debugger. The question is only what
bounds the file.

## Options, unranked and uncosted

- **Truncate at startup.** One launch's worth of diagnostics, always. Cheapest;
  loses history across runs, which is what makes an intermittent startup failure
  diagnosable.
- **Rotate at a size cap.** Keeps recent history, bounded. Needs a cap and a
  rename.
- **Write the data-locations line only when something is wrong.** Attacks the
  volume rather than the file — but RFC 041 §7 asked for the resolved case to be
  discoverable too, so this narrows an RFC requirement and should not be done
  quietly.
- **Leave it.** Defensible. One line per launch may simply never matter.

**No recommendation is made here.** This note exists so the decision is taken
deliberately rather than discovered from a support question.

## Related

Task 037's two existing follow-ups — a `temp_dir()` fallback for the fatal
startup diagnostic, and the ~22 remaining `eprintln!` sites silenced on Windows
release builds — are the same seam. If any of the three is scheduled, all three
should be looked at together.
