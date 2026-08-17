# RFC 042: Windows Store distribution and the CPU/GPU binary

**Status.** Proposed — **accepted for implementation by the project owner
2026-08-16**. Remains in `rfcs/proposed/` until the work ships, per RFC 000.
**Blocked on [RFC 041](./041-application-data-locations.md).** A packaged arama
cannot write to its own install directory, so it cannot run until data locations
move. Nothing here is actionable before that ships.
**Tracks.** Publish arama to the Microsoft Store as a single listing, and decide
whether the CPU and CUDA builds can become one executable.
**Touches.** Packaging and the release workflow. Possibly `crates/ai`'s CUDA
feature plumbing. No product logic.

## Summary

arama publishes five executable assets per release, two of them Windows: a CPU
build and a CUDA build. The Store has no "has an NVIDIA GPU" targeting
dimension, so shipping both capabilities under **one listing** requires either
one executable that adapts at runtime, or one package containing both with a
selector.

**The runtime decision already exists.** `crates/ai/src/model/model_manager.rs:13`:

```rust
pub fn device() -> Device {
    Device::new_cuda(0).unwrap_or(Device::new_metal(0).unwrap_or(Device::Cpu))
}
```

CUDA → Metal → CPU, chosen at runtime, with no `#[cfg]` gating. The same line
runs in both builds, and the CUDA build already falls back to CPU when no GPU is
present.

**What forces two binaries is linking, not logic.** `candle-core` 0.11 pins
`cudarc` with `dynamic-linking`, so the CUDA executable carries an import-table
dependency on the CUDA runtime libraries. On a machine without them the
**process fails to start** — the fallback never runs.

## Why the Store, stated as the owner did

Recorded because it shapes what counts as an acceptable answer:

- **Trust.** arama reads a user's whole photo library and runs AI over it
  locally. "Microsoft audited this" is credibility no README provides, and it is
  the strongest argument — a visitor who cannot read Rust has no other way to
  check the no-egress promise.
- **A reduced listing looks wrong.** A CPU-only Store entry beside a fuller
  GitHub release reads as deliberately hobbled, and advanced users will ask why.
- **Installation is easy**, which matters for reach.

**Therefore CPU-only publication is rejected as an option.** The Store listing
must be the real application.

GitHub releases remain the primary channel and are unchanged by this RFC; the
Store is an addition for Windows users.

## Phase 0 — a fact that decides the design

**Can `cudarc` load CUDA at runtime rather than linking it?**

`cudarc` distinguishes `dynamic-linking` (bind at link time — what candle uses)
from runtime loading. If runtime loading is available *and* candle can be built
with it, arama gets **one executable, one listing, no launcher, no bundling**,
and every option below becomes unnecessary.

**This has not been established.** It requires reading `cudarc`'s feature set
and determining whether candle 0.11 can be made to use it — candle hardcodes the
feature list, so this may need a `[patch]` or an upstream change. arama has
patched a dependency before (`pt2safetensors`), so the mechanism exists; the
maintenance cost is the question.

**Settle this before choosing among the options below.** Designing around a
guess here would mean building a launcher that a single build flag makes
pointless.

## Options, if Phase 0 says no

### A — Bundle both binaries with a selector

One package, two executables, a shim that picks.

- Size: ~7.0 MB + ~7.7 MB. Negligible for a desktop application.
- **Introduces a failure mode that does not exist today.** A machine with an
  NVIDIA GPU but a broken or outdated driver: detection says "GPU present",
  the CUDA binary is launched, it **fails to load**, and the user sees a process
  that did not start. Making this robust means catching the load failure and
  re-executing the CPU binary — workable, but new machinery whose failure is
  invisible.
- **Creates a fourth distribution shape.** CI verifies five assets — count,
  layout, ffmpeg contract. A bundled package is assembled differently from
  anything the release workflow proves. RFC 030 exists because arama's
  distribution contracts drifted before, and the 0.38.0 cycle cost four tag
  pushes to defects that all looked fine on inspection.
- **Makes a GPU setting mandatory.** If the selector chooses and the user
  disagrees — laptop on battery, GPU busy — there is no escape without one.

### B — Two Store listings

Rejected. It splits ratings, pushes a hardware question onto users at install
time, invites Store review scrutiny for near-duplicate listings, and dilutes the
trust argument that motivates publishing at all.

### C — CPU-only listing

Rejected by the owner, for the reasons under *Why the Store*.

## Goals

- One Store listing, one name, carrying the full application.
- GPU used when available; CPU otherwise — which the code already decides.
- No new silent failure: if the GPU path cannot be used, the user is told or it
  falls back visibly.
- The Store package's provenance is as verifiable as the GitHub assets'.

## Non-goals

- Changing GitHub release distribution.
- macOS or Linux packaging.
- The data-location change. That is RFC 041 and precedes this.
- Making CUDA available where the hardware does not support it.

## Design questions

### 1. Does the Store package come from CI?

RFC 034 established that CI creates releases because an operator-driven channel
silently produced nothing. A hand-assembled Store package reintroduces exactly
that shape.

**Recommendation: whatever is submitted is built and verified by the same
workflow that builds the assets**, even if submission itself stays manual.

### 2. Is GPU use a user setting?

Today it is automatic with no control. Option A makes a setting necessary;
Phase 0 succeeding makes it optional but still probably right — "use GPU when
available" is a reasonable thing for a user to want to turn off.

### 3. What capability does the manifest declare?

arama reads arbitrary user-chosen directories. On Windows that is
`broadFileSystemAccess`, which Store review scrutinises. **This should be
checked early** — a capability the reviewer rejects would invalidate the whole
plan, and it is cheaper to learn now than after packaging work.

## Testing and verification

- **A packaged build that runs**, on a machine without CUDA and on one with it.
- Under option A: the broken-driver case specifically. A selector that has never
  met a failing GPU has not been tested.
- The ffmpeg artifact-absence contract, applied to whatever the Store package
  contains — RFC 032's guarantee does not stop at the GitHub channel.

## Risks

- **Phase 0 is the design.** Answering it wrong, or skipping it, produces work
  that a build flag would have made unnecessary.
- **Store review is an external gate** on a schedule arama does not control.
  Nothing here should be sequenced as though approval is certain.
- **A fourth distribution shape** — option A's real cost, and the one this
  project has evidence about.

## Open questions

- Does the Store submission need its own version scheme? Store versioning is
  four-part (`1.2.3.4`) and arama uses three-part `X.Y.Z`, with RFC 034 Part F
  asserting the manifest must equal the tag. That interaction is unexamined.
- Does packaging change what "arama" is called? The listing name, the executable
  name, and the RFC 030 asset naming are three different things today.
