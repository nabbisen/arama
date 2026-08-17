# RFC 042 Handoff — Windows Store distribution and the CPU/GPU binary

Companion to [RFC 042](../proposed/042-windows-store-distribution.md), which is
**accepted for implementation** (owner, 2026-08-16) and stays in
`rfcs/proposed/` until the work ships, per
[RFC 000](../done/000-rfc-lifecycle-policy.md).

## 1. Phase 0 can start now. Everything else cannot.

**Blocked on [RFC 041](../proposed/041-application-data-locations.md)** — a
packaged arama cannot write its models or cache until data locations move, so
there is nothing to package.

**But Phase 0 is pure research and is not blocked.** Do it first, in parallel
with RFC 041 if that is convenient, because its answer decides what the rest of
this RFC even is.

## 2. Phase 0 — one question, and it is the design

**Can `cudarc` load CUDA at runtime rather than binding it at link time?**

If yes, and candle can be built that way, arama gets **one executable, one
listing, no launcher, no bundling** — and §3's options become unnecessary.

What is established: `candle-core` 0.11 pins `cudarc` with a hardcoded feature
list including `dynamic-linking`, which binds the CUDA libraries at link time and
gives the executable an import-table dependency. That is why the CUDA build
cannot start without CUDA present, and why the existing runtime fallback in
`ModelManager::device()` never gets to run.

What is **not** established, and what to find out:

1. Does `cudarc` offer a runtime-loading mode at the version candle uses?
2. Can candle 0.11 be built with it — a feature toggle, a `[patch]`, or only an
   upstream change?
3. What does a binary built that way do on a machine with no CUDA? **Starting is
   necessary but not sufficient** — it must also fall back to CPU cleanly rather
   than failing at first inference.

**Answer by building and running, not by reading.** A crate's feature table says
what is expressible, not what works. arama has patched a dependency before
(`pt2safetensors`), so the mechanism exists if a patch is needed; the
maintenance cost of carrying one is a real input to the decision, not a detail.

**Report before proposing an option.** If Phase 0 succeeds, most of this RFC is
moot and should be said so plainly rather than implemented around.

## 3. If Phase 0 says no

Option A — bundle both binaries with a selector — is the only survivor; B and C
are rejected in the RFC with reasons.

Do **not** start building it on a Phase 0 "probably not". The difference between
one binary and two-in-a-trench-coat is worth a definite answer.

If it comes to A, the two things that matter most:

- **The broken-driver case is the test.** A machine with an NVIDIA GPU and an
  outdated driver: detection says GPU present, the CUDA binary is launched, it
  fails to load. A selector that has only ever met working GPUs and absent GPUs
  has not been tested. Catch the load failure and re-exec the CPU binary.
- **A bundled package is a fourth distribution shape** that CI does not
  currently verify. RFC 034 exists because an operator-assembled release channel
  silently produced nothing. Whatever is submitted should be built by the same
  workflow that builds the assets.

## 4. Two things to check early, because either could invalidate the plan

**4.1 `broadFileSystemAccess`.** arama reads arbitrary user-chosen directories.
Store review scrutinises this capability. **Establish whether it is grantable for
an app of this kind before packaging work begins** — a rejected capability
invalidates everything downstream, and learning it late wastes the whole effort.

**4.2 Version scheme.** Store packages use four-part versions (`1.2.3.4`); arama
uses three-part `X.Y.Z`, and RFC 034 Part F asserts the manifest must equal the
tag. That interaction is unexamined. It is not hard, but it is exactly the class
of mismatch this project has repeatedly had to correct, and it should be settled
deliberately rather than discovered at submission.

## 5. Non-change scope

- GitHub release distribution. Unchanged.
- macOS and Linux packaging.
- Data locations. RFC 041.
- `ModelManager::device()`'s fallback chain — it is already correct.
- Making CUDA work where the hardware cannot.

## 6. Deliverables

**For Phase 0**, a review request carrying the answer and the evidence for it —
what was built, what it did on a machine without CUDA, and whether a patch is
required. That package alone may close most of this RFC.

Everything beyond Phase 0 waits for both that answer and RFC 041 shipping.

Report observed output; a check not run is recorded as not run.
