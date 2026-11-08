> **AUTHORIZED 2026-10-21 — owner:** *"All of them are authorized."*
>
> **Correction to this banner's earlier text.** It said RFC 047 was "authorized
> as an RFC on 2026-09-05". **That was wrong.** The 2026-09-05 grant read
> *"RFC-047 is authorized"* and meant **Task 047**, the snora upgrade — the
> number collision recorded in the architect's release schedule (local, not in the repository) under *"Process
> hazard found 2026-09-02"*, which states plainly that RFC 047 *"remains
> proposed and unscheduled"*.
>
> **So this task is authorized and the RFC it implements is formally still
> Proposed.** The architect has flagged that to the owner. It does not block
> you: the task-level grant above is the signal to start, and the RFC's analysis
> is unchanged either way.
>
> **Do T058 and T059 first.** `main` is red on `Audit` until both land, and a
> behaviour change this size should not go onto a red branch where its own CI
> result cannot be read.
>
> **This is the largest task in 0.43.0.** If it has to slip, it slips alone —
> the advisory and packaging work does not depend on it.

> **Added 2026-11-08 — T069 follows this task on the same function.**
> `handle_cache_require` returns `vec![]` silently when the cache cannot be
> opened (both the storage-path and the writer-open branches). **T069 fixes that,
> after you land.** So: **leave those two early returns' behaviour as it is** —
> do not fix it here, which would widen this task and make both diffs
> unattributable, and do not make it worse. If your `spawn_blocking`
> restructuring moves them, keep them recognisable, and say in your package where
> they ended up so T069 can find them.
>
> **Moved 2026-11-08** from `.git-exclude/tasks/dev-team/` into `rfcs/handoffs/`, at the owner's direction, and completed against the 17-item Developer Handoff structure. The authorization above is unchanged.

> **SECOND REWORK REQUESTED 2026-11-09 — review of `2567fcf`.** Everything
> the first rework asked for is done. One regression came with it: replacing
> the pair scan's abort handle with `ScanControl` means a cancelled scan still
> delivers its (partial) result, and `handle_similar_pairs_dialog_message`
> accepts any result — so after close-and-reopen, the old scan's partial pairs
> land in the new dialog, and its arrival also drops the new scan's control.
> **Fix:** the result carries the identity of the scan that produced it
> (a generation number); a stale one is dropped without touching state. Test it
> through `App::update`. Also, smaller: set the stop-early test's flag inside
> the hook (exactly one chunk, by construction) and correct its comment; make
> the grandchild heartbeat test count lines rather than read `date +%s%N`
> (BSD `date` has no `%N`).
>
> **REWORK REQUESTED 2026-11-08 — architect review of the first delivery.**
> The authorization above covers this rework; nothing new needs granting. The
> thumbnail-phase work stands. Five changes, then resubmit as a new package:
>
> 1. **Stop must return the app to idle.** The Stop arm never calls
>    `processing_off()`, and an aborted task never delivers its completion
>    message, so after one Stop the directory tree stops responding (the aside
>    blocks expand and selection while processing) and media-type or depth
>    changes never start a run. Call it in the Stop arm, and assert
>    `!processing` after Stop in the existing `App::update` test.
> 2. **Closing the pairs dialog must stop the scan.** Aborting a
>    `spawn_blocking` task has no effect (tokio documents this), so closing the
>    dialog today abandons the scan and leaves it running. Use the thumbnail
>    phase's mechanism: a shared flag set on close, checked inside the scoring
>    loop.
> 3. **The scan must report progress (§6), not only that it is running.** A
>    determinate *N of M* from the same loop as 2. Not streamed results
>    (RFC 047 §5).
> 4. **§10: reap the ffmpeg process tree on Windows** with `command-group`
>    (already a workspace dependency, used by the sidecar for Task 021), and
>    either let Stop reach a running ffmpeg through the existing `try_wait`
>    poll, or argue in the package why that half is separable.
> 5. **Make the stop-early test deterministic** — trigger the flag from the
>    run's own progress, not from a 20 ms sleep.
>
> Also: reset `embedding_completed` in `handle_cache_require` (the one choke
> point every run start passes through — two settings-triggered starts miss it
> today), and describe the full-run test as what it does (no decode happens in
> that phase). §8 below has been corrected; it contradicted §2c.

# T061 Handoff — indexing that can actually be stopped (RFC 047)

Companion to [RFC 047](../proposed/047-indexing-cancellation.md). **Read the RFC first** — this handoff settles what it left open and names the traps.

**RFC:** `rfcs/proposed/047-indexing-cancellation.md` — read it first and in
full. This task does not restate its analysis.

## 1. The shape of the problem, in one line each

**Four documents describe an application that can be interrupted. It cannot.**

- **The thumbnail phase cannot be aborted.** `handle_cache_require`
  (`app/src/core/update/cache.rs:30-56`) wraps `writer.upsert_all(requests)` in
  an `async move` block with **no `.await` in it at all**. The `.abortable()`
  handle is real and Stop dispatches to it; there is no point at which the
  future can be dropped. **The embedding phase awaits and so aborts correctly —
  someone solved this once and the fix did not reach the sibling phase.**
- **Three async bodies do blocking work** (RFC §1.2): `handle_cache_require`,
  `SimilarPairsDialog::default_task`, and `download_generation` — the last
  occupying a tokio worker for a 605 MB synchronous conversion *while a second
  download runs*.
- **The pair scan is O(N²) with no bound, no progress and no way out.**
- **The Similarity Pairs button ungreys ~35 lines before embedding is spawned**,
  so it is available before any embedding exists, and `using-arama.md` says
  twice that it stays greyed until indexing finishes.

## 2. What to build — RFC §3

**2a. `spawn_blocking` for the three sites.** The pattern is already in this
repository at `publication.rs:22` and
`sidecar/.../discovery/runtime.rs:45`. **This is not a technique to introduce,
it is one to apply consistently.** It fixes scheduler nesting and blocked
workers. It does **not** fix cancellation — do not report it as if it does.

**2b. Cooperative cancellation for the indexing loop.** Chunk the work, yield
between chunks, check a shared `AtomicBool` that both Stop and a directory
switch set.

**The RFC is explicit about why both halves are needed, and it is the thing
most likely to be got wrong:** dropping a future at an await point cancels
*the future*, but rayon work already dispatched inside `upsert_all` keeps
running. **The flag is what stops the work; the yield is what lets the flag be
observed.** A `yield_now` alone looks like it works and does not.

**2c. The pair scan: interruptible, progress-reporting, bounded, and honest.**

> **Changed 2026-10-21.** This previously said the bound and the
> retained-versus-total counts were RFC 048's and must not be implemented here.
> **That is reversed.** RFC 048 §3 moved into RFC 047 §3.3, because the two were
> halves of one user-visible behaviour and shipping only the cancellation half
> would leave a scan that stops cleanly and still misreports what it found.

So this task owns all four: the scan can be **cancelled**, it **reports
progress**, it applies **one cap to images and videos alike** (today
`MAX_IMAGE_SIMILAR_PAIRS = 50` caps images and nothing caps videos), and the
dialog **renders retained-versus-total counts** so truncation is
distinguishable from completeness.

**Raising or removing the cap is still not this task's** — that depends on
RFC 048 §1's virtualisation work and is explicitly deferred.

**2d. Gate the pairs button on completion, not file count.** `embedding_cached`
should mean what its name says.

## 3. The hazard this interacts with

The audit's B3: media-path ffmpeg invocations have no timeout and no output cap.
**A hung ffmpeg inside an uncancellable phase cannot be killed from the UI.**
RFC §4 says bounding the subprocesses belongs with this work rather than after
it. Treat it as in scope; if you conclude it is genuinely separable, say why
rather than quietly dropping it.

## 4. Cancellation is partial — decided, with a condition attached

**RFC 047 §7 is no longer an open question.** Decided 2026-10-21: cancellation
is **cooperative and partial**. Stopping mid-index leaves the cache populated
for some files and not others, and that is correct — Stop means stop doing more
work, not undo work already done, and a "transactional" rollback across both
SQLite rows and thumbnail files on disk would be real machinery whose own
failure mode is worse than the state it prevents.

**The condition is binding on §2d and is the part to get right.** A partial
index is only acceptable if the user can tell. So `embedding_cached` must mean
*embedding completed*, and a directory stopped midway must **not** satisfy it.
**The pairs button staying greyed after a Stop is the correct, visible
consequence of a partial index — not a bug to work around.**

Read RFC 047 §7 and §7.1 in full; they carry the reasoning.

## 5. Documentation — RFC §6's last line

> *"No document describes an interruption the code does not implement."*

Four documents currently do. Find them and correct them in the same change —
`using-arama.md`'s two claims about the greyed-out button are a known starting
point, not the whole list. **A doc fix without the code fix, or the code fix
without the doc fix, both leave the repository lying.**

## 6. Verification

**Gates are necessary and nowhere near sufficient here.** This is a behaviour
change in concurrency; `cargo test` passing proves very little about it.

RFC §6's acceptance list plus the two items added 2026-10-21, each
demonstrated and reported:

1. **Stop interrupts the thumbnail phase**, on a directory large enough to
   observe it. Say how many files, and how long it took to stop.
2. **Switching directories mid-index cancels both phases.**
3. **The three blocking bodies run under `spawn_blocking`** — show the diff.
4. **The pair scan reports progress and can be cancelled**, applies one cap to
   both media types, and shows retained-versus-total counts.
5. **After a Stop, a partially-indexed directory does not present itself as
   fully indexed** (§4's condition).
6. **The pairs button reflects embedding completion** — not file count.
7. **No document describes an interruption the code does not implement.**

### 6.1 How to evidence it — amended 2026-10-27

> **The owner rewrote the local GUI rules file (`.git-exclude/rules/project-instructions-gui-common.md` — in the working copy, not the repository)
> on 2026-10-27, adding "GUI Evidence and Automation Rules". Read it before
> planning any run.** It governs this task and it changes what good evidence
> looks like here. The amendments below are what it implies for these seven
> items; where the two disagree, the rules file wins.

**Prefer recorded state and machine comparison over pictures.** The rules put it
directly: look at a picture only where a comparison says it differs, or where
the picture *is* the deliverable. **For this task, almost none of it is.** Items
1, 2, 5 and 6 are state claims, not appearance claims.

**Most of this is provable in-process.** `iced_test::Simulator` is already used
in `app/src/core/tests.rs` (RFC 044 §0.2b probed it), and that file already
asserts exactly this shape of state — `root.dir_node.is_none()`, notice counts,
message contents. Prefer that to a capture:

| Item | Preferred evidence |
|---|---|
| 1. Stop interrupts the thumbnail phase | instrumented log plus a **timing number read from the source**, not from frames |
| 2. Directory switch cancels both phases | same |
| 3. `spawn_blocking` at three sites | the diff |
| 4. Pair scan: progress, one cap, retained-vs-total | assert the counts and the rendered text; a capture only if the dialog's *appearance* is in question |
| 5. Partial index does not present as complete | **assert `embedding_cached` is false after a Stop** — this is a flag, not a picture |
| 6. Pairs button reflects completion | assert the disabled state |
| 7. No document describes an unimplemented interruption | the diff |

**No frame-sampling loops.** The rules forbid photographing in a loop to find a
number, and the timing figure in item 1 is exactly that temptation. **Instrument
the code or use one timed check** — a capture at the start and one after the
expected interval. *"Stop acknowledged in ~200 ms on 1,200 files"* remains worth
more than "it works", but it must come from a log or a measurement, never from
counting frames.

**If you do capture anything**, the rules bind: fixed window size obtained **at
launch** and never by resizing a running window, capture through
`niri msg action screenshot-window` with an absolute path, isolated
`XDG_CONFIG_HOME` **and** `XDG_STATE_HOME` under a scratch directory, exactly
one instance chosen by pid, the backend stated, and before/after compared only
on the same backend. **Abort at the first anomaly and report it.**

**Repeat limit.** If a capture set has to be repeated a second time, stop and ask
the architect before a third.

Plus: `cargo build --workspace --locked`, `cargo test --workspace --locked`,
`cargo fmt --check`, `cargo clippy --workspace --all-targets --locked -- -D
warnings`.

**The Windows job matters more than usual.** Process-tree reaping on a probe
timeout (Task 021) is Windows-specific and this task touches subprocess
lifetime. The architect pushes and watches all seven workflows.

## 7. Change scope

- see the sections above

## 8. Non-change scope

- **raising or removing** the pair-scan cap — deferred to RFC 048 §1 (§2c). *Corrected 2026-11-08: this line used to read "the pair-scan bound or truncation counts — RFC 048", left over from before 2026-10-21 and contradicting §2c and §6 item 4. The cap and the counts are this task's; only changing the cap's value is not.*
- anything in RFC 046's audio path
- `CHANGELOG.md` or any `Cargo.toml` version field

## 9. Compatibility constraints

- **No payload or schema change.** `VIDEO_PAYLOAD_VERSION` and `IMAGE_PAYLOAD_VERSION` must not move.
- A Stop leaves a partially populated cache — a state existing caches can already be in today through the embedding phase (§4) — so no migration is needed.
- **`embedding_cached` changes meaning** to *embedding completed* (§4). Check every reader of it, not only the pairs button.
- The thumbnail-result message type may change shape to report cancellation. T069 builds on whatever shape this task leaves.

## 10. Security constraints

- **Cancellation must not orphan subprocesses.** A hung ffmpeg on untrusted media is the hazard in §3 (audit B3). Stopping must terminate it — including its process tree on Windows, per Task 021's reaping.
- Terminate only the processes this task spawned, by handle — **never by process name across the system**.
- No new network or filesystem surface.

## 11. Known risks

- **`yield_now` without the flag looks like it works and does not** (§2b) — rayon work already dispatched inside `upsert_all` keeps running after the future is dropped.
- Windows process-tree reaping on timeout is Windows-specific; the `-msvc` CI job is the only proof.
- **T069 follows on `handle_cache_require`.** Leave its two silent early returns alone (banner).
- **T064 also edits `app/src/core/view.rs`.** This task has right of way.
- Scope creep into RFC 048 is out: the cap and the counts are this task's (§2c); raising or removing the cap, and RFC 048 §1's rendering work, are not.

## 12. Acceptance criteria

The items in §6, each demonstrated as §6.1 describes:

- Stop interrupts the thumbnail phase.
- Switching directories mid-index cancels both phases.
- The three blocking bodies run under `spawn_blocking`.
- The pair scan reports progress, can be cancelled, applies one cap to both media types, and shows retained-versus-total counts.
- A partially indexed directory does not present itself as complete.
- The pairs button reflects completion, not file count.
- No document describes an interruption the code does not implement.
- `cargo build`, `test`, `fmt --check` and `clippy -D warnings` clean.

## 13. Required deliverables

Review request with `## Entry point`. Include §6's six acceptance items with
observed evidence, §4's answer to the open question, and the list of documents
corrected under §5.

**If the work turns out larger than it looks, stop and report rather than
pushing through.** A partial, well-described result is more useful here than a
large diff that bundles RFC 048's work in with it.

Report observed output; a check not run is recorded as not run.

## 14. Handoff checklist

The 17 items the architect's operating instructions require of every Developer Handoff, and where each is covered here.

| Required item | Covered in |
|---|---|
| Task title | Heading |
| Purpose | §1 |
| Background | §1, §3 |
| Applicable RFC and requirements | Authorization banner and companion line, §2 |
| Change scope | §7 |
| Explicit non-change scope | §8 |
| Required implementation | §2, §4 |
| Required tests | §6 |
| Required documentation updates | §5 |
| Acceptance criteria | §12 |
| Prohibited shortcuts | §2b, §2c, §8 |
| Relevant code or module boundaries | §7, §2 |
| Compatibility constraints | §9 |
| Security constraints | §10 |
| Known risks | §11 |
| Required evidence | §6.1 |
| Required review-request format | §13 |
