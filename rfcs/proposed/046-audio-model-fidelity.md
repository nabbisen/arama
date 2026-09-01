# RFC 046: Audio model fidelity, and the gap that let it survive

**Status:** Proposed
**Raised by:** an external architecture audit of 0.41.2 at `651dc1f`,
2026-09-01, independently verified in
[`external-audit-2026-09-01-verification`](../notes/external-audit-2026-09-01-verification.md)
**Severity:** the audit's only Critical finding
**Relates to:** [RFC 018](../done/018-ai-video-pipeline-resilience.md),
[RFC 021](../done/021-clip-safetensors-source-strategy.md)

## Summary

**arama downloads a 377.6 MB wav2vec2 model and runs 4.9 % of it.** The
transformer encoder — the part that makes wav2vec2 a wav2vec2 — is not
implemented. Audio similarity is computed from a mean-pooled 7-layer CNN and a
linear projection, and four documents describe it as something else.

This RFC does three separable things, deliberately ordered so the cheap ones do
not wait on the expensive one:

1. **Fix what is broken in what already runs** — a skipped normalisation and a
   silent zero-vector on failure. Independent of every decision below.
2. **Make the documentation true today**, whatever route (3) takes. This is the
   only part the audit calls blocking for 0.42.0.
3. **Decide what audio similarity should be.** Owner-reserved. Three routes,
   priced below.

It also adds the gate that would have caught all of this, because the deeper
finding is not the defect — it is that arama's process has never been pointed at
the AI.

## 1. What is actually running

Measured from the shipped weights by parsing the safetensors header directly,
not inferred:

| | |
|---|---|
| File | 377.6 MB, 212 tensors |
| Read by `Wav2vec2Encoder::load` | **11 tensors, 18.4 MB (4.9 %)** |
| Never read | **201 tensors, 359.2 MB (95.1 %)** |
| …of which the transformer encoder | **197 tensors, 359.1 MB** |

`Wav2vec2Encoder` holds a `FeatureExtractor` and a `FeatureProjection` and
nothing else. `encode_one` is:

```
feature extractor (7× conv → gelu) → linear projection → mean over time → L2
```

The code says so itself, at
`crates/ai/src/pipeline/encode/audio/wav2vec2_encoder.rs`:

```rust
// A full wav2vec2 stack would place 12 Transformer blocks here.
// This skeleton keeps the high-level feature extraction flow explicit.
```

```rust
// 3. A full model would pass this through the Transformer encoder;
// this skeleton pools instead.
```

And `wav2vec2_config.rs` has `num_hidden_layers`, `num_attention_heads`,
`intermediate_size` and `layer_norm_eps` **commented out**. This was a
deliberate scaffold, recorded honestly in the code — and then never revisited,
while the documentation moved ahead of it.

**Stated plainly so no one has to infer it:** what ships is not wav2vec2
embeddings. It is a mean-pooled convolutional feature. That is a real audio
representation — it captures spectral and timbral texture, and it is not noise —
but it carries none of the contextual modelling the transformer provides, and it
is not what four documents promise.

## 2. Two defects in what already runs — fix regardless of route

These are bugs in the 4.9 %, not consequences of the missing 95.1 %. They
survive every route below and should not wait for the decision in §3.

**2.1 The conv-0 GroupNorm is skipped (audit A2).** `config.json` declares
`feat_extract_norm: "group"`. `Wav2vec2Config` does not deserialise that field
at all, `FeatureExtractor::forward` is bare `conv → gelu` with no normalisation,
and `wav2vec2.feature_extractor.conv_layers.0.layer_norm.{weight,bias}` (`[512]`
each) sit in the file unread. **So even the 4.9 % is composed wrongly.**

**2.2 A failed segment becomes a zero vector and is averaged in (audit A10).**

```rust
self.encode_one(seg).unwrap_or_else(|_| vec![0.0; self.feature_dim])
```

A zero vector is not a neutral element of a mean — it pulls the result toward
the origin, and it is not unit-length, so it silently violates the invariant the
whole comparison layer assumes. **Failed segments must be excluded from the
mean, not averaged into it**, and a segment set that is entirely failures must
produce an honest absence rather than a zero vector. RFC 035/036 built
`SimilarityReadOutcome` for exactly this distinction; this is the same problem
one layer down.

## 3. The decision — owner-reserved

**This is the part I am not deciding.** Three routes, with what each actually
costs.

### Route A — implement the transformer encoder

12 transformer blocks against the weights already downloaded. The model becomes
what the documentation says.

- **Cost:** large. Attention, layer norms, the positional convolution embedding,
  and `do_stable_layer_norm: false` ordering all have to be right.
- **The real risk is silent wrongness.** A subtly wrong attention implementation
  produces plausible embeddings and plausible similarity scores, and nothing in
  the application can show a user that they are wrong. **This route is only
  viable with §4's golden-vector gate in place first** — otherwise arama would
  be replacing an honest scaffold with a confident one.

### Route B — swap the model

Choose an audio representation arama can run correctly and completely, sized to
what the feature needs.

- **Cost:** medium, and it interacts with [RFC 021](../done/021-clip-safetensors-source-strategy.md)'s
  weight-sourcing strategy and the model registry's generation/manifest
  machinery.
- **Attraction:** the download shrinks, and "the model we ship is the model we
  run" becomes true by construction rather than by review.

### Route C — restate the claim, keep the current representation

Describe what runs, stop calling it wav2vec2 embeddings, and stop paying for the
inert weights.

- **Cost:** small for the documentation. **Not small for the download.**
  Safetensors is a single file — arama cannot fetch 4.9 % of it. Actually
  stopping the 359 MB either needs a stripped artifact hosted somewhere, which
  is a supply-chain and RFC 021 question, or a different model, which is Route
  B. **Restating without re-sourcing means continuing to download 359 MB that
  nothing reads** — honest, but wasteful, and it should be recorded as such
  rather than presented as a clean close.
- **This is the only route that removes the Critical from the user's view this
  week**, and it composes with A and B rather than competing: §3.1 happens
  regardless.

### 3.1 The documentation is corrected now, under every route

`README.md`, `docs/src/dev/architecture.md` and `docs/src/users/faq.md` claim
wav2vec2 audio embeddings (audit D2). Whatever §3 decides and however long it
takes, **those sentences are false today** and cost one afternoon to fix. The
audit is right that this is the blocking half, and it should not be sequenced
behind an engineering decision.

The honest framing is available and is not embarrassing: *arama compares the
audio track's low-level acoustic texture.* That is what it does, it is useful,
and it is defensible.

## 4. The gate — the finding under the finding

The audit's structural observation is sharper than any individual defect:

> Of 45 RFCs, none covers model fidelity. The AI crate's tests are 62 %
> downloader. The result is a Critical defect in the one place the project has
> no process coverage.

Verified: 43 done + 2 proposed = 45 RFCs. The AI-adjacent ones cover pipeline
*resilience* (018), where weights come *from* (021), dependency choices
(022/023), and dialog error and absence states (035/036). **None asks whether
the implemented forward pass matches the architecture.**

This project's reflex — add a gate after a defect escapes — is correct and has
worked every time it has been applied. It has never been applied here, because
**a wrong embedding has no observer.** Every other defect class arama has hit
was caught by a rendered capture, a CI job, or the owner using the application.
A wrong 768-float vector looks exactly like a right one.

**Three gates, smallest first:**

1. **Tensor-set coverage.** Assert that the set of tensors an encoder loads
   equals the set its architecture requires, derived from `config.json`. ~20
   lines. **It falsifies A1 and A2 immediately**, and it is the gate that would
   have made this RFC unnecessary.
2. **Golden vectors.** A fixed input, a reference embedding, an epsilon. Pins
   Route A against silent wrongness, and pins CLIP against the preprocessing
   drift the audit raises as A11.
3. **Invariants.** Every stored vector is unit-length and of the declared
   dimension, asserted at the boundary rather than assumed by comment. Today
   `image.rs:8` carries the assumption as a bare comment: *"Assumption: vectors
   were L2-normalized when cached."*

Gate 1 lands regardless of §3 and should land first — it is the cheapest
falsifier this project has been offered.

## 5. Non-goals

- **CLIP.** Image similarity is verified working; A11 (aspect-ratio distortion
  in preprocessing) is a separate quality question and a payload-version bump,
  not part of this RFC.
- **Deciding §3 in this document.** Owner-reserved, deliberately.
- **The indexing and scale findings.** [RFC 047](./047-indexing-cancellation.md)
  and [RFC 048](./048-library-scale.md).
- **Removing audio similarity as a feature.** Not proposed by anyone; Route C
  keeps it and describes it correctly.

## 6. Acceptance

- §2.1 and §2.2 fixed, with a test that fails without each.
- Gate 1 in place and demonstrated to fail against the current encoder before
  the fix, then pass after.
- No document claims wav2vec2 embeddings unless the encoder runs.
- §3's route chosen by the owner and recorded here as a dated decision block,
  in the form RFC 042 §3b uses.

## 7. Open questions

- **§3's route.** Owner-reserved.
- **If Route C: is downloading 359 MB of unread weights acceptable as an interim
  state, and for how long?** It should be time-boxed rather than left.
- **Does the audio modality earn its cost at all?** Nobody has asked. The
  feature was built because it was interesting; whether users want "find this
  video by its audio" is unmeasured, and Route B's sizing depends on the answer.
