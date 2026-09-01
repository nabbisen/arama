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

## 2a. Settled 2026-09-01 — the audio modality earns its cost

> **The project owner, asked whether the audio modality is worth its cost:**
>
> *"In my opinion, yes, it does. Video similarity can not effectively calculated
> only with their visual because similar image can exist. The voice can be
> another unique characteristics at this point."*

**This closes §7's second open question and narrows §3.** Audio stays. What
follows from it is more interesting than the answer itself, because the stated
job is **discrimination** — telling apart videos that look alike — and that is a
harder requirement than "produce an audio vector".

### It exposes a second defect, independent of A1 and A2

**The pipeline mean-pools twice.**

1. `encode_one` averages over the time axis *within* each segment
   (`projected.mean(1)`).
2. `mean_embeddings`
   (`pipeline_manager/video_similarity_pipeline.rs:250`) then averages those
   per-segment vectors *across* the whole video.

Segments are 20 seconds (`audio_segment_duration_secs: 20.0`). So a
forty-minute video collapses to **one 768-float vector describing its average
acoustic texture.**

That is close to the worst possible representation for the owner's stated job.
Two unrelated videos that are both "a person speaking indoors" average to
similar textures; the very cases audio is supposed to disambiguate are the ones
mean-pooling erases. **This holds even if the transformer encoder is
implemented perfectly** — Route A alone does not fix it.

### The project already built the right comparison, and it is dead code

`pipeline/score/similarity/video/video_similarity_calculator.rs:114`,
`cross_max_similarity`, is a bidirectional thresholded max-match over **sets** of
segment embeddings. Its own doc comment states the rationale:

> *"This keeps scores stable when videos have opening/ending cuts, inserted
> silence or dark frames, while unrelated videos stay low because every pair has
> a low score."*

Above it, commented out, sits `compare` — the intended design, applying
cross-max to per-segment **image and audio** embeddings and combining them with
`image_weight` / `audio_weight`.

**So the discriminating comparison the owner just described was designed, written
and then bypassed.** It is dead only because the pipeline collapses to a mean
before it can be used.

### The route question is therefore two questions

| | Question | Where it lives |
|---|---|---|
| **Representation** | Does the encoder produce contextual embeddings, or CNN texture? | §3, A1 |
| **Comparison** | Are segments compared as a set, or averaged into one vector? | here, and it is independent |

**Choosing a better model while keeping double mean-pooling would waste most of
the gain.** Either can be done first; neither substitutes for the other.

### One question the owner's answer opens

**Is the job "the same audio, re-encoded" or "similar audio content"?** They are
different problems with different right answers:

- **Same audio** — a re-upload, a different resolution, a remux. **Audio
  fingerprinting** (Chromaprint-style) is built for exactly this, is far more
  accurate at it than any embedding, and needs no 377 MB model.
- **Similar content** — different recordings of similar material. That is what
  learned embeddings are for, and fingerprinting cannot do it at all.

"Similar image can exist" reads like the first, but it is not stated, and the two
lead to different models and different costs. **This should be answered before
§3's route is chosen**, because it can eliminate two of the three routes.

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
- **Weakened by §2a.** Route C as a *terminal* answer means accepting a weak
  discriminator for a capability the owner has now said matters. It remains the
  right *interim* step — the documentation must be true regardless — but it
  should no longer be read as a place to stop.

### 3.2 Settled 2026-09-01 — the mandate, and the design it implies

> **The project owner, on choosing between the routes:**
>
> *"We can revise specs and switch models if it becomes better. I want 'finally
> clean, safe and secure, and robust and sophisticated design'."*

**This is a stronger instruction than picking a route, and it reframes the
question.** If specs and models are revisable, then the thing to get right is not
*which model* but **what seam makes the model a decision we can revisit cheaply**.
A route chosen today under that mandate must not be a route that has to be
unpicked tomorrow.

#### The root cause is narrower than §2a said, and that is good news

`crates/ai/src/pipeline/encode/audio.rs:8-16` — the `AudioEncoder` trait's own
doc comment:

> *"Returns one embedding vector per segment **instead of collapsing all
> segments into one vector**. Callers can then compute cross-max similarity,
> which is robust to opening cuts, ending cuts, and timeline offsets."*

**The encoder already does the right thing.** `encode_segments` returns
`Vec<Vec<f32>>` — the full sequence, one vector per segment, exactly as designed.

The collapse happens downstream, and the reason is the **cache payload shape**:

```rust
// crates/cache/src/core/payload.rs:29
pub(crate) struct VideoPayload {
    pub thumbnail_path: Option<String>,
    pub clip_vector: Option<Vec<f32>>,        // frame-averaged
    pub wav2vec2_vector: Option<Vec<f32>>,    // scene-averaged
}
```

**There is nowhere to put a sequence, so `mean_embeddings` averages one to fit.**

So the design intent is recorded in *three* places — the trait doc,
`cross_max_similarity`'s doc, and the commented-out `compare` — and defeated in
*one*: a payload field that can hold a single vector. The pipeline is not
wrongly designed; it is **truncated by its storage.**

#### The design that follows

1. **Store the sequence.** `VideoPayload` holds per-segment embeddings rather
   than a mean. `VIDEO_PAYLOAD_VERSION` (`engine.rs:38`, currently `1`) bumps to
   `2`, which is the migration mechanism this project already built and already
   uses — a version bump purges stale entries rather than misreading them.
2. **Compare sets, not means.** `cross_max_similarity` already exists, is
   already tested, and its threshold is already a config field
   (`cross_max_similarity_threshold`). This is wiring, not invention.
3. **Mirror the seam on the image side.** `AudioEncoder` is a trait; CLIP is
   concrete. An `ImageEncoder` trait makes both halves swappable, which is
   precisely what *"switch models if it becomes better"* requires.
4. **Then the model choice is cheap and empirical.** With the seam in place,
   Route A and Route B stop being a fork and become experiments that can be run
   and compared on real libraries.

**This is why the mandate narrows §3 without answering it.** The question
*"fingerprinting or embeddings?"* (§2a) no longer has to be answered before
committing — it becomes a swap behind a trait, which is exactly the outcome the
owner asked for.

#### What "sophisticated" must not become

This project's own ruleset warns against the failure mode:

> *"Strike a balance between feature-specific tuning and general-purpose
> flexibility. Avoid creating rigid structures tied too closely to specific
> features, but also avoid vague definitions resulting from over-pursuing
> abstraction."*

**The guard, stated concretely.** Add `ImageEncoder` because a second
implementation is genuinely in prospect and the audio side already proves the
shape. **Do not** build a plugin registry, a runtime-configurable metric, a
generic "similarity strategy" abstraction, or a trait with one implementation
and no candidate second one. Two implementations justify a trait; one does not.

**And "clean" has a cost here that should be said out loud:** storing per-segment
embeddings makes the video cache larger — segments are 20 s, so a long video
holds many vectors where it held one. That is a real trade against RFC 016's
cache capacity work, and it needs measuring rather than assuming.

#### Sequencing

The mandate does not change what happens first. §2's two defects, §3.1's
documentation, and §4's gate are all prerequisites — they cost little, and every
one of them makes the larger design verifiable rather than hopeful. **The gate
especially: none of the above can be evaluated without a way to see that an
encoder loads what it claims to.**

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
- §3.2's design measured, not assumed: the cache-size cost of storing per-segment
  embeddings reported against a real library before the payload version bumps.

## 7. Open questions

- **§3's route.** Owner-reserved.
- **If Route C: is downloading 359 MB of unread weights acceptable as an interim
  state, and for how long?** It should be time-boxed rather than left.
- ~~**Does the audio modality earn its cost at all?**~~ **Answered 2026-09-01 —
  yes.** See §2a, which also records the two defects and the one new question
  the answer exposes.
- **Is the job "the same audio, re-encoded" or "similar audio content"?** §2a.
  **No longer needs answering first** — §3.2's seam turns it into a swap rather
  than a commitment. Still worth answering, because it decides which experiment
  to run first.
- **Does the comparison move to per-segment cross-max?** §2a. Independent of the
  model choice, and `cross_max_similarity` already exists.
