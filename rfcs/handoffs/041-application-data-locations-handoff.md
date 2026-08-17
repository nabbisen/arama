# RFC 041 Handoff — Application data locations

Companion to [RFC 041](../proposed/041-application-data-locations.md), which is
**accepted for implementation** (owner, 2026-08-16) and stays in
`rfcs/proposed/` until the work ships, per
[RFC 000](../done/000-rfc-lifecycle-policy.md).

**Do this before [RFC 042](../proposed/042-windows-store-distribution.md).**
That work cannot start until a packaged arama can write anything at all.

## 1. Design authority

1. [RFC 041](../proposed/041-application-data-locations.md);
2. [RFC 017](../done/017-visible-recoverable-error-ux.md) — the tier model
   governing how a failed migration is surfaced;
3. [RFC 030](../done/030-distribution-and-version-contracts.md) — the
   executable-asset layout, which this does **not** change.

## 2. The RFC's §2 is wrong about the cache. Correcting it here.

RFC 041 recommends *"cache — abandon and rebuild,"* citing RFC 015's precedent.
**That precedent does not apply and the recommendation is withdrawn.**

RFC 015 abandoned the v1 cache because its **format** had changed and the old
data was unusable. Here the data is perfectly valid — it is in a different
folder. Abandoning it would force a full re-index, which by the owner's own
account is arama's slowest operation and scales with library size. For a large
library that is hours of work destroyed to avoid a file move.

**Move all three: settings, models, and cache.** Abandon nothing.

The instinct to treat a cache as disposable is usually right and is wrong here,
because the cost of rebuilding is borne by the user and is large.

## 3. Settled design questions

**3.1 Follow the platform's own distinction** (RFC §1 as written). Settings in
the config location, models in data, cache in cache. Concretely: on Windows the
cache and models belong under `%LOCALAPPDATA%`, **not** `%APPDATA%` — a
multi-hundred-megabyte model in a roaming profile is a real misbehaviour that
will surface as slow logins on managed machines.

`app-json-settings` already resolves the config location. It does **not** solve
data and cache; those need their own resolution in `env/src/dir.rs`.

**3.2 Migrate everything, once, on first run.** Per §2 above.

**3.3 No portable mode.** One behaviour. The owner has confirmed portability is
not essential, and a mode question is a support burden this project does not
need.

## 4. Required implementation

**4.1 Remove both `.at_current_dir()` calls** (`app/src/core.rs:100`, `:237`,
and check for others). This alone fixes the "settings follow where you launched
from" defect.

**4.2 Replace `current_exe().parent()`** in `env/src/dir.rs`'s `local_dir()` and
`cache_dir()` with platform data and cache locations.

**4.3 First-run migration**, one pass, in this order of care:

- **Settings** — smallest, most precious. If the new location has none and the
  old location has some, read and rewrite.
- **Models** — large. Move rather than copy where the filesystem allows it.
- **Cache** — large, and a move is what saves the user a re-index.

**4.4 A failed migration must be visible.** RFC 017's tiers govern. Losing a
user's settings silently, or silently re-indexing a large library, is the defect
this task exists to avoid — do not let the failure path be the quiet one.

**4.5 Creating the new location is a startup precondition.** If it cannot be
created, that is fatal and belongs in RFC 017's fatal-startup tier, not a toast
the user might miss.

## 5. Traps

- **Both locations populated.** A user who runs an old build and a new build has
  data in both. Decide which wins, implement it deliberately, and say so in the
  code. Do not let it be whatever the filesystem happens to return.
- **A move across filesystems is a copy-then-delete**, not a rename, and can
  fail halfway. A partially-moved model directory must not look like a complete
  one to the setup wizard's readiness check
  (`ModelContainer::ready_in` checks *presence*, not integrity — see review 096).
- **`%LOCALAPPDATA%` vs `%APPDATA%`** is the mistake most likely to pass review
  and hurt users later. Roaming profiles synchronise.
- **The scratch-profile smoke method depends on the current layout.** RFC 036's
  and RFC 040's evidence runs rely on CWD-relative and exe-relative resolution to
  isolate a test profile. **Those will break.** Establishing a replacement
  isolation mechanism is part of this task, not an afterthought — without it,
  every future rendered-evidence cycle is blocked.

That last one is the trap most likely to be discovered too late.

## 6. Non-change scope

- What is stored. Only where.
- A user-configurable location.
- RFC 030's executable-asset layout.
- Store packaging. RFC 042, and it follows this.
- The setup wizard's own flow, beyond where it writes.

## 7. Verification

**All three platforms.** This is platform-resolution code and Linux proves
nothing about Windows. RFC 038's native-smoke workflow already runs on
`windows-latest` and `macos-latest` — extend it rather than building something
new.

Required:

- Launching from two different working directories yields the **same** settings.
  That is the defect in §4.1, and it is trivial to assert.
- Migration from the old layout, per data kind, including the both-populated
  case.
- Nothing is written to the executable's directory. Assert it, do not eyeball it.
- The resolved locations are **logged or otherwise discoverable**, so a failing
  run can be diagnosed without a debugger.

**Not required here:** the packaged-Windows question. That is RFC 042's Phase 0
and needs a packaged build to answer.

## 8. Acceptance criteria

- No `.at_current_dir()` and no `current_exe().parent()` data path remains.
- Settings, models and cache each resolve to the platform-appropriate location,
  with cache and models off the roaming profile on Windows.
- All three migrate; nothing is abandoned.
- A failed migration or an uncreatable location is surfaced per RFC 017.
- A replacement scratch-profile isolation method exists and is documented.
- Verified on Windows, macOS and Linux.

## 9. Required deliverables

A review request under `.git-exclude/review-request/NNN-<slug>/` opening with an
`## Entry point` section giving the pinned commit, the exact `git show` command,
and plain paths to every file. Include the per-platform verification, the
migration evidence, and the replacement isolation method. Report observed
output; a check not run is recorded as not run.
