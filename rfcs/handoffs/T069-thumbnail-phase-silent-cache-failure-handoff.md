> **AUTHORIZED 2026-11-08 — owner:** *"Authorized."*, given to the architect's
> question of whether to write this fix as a task. Recorded as authorizing both
> the writing and the work — the same reading as *"Authorized. Proceed"* for
> T063/T064. Written 2026-11-08 by the architect.
>
> **Do this AFTER T061.** T061 rewrites `handle_cache_require` — the exact
> function this task fixes — for cancellation and `spawn_blocking`. Running both
> at once would make neither diff attributable, and T061's restructuring will move
> the lines below. **Work from the behaviour described here, not the line
> numbers**, which are as of `d930756`.
>
> Found while reviewing T067 (review 159).
>
> **Moved 2026-11-08** from `.git-exclude/tasks/dev-team/` into `rfcs/handoffs/`, at the owner's direction, and completed against the 17-item Developer Handoff structure. The authorization above is unchanged.

# T069 Handoff — the thumbnail phase swallows a cache that cannot be opened

Companion to [RFC 017](../done/017-visible-recoverable-error-ux.md). Found while reviewing T067 (review 159).

## 1. The defect

`handle_cache_require` (`app/src/core/update/cache.rs`, around lines 27-62 at
`d930756`) opens the cache inside its async body and handles **both** ways that
can fail by returning an empty result:

```rust
let Ok(cache_path) = cache_storage_path() else {
    return vec![];
};
let Ok(writer) =
    ImageCacheWriter::onetime(arama_cache::DbLocation::Custom(cache_path))
else {
    return vec![];
};
```

The result goes to `handle_thumbnail_cache_finished`, which **does** surface
per-file errors as a toast (`toast.cache_error.title`). **But an empty vec is
indistinguishable from "nothing to do".** So:

- One file fails → the user is told.
- **The whole cache fails to open → the user is told nothing**, and indexing
  appears to complete while producing nothing.

**No diagnostic, toast or notice on either branch** — zero surfacing calls.

**What reaches it:** a permissions problem on the cache directory, a full disk,
a corrupt database, an unresolvable platform directory, or **a schema this build
does not recognise**. The last became concrete with T067: localcache's one-way v6
migration means an older arama meets exactly this error on a cache a newer one
touched — verified in review 159 — and the shipped versions handle it silently.
We cannot change those; we can stop it happening from here on.

## 2. The precedent is in the same file

**The reload phase already does this correctly.** Its reader-open failures go
through:

```rust
(Err(err), _) => self.push_error_toast(
    t("toast.cache_reload_failed.title"),
    format!("{}: {err}", t("toast.cache_reload_failed.image_reader.body")),
),
```

with `toast.cache_reload_failed.storage_path.body` covering the path case. **The
thumbnail phase never got the same treatment.** The same shape as RFC 047's own
finding: *"someone solved this problem once and the fix did not reach the sibling
phase."* Follow it rather than invent a new pattern.

## 3. What to build

**3.1 Make the result able to say "the cache could not be opened."** Today the
async body returns `Vec<(PathBuf, Arc<Result<()>>)>`, which can only express
per-file outcomes. It needs to distinguish a cache-level failure from per-file
results — a `Result` around the vector, or a dedicated variant. Your choice;
state it.

**3.2 Surface it in `handle_thumbnail_cache_finished`**, with the underlying
error included, as the reload phase does.

**3.3 The wording must fit the phase.** *"Cache reload failed"* describes the
reload phase; this is indexing. New keys are likely right — in **both**
`crates/i18n/src/en.rs` and `ja.rs`; the `Untranslated strings` CI job enforces
it. Say whether you reused or added, and why.

**3.4 Decide the tier, against RFC 017.** A per-file failure is transient. A cache
that cannot be opened at all fails **every** indexing attempt until the user acts.
[RFC 017](../done/017-visible-recoverable-error-ux.md) distinguishes
these; read it and **say which tier you chose and why** — a toast that fires on
each attempt, or a persistent state. If a persistent state, consider whether it
should point the user at what to do (the cache directory, or the in-app clear —
which itself needs the cache to open; see review 159).

**3.5 Keep `processing` honest.** If the cache cannot be opened, indexing has not
happened. Whatever the UI shows as "done" must not claim otherwise — the same
principle RFC 047 §7.1 applied to a partial index.

## 4. Verification

**Prefer assertions to pictures** — the local GUI rules file (`.git-exclude/rules/project-instructions-gui-common.md` — in the working copy, not the repository)
governs this. The failure is a state, not an appearance.

- **A test that makes the cache unopenable and asserts the user is told** — not
  that an empty vec came back. The simplest reliable way to force the failure is
  your call: a path that cannot be created, or a file whose schema this build
  rejects. **The v6-schema case from review 159 is the realistic one** if you can
  stage it.
- The per-file path still works and still toasts as before — a regression check,
  since you are changing the shared return type.
- Both locales render the new text.
- `cargo build`, `cargo test`, `cargo fmt --check`, `cargo clippy -D warnings`,
  all `--workspace --locked`.

## 5. Change scope

- `app/src/core/update/cache.rs`
- the message type carrying the thumbnail result, wherever it is defined
- `crates/i18n/src/en.rs`, `crates/i18n/src/ja.rs`

## 6. Non-change scope

- the reload phase's error handling — it is the precedent, not a target
- `crates/cache/` — the library reports the error correctly; the defect is in
  how the app handles it
- `CHANGELOG.md` or any `Cargo.toml` version field

## 7. Documentation updates

- None required beyond the new i18n strings, which are the user-facing text.
- If `docs/src/users/cache.md` describes what happens when the cache cannot be used, align it.

## 8. Compatibility constraints

- The shared thumbnail-result type changes shape (§3.1). **The per-file error path must behave exactly as before** — the regression check in §4.
- T061 lands first and may have restructured `handle_cache_require`. Build on what it leaves.
- No change to `crates/cache/`, the payload versions, or the schema.

## 9. Security constraints

- The surfaced message includes the underlying error, which can contain the cache path — and so the user's home directory. On the user's own screen that is acceptable.
- **It must not reach committed evidence.** Per the GUI rules, run with isolated `XDG_*` and `ARAMA_DATA_HOME`, so any path shown is a neutral scratch path.

## 10. Known risks

- A toast on every indexing attempt can become noise; a persistent state needs a recovery path, and the in-app clear itself needs the cache to open (review 159).
- T061 will have moved the code — work from the behaviour, not the line numbers (banner).
- Changing the shared return type can silently break the per-file path.

## 11. Acceptance criteria

- An unopenable cache produces a user-visible message carrying the reason, in both locales.
- The indexing state does not claim completion when nothing was indexed (§3.5).
- The per-file error toast behaves as before.
- The chosen RFC 017 tier is stated and justified (§3.4).
- `cargo build`, `test`, `fmt --check` and `clippy -D warnings` clean; `Untranslated strings` passes.

## 12. Required deliverables

Review request with `## Entry point`. Include your §3.1 and §3.4 choices with
reasons, the test that forces the failure and asserts it surfaces, and the
regression check for per-file errors.

You cannot push: report the commit and the architect pushes and watches every
workflow on that SHA.

Report observed output; a check not run is recorded as not run.

## 13. Handoff checklist

The 17 items the architect's operating instructions require of every Developer Handoff, and where each is covered here.

| Required item | Covered in |
|---|---|
| Task title | Heading |
| Purpose | §1 |
| Background | §1, §2 |
| Applicable RFC and requirements | Authorization banner and companion line, §3.4 |
| Change scope | §5 |
| Explicit non-change scope | §6 |
| Required implementation | §3 |
| Required tests | §4 |
| Required documentation updates | §7 |
| Acceptance criteria | §11 |
| Prohibited shortcuts | §2, §6 |
| Relevant code or module boundaries | §5 |
| Compatibility constraints | §8 |
| Security constraints | §9 |
| Known risks | §10 |
| Required evidence | §4 |
| Required review-request format | §12 |
