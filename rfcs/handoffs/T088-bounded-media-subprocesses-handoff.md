> **AUTHORIZED 2026-11-18 — owner:** *"Authorized."* Written 2026-11-18 by the architect, as item 5 of the
> 0.44.0 plan (`.git-exclude/schedule/README.md`, owner: *"Authorized."*). The
> plan was not authorization of this task; this banner is.
>
> **Independent of T085–T087 and T089.**

# T088 Handoff — every media subprocess is bounded, and Stop stops the one running

## 1. Purpose

Bound every ffmpeg and ffprobe that indexing runs, in time and in output. Kill
each one with its whole process tree when a bound is hit **or when the user
presses Stop**. Do it through one runner, so the four call sites cannot drift
apart again.

Today a hung ffmpeg in the embedding phase cannot be stopped from the UI. The
external audit's B3 named this on 2026-09-01. RFC 047 §4 said that bounding the
subprocesses *"belongs with this work rather than after it"*, and it was left
over when RFC 047 shipped in 0.43.0.

## 2. Background

**The four places indexing starts a media subprocess** (architect, 2026-11-18):

| Site | Started by | Time bound | Output bound | Tree kill | Stop reaches it |
|---|---|---|---|---|---|
| `crates/cache/src/core/thumbnail.rs` `run_ffmpeg` → `run_command_with_timeout` | the thumbnail phase, up to 2 per video | **30 s** (`FFMPEG_TIMEOUT`) | stdout is null; **stderr is read whole** (`read_to_string`) | **yes** (`group_spawn`, review 161 #4) | no: Stop is seen between chunks (`app/src/core/update/cache.rs:102`), so a chunk's running ffmpegs finish first |
| `crates/ai/src/pipeline/extract/video_extractor.rs` `get_duration` (ffprobe) | the embedding phase, 1 per video | **none** | **none** (`.output()`) | no | no |
| same file, `seek_single_frame` | the embedding phase, 1 per sampled frame | **none** | **none**: stdout is read whole, then checked against `size × size × 3` | no | no |
| same file, `extract_one_audio_segment` | the embedding phase, 1 per audio window | **none** | **none** | no | no |

- **Stop in the embedding phase** is `Task::abortable` (`app/src/core/update/cache.rs:261`
  around `image_embedding`, `crates/ai/src/pipeline/encode/image/embeddings.rs:76`).
  It stops at the next `await`. It cannot interrupt a blocking body, nor a child
  process.
- **The discovery probe** (`crates/engine/sidecar/src/media/video/video_engine.rs:76`,
  RFC 039) already has `probe_timeout` and `attempt_timeout`. It runs at
  start-up and when the ffmpeg setting changes, not during indexing. Check
  whether its output is bounded. Move it onto the runner only if that is a
  clean fit, and say which you chose.
- **`command-group` 5** is already a dependency of `arama-cache` and
  `arama-sidecar`. It is what `group_spawn` comes from: a process group on Unix,
  a job object on Windows. T076 proved the Windows grandchild reaping on the
  runner.
- **Crate graph:** `arama-ai` depends on `arama-cache` and `arama-sidecar`;
  `arama-sidecar` depends only on `arama-env`; `arama-cache` does not depend on
  `arama-sidecar`.

## 3. Applicable RFC and requirements

[RFC 047](../done/047-indexing-cancellation.md) §4 (this hazard) and §7
(cancellation is cooperative and partial: a cancelled file is not a failed
file). RFC 017 (errors stay visible). RFC 032 (arama never obtains ffmpeg for
the user; unchanged here). The handoffs README rules.

## 4. Change scope

- `crates/engine/sidecar/` (the runner, its tests)
- `crates/cache/Cargo.toml`, `crates/cache/src/core/thumbnail.rs`,
  `crates/cache/src/core/video.rs` (and its `upsert_all` caller, to carry Stop)
- `crates/ai/src/pipeline/extract/video_extractor.rs`, and the pipeline above it
  (`crates/ai/src/pipeline_manager/video_similarity_pipeline.rs`,
  `crates/ai/src/pipeline/encode/image/embeddings.rs`), to carry Stop
- `app/src/core/update/cache.rs` (pass Stop's flag down; read the new
  "cancelled" outcome)
- `Cargo.lock` only if a crate gains a workspace dependency it already has
- `CHANGELOG.md` `[Unreleased]`

## 5. Explicit non-change scope

- What ffmpeg is asked to do: the arguments, timestamps, sample rates and frame
  sizes stay exactly as they are.
- Toolchain discovery and validation (RFC 039), except as §2 says for the probe.
- The image (non-video) path: it starts no subprocess.
- No new dependency. `command-group` is already there.
- The UI, beyond what Stop already shows.

## 6. Required implementation

1. **One runner, in `arama-sidecar`**, the crate that owns the toolchain. Its
   shape is your design; it must give:
   - a **time limit**: the child, and its tree, is killed when the limit passes;
   - an **output limit** on stdout: past the limit, kill the child and return an
     "output too large" error, never a truncated success;
   - a **bounded stderr**: drain it fully, so the pipe never blocks the child,
     but **keep** at most a fixed amount (for example the last 16 KiB) for the
     error message;
   - **cancellation**: a shared flag (an `Arc<AtomicBool>` like the thumbnail
     phase's `cancel_flag`, or a type of your own) is checked on every poll.
     When it is set, kill the tree and return "cancelled", distinct from failure
     and from timeout;
   - **`group_spawn`** for every child.

   `arama-cache` then depends on `arama-sidecar` (no cycle; §2). Delete
   `run_command_with_timeout` from `thumbnail.rs` once it uses the runner, and
   move its tests to the runner.
2. **Limits per call**, as named constants with a one-line reason each:

   | Call | Time | stdout |
   |---|---|---|
   | ffprobe duration | short, for example 10 s | small, for example 4 KiB |
   | one frame | for example 30 s, as thumbnails | **exactly** `size × size × 3`: one byte more is too large |
   | one audio window | for example 30 s | `ceil(duration × sample_rate) × 4` bytes (f32, mono) plus a small margin; say why the margin is needed, or use none |
   | thumbnail | 30 s, unchanged | null, unchanged |

   **Measure** real durations on the fixtures (`.git-exclude/fixtures/`) and
   report them, so each limit has observed headroom behind it.
3. **Stop reaches the running child.** Carry the indexing cancel flag:
   - into the thumbnail phase's video thumbnails (through `upsert_all`, or
     however the call reaches `generate_video_thumbnail`);
   - into the embedding phase (`image_embedding` → the video pipeline → the
     extractor). The embedding phase gets a flag like the thumbnail phase's,
     set by the same Stop.
   A cancelled file is **not** recorded as a failure, and **not** cached as
   empty (RFC 047 §7). The next run indexes it again.
4. **Errors stay visible** as today (the failure counts). A timeout or too-large
   output is logged with the kept stderr and the limit that was hit.

## 7. Required tests

- **Runner tests (Tier 1)**, with stand-in commands (`sh` / `cmd`, as
  `thumbnail.rs`'s tests already do), never a real ffmpeg:
  - a child that sleeps past the limit returns "timed out" within limit + one
    poll, and is gone afterwards;
  - a child that writes more than the stdout limit returns "too large", not
    success;
  - a child that floods stderr completes without deadlock, and the kept stderr
    is bounded;
  - setting the flag mid-run returns "cancelled" within one poll, and the child
    is gone;
  - **a grandchild is killed too** on timeout and on cancel. Unix runs locally;
    the Windows job object and macOS run in CI, as T076's tests do.
- **Extractor tests:** a frame of the wrong size is still rejected (Task 040's
  rule), now through the runner.
- **One real Stop:** index the fixture video folder through
  `scripts/gui-session.sh`, press Stop while the embedding phase is running, and
  record the time from Stop to idle. Do it before and after this change, on the
  same profile setup. Then confirm that the next run indexes the stopped files.
- Gates, all `--locked`: `fmt`, `check`, `clippy -D warnings`,
  `test --workspace` (isolated), `audit --deny warnings`, T066's script.

## 8. Required documentation updates

`CHANGELOG.md` `[Unreleased]`: Stop now also stops a video being read, at once;
a video that hangs ffmpeg is given up after a limit instead of holding indexing.
Plus any user doc that describes Stop's timing (`grep -rn -i stop docs/src/users`).

## 9. Acceptance criteria

- All four indexing call sites go through one runner, with a time limit, a
  stdout limit, bounded stderr, tree kill and cancellation.
- Stop kills a running ffmpeg in both phases; stopped files are re-indexed next
  time, not recorded as failed.
- The limits rest on measured durations.
- All tests above pass, including the grandchild test on all three CI systems.

## 10. Prohibited shortcuts

- A timeout without a tree kill, or a kill of only the direct child.
- Truncating stdout and returning success.
- Reading stderr whole, anywhere.
- Treating "cancelled" as "failed", or caching a cancelled file.
- A real ffmpeg in a unit test; a fake ffmpeg in the user's settings.
- Running any arama binary except through `cargo test`, `cargo run` or
  `scripts/gui-session.sh`.

## 11. Module boundaries

The runner lives in `arama-sidecar`. `arama-cache` and `arama-ai` call it.
`app` only passes the flag and reads the outcome. No UI crate changes.

## 12. Compatibility constraints

Linux, macOS, Windows. `group_spawn` behaves differently per system (a process
group, or a job object); the grandchild test must run on each, in CI.

## 13. Security constraints

The point of the task, in part: an input file must not be able to make arama
hang, or make it hold unbounded memory, through ffmpeg. The arguments stay as
they are, so nothing about what is executed changes.

## 14. Known risks

- **A limit too tight fails real videos.** Hence the measured durations (§6.2),
  and a limit well above them.
- **Polling cost:** the existing poll interval (50 ms) is fine. Keep one
  interval for all calls.
- **Windows:** killing a job object can race the child's own exit. Both
  outcomes must be handled; T076's tests show the pattern.

## 15. Required evidence

### 15.1 Evidence plan

| Question | Level | Evidence |
|---|---|---|
| Does the runner hold every bound? | unit | the runner tests, incl. grandchild, on Linux locally and on all three CI systems |
| Do the limits fit real files? | machine | measured durations on the fixtures, per call kind |
| Does Stop stop a running ffmpeg? | real system | time from Stop to idle, before and after; the next run re-indexes |
| Did anything regress? | machine | the gates |

No capture needed: the evidence is timings and test output.

### 15.2 Deliverables

The commit, and the rows above, observed.

## 16. Review-request format

A package under `.git-exclude/review-request/` with a `## Entry point` section:
the commit, the `git show` command, and full repo-relative paths, one per line,
to the evidence files. Report observed output; a check not run is recorded as
not run. You cannot push: report the commit, and the architect pushes.

## 17. Handoff checklist

| Required item | Covered in |
|---|---|
| Task title | Heading |
| Purpose | §1 |
| Background | §2 |
| Applicable RFC and requirements | §3 |
| Change scope | §4 |
| Explicit non-change scope | §5 |
| Required implementation | §6 |
| Required tests | §7 |
| Required documentation updates | §8 |
| Acceptance criteria | §9 |
| Prohibited shortcuts | §10 |
| Relevant code or module boundaries | §11 |
| Compatibility constraints | §12 |
| Security constraints | §13 |
| Known risks | §14 |
| Required evidence | §15 |
| Required review-request format | §16 |
