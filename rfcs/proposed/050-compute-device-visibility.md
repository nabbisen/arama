# RFC 050: Which device is actually running the model, and saying so

**Status:** **Accepted by the project owner, 2026-10-21** (*"All of them are
authorized."*). No implementing task has been written yet — acceptance of the
RFC is not authorization of work, per the project's standing rule.
**Raised by:** the portfolio review of 2026-10-21
([`design-philosophy-review-2026-10-21`](../notes/design-philosophy-review-2026-10-21.md)),
lifted out of [RFC 042](./042-windows-store-distribution.md)'s design question 2
at the owner's direction
**Relates to:** [RFC 042](./042-windows-store-distribution.md),
[RFC 017](../done/017-visible-recoverable-error-ux.md),
[RFC 018](../done/018-ai-video-pipeline-resilience.md)

## Summary

arama picks its compute device in one line and never tells anyone which one it
picked.

```rust
// crates/ai/src/model/model_manager.rs:13
Device::new_cuda(0).unwrap_or(Device::new_metal(0).unwrap_or(Device::Cpu))
```

**Verified 2026-10-21:** nothing in `crates/ui/` or `app/src/` surfaces the
result, and no key in `crates/i18n/` mentions a device, CUDA, Metal or GPU.

Two distinct defects fall out of that line, and they fail different principles.

**1. The user cannot find out what is running.** Someone who installed arama to
use their GPU has no way to confirm it is being used. The only symptom of CPU
fallback is that indexing is slower than they expected, with nothing to
attribute it to. That is the owner's second principle — *users must not be
confused or misunderstand* — failing on a question the user is entitled to ask.

**2. A GPU failure is indistinguishable from a GPU absence.** `unwrap_or`
discards the error. A machine with a broken or outdated CUDA driver, a busy
device, or a version mismatch takes exactly the same path as a machine with no
NVIDIA hardware at all: silently, to CPU. **A degradation nobody is told about
is not robust**, and recovering from it is impossible when its existence is
never reported.

## 1. Why this is its own RFC and not RFC 042's

RFC 042 raises it as *"Design question 2: Is GPU use a user setting?"*, framed
around Store packaging — whether bundling two binaries would force a control to
exist.

**But the defect does not depend on the Store at all.** It is present today, on
every platform, in every build, including the GitHub release assets. RFC 042 is
blocked on an upstream `candle` pull request that has had no maintainer
response, so as things stand that framing ships nothing, and a defect that
exists now is waiting behind a decision about a distribution channel.

RFC 042 keeps the *packaging* question. This RFC takes the *visibility* defect.

## 2. What the line actually hides

| Machine | What happens | What the user sees |
|---|---|---|
| NVIDIA GPU, working driver | CUDA | nothing |
| NVIDIA GPU, broken/outdated driver | **CUDA init fails → CPU** | nothing |
| Apple Silicon | Metal | nothing |
| Apple Silicon, Metal unavailable | **Metal init fails → CPU** | nothing |
| No GPU | CPU | nothing |

Rows 2 and 4 are the ones that matter. They are failures, they are recoverable
by the user (update the driver, free the device), and arama throws the
information away before anyone can act on it.

## 3. Design

**3.1 Resolve the device once, keep the outcome, and keep the reason.** Replace
the nested `unwrap_or` with a resolution that records which device was selected
**and** what happened to each one it tried. The fallback order — CUDA, Metal,
CPU — does not change; only whether the attempt's outcome survives.

**3.2 Surface it in AI settings.** That tab is already the place where the AI
subsystem reports what it is doing: `settings.ai.clip_ready`,
`settings.ai.wav2vec2_error`, `settings.ai.ffmpeg_checking`,
`settings.ai.ffmpeg_ready`. A device line belongs with them, and the precedent
for distinguishing ready from errored is already set there.

**Two states, not one.** *"Running on CPU"* and *"Running on CPU — GPU was found
but could not be used"* are different facts and must read differently. The
second should carry whatever the driver reported, in the spirit of
[RFC 017](../done/017-visible-recoverable-error-ux.md): an error the user can
act on is worth showing.

**3.3 Say it in both locales.** New keys in `crates/i18n/src/{en,ja}.rs`. The
`Untranslated strings` CI job already enforces this.

## 4. Non-goals

- **A setting to force CPU or GPU.** That is a control; this RFC is about the
  user being able to *see* what happened. The control is RFC 042's design
  question 2 and stays there — and it is a much easier question to answer once
  the information exists.
- **Changing the fallback order**, or making CUDA work where the hardware or
  driver does not support it.
- **Per-operation device reporting.** One resolved device for the session is
  what the code has and what the user needs.
- **Benchmarking or speed estimates.** Saying *which* device is running is this
  RFC; saying how fast it is would be a different claim needing measurement.

## 5. Acceptance

- The resolved device is visible in AI settings, in English and Japanese.
- **A GPU that was found but failed reads differently from no GPU at all**, and
  the failure carries the reason where the platform gives one.
- No `unwrap_or` silently discards a device-initialisation error.
- Demonstrated on at least one machine where the GPU path is taken and one where
  it is not — a screenshot of each, since this is a UI claim.

## 6. Open question

**Should a GPU-found-but-unusable state be more prominent than a settings tab?**
It is a recoverable failure the user is unaware of, and RFC 017's tiering exists
for exactly this judgement. A toast on first index would reach someone who never
opens settings; it would also fire on every launch for a user who has decided
they do not care. **Recommend starting in settings only**, and revisiting if it
turns out people do not find it — that direction is reversible, and the reverse
is not.
