# External audit, 2026-09-01 — independent verification

An external architect audited arama 0.41.2 at commit `651dc1f`, covering
specifications, documentation and code, and returned 48 findings across four
axes (1 Critical, 8 High, 23 Medium, 15 Low, 1 Info).

**This note records what was checked independently, not what the audit
asserted.** The audit is external input; this project does not accept findings
on self-report any more than it accepts a dev-team package on self-report. The
report itself is at `.git-exclude/tmp/audit/report/` (untracked).

**Overall: the audit is unusually good.** Every quantitative claim tested below
reproduced, most to the digit. Two items needed correction, both minor and
neither affecting a remediation. The auditor also declared a hypothesis they had
formed and dropped after checking it — the RFC 041 Windows `fs::rename`
concern — which is the behaviour this project asks of its own reviewers.

## Confirmed by independent measurement

**A1 — the wav2vec2 encoder is never run.** Re-derived from the shipped weights
at `~/.local/share/arama/model/wav2vec2-base-960h/model.safetensors` by parsing
the safetensors header directly:

| Measure | Audit | Independently measured |
|---|---|---|
| File size | 377.6 MB | **377.6 MB** |
| Tensors total | 212 | **212** |
| Never read | 201 (95.1 %) | **201 (95.1 %)** |
| …of which transformer encoder | 197 | **197** |
| Loaded by code | 13 (18.4 MB) | **11 (18.4 MB)** |

The source is unambiguous: `Wav2vec2Encoder` holds a `FeatureExtractor` and a
`FeatureProjection` and nothing else, and `encode_one` goes
feature-extractor → projection → `mean(1)` → L2-normalise, with the comment
*"A full model would pass this through the Transformer encoder; this skeleton
pools instead."* `wav2vec2_config.rs` has `num_hidden_layers`,
`num_attention_heads`, `intermediate_size` and `layer_norm_eps` all commented
out — the omission was deliberate and is recorded in the code.

**A2 — the conv-0 GroupNorm is skipped.** `config.json` reads
`feat_extract_norm: "group"`; `Wav2vec2Config` does not deserialise that field
at all; `FeatureExtractor::forward` is bare `conv → gelu` with no normalisation;
and `wav2vec2.feature_extractor.conv_layers.0.layer_norm.{weight,bias}` ([512]
each) are present in the file and never read. Confirmed exactly.

**A3** — `points.sort_by(|a, b| a.partial_cmp(b).unwrap())` at
`video_similarity_config.rs:119`, and `get_duration` parses ffprobe output with a
bare `.parse()` and no `is_finite` guard. Rust's `f64::from_str` accepts `nan`
and `inf`. Confirmed.

**B1** — `remove_dir_all` on the cache directory, no confirmation, failure to
`eprintln!` only, and `Task::none()` so no toast and no reload. Confirmed
verbatim.

**B2** — the async block in `handle_cache_require` contains **no `.await`**, so a
future that `Task::abortable()` cancels by dropping completes in a single
`poll()` and cannot be interrupted. Confirmed.

**C2** — the gallery iterates the full map and builds a widget per entry with no
windowing. Confirmed.

**D1** — measured 938 MB of models on disk (CLIP 578 MB, wav2vec2 361 MB;
`model.safetensors` alone is 605,157,852 bytes) against documented "~700 MB"
(`faq.md:20`, `first-run.md:14`), "~750 MB disk space" (`installation.md:13`)
and "~350 MB" for CLIP (`first-run.md:11`), while
`env/src/file_system.rs:1` enforces `MIN_SETUP_DISKSPACE_MB: u16 = 3096`.
Confirmed — the documented requirement is roughly a quarter of the enforced one.

**D3** — `TargetMediaType::default()` sets `include_video: false`;
`settings.md:11` says **On**. Confirmed.

**D5** — `cargo doc --workspace --no-deps --locked` emits exactly the three
reported warnings. Confirmed.

**D6** — all three files exist and are git-tracked with no `mod` declaration
reaching them: `crates/ui/widgets/src/dir_tree.rs`,
`crates/ai/src/pipeline/encode/image/video.rs`, and
`crates/ui/widgets/src/dialog/settings_dialog/tab/ai_settings/output.rs`.
Confirmed.

**D8** — `CHANGELOG.md:4` says `arama-vX.Y.Z.tar.gz`; the workflow produces
`arama-<tag>.tar.gz` and the published 0.41.2 asset is `arama-0.41.2.tar.gz`.
Confirmed.

**B6** — the real data directory contains test artifacts, including
`model/test-progress-multi-chunk-…` and several `.test-*.lock` files.
Confirmed.

**A6** — precise, and the audit was more careful than a skim suggests: the outer
directory map is already a `BTreeMap`; it is the **inner file map**
(`FastHashMap<String, String>`) that renders in hash order, which is exactly what
the proposed fix targets.

**"Of 45 RFCs, none covers model fidelity."** Counted: 43 done + 2 proposed = 45.
The AI-adjacent RFCs are 018 (video pipeline *resilience*), 021 (where CLIP
weights come *from*), 022/023 (dependency choices) and 035/036 (dialog error and
absence states). **None addresses whether the implemented forward pass matches
the model architecture.** The claim holds.

## Corrected

**B11 is wrong.** The finding states that the `lru` 0.16.4 / RUSTSEC-2026-0253
`unsound` advisory "is not in the warning ledger", and the remediation asks for
it to be recorded.

**It is already recorded.** `rfcs/notes/audit-warning-burn-down.md` carries an
`lru` 0.16.4 — added 2026-08-15 entry with the advisory ID, its `unsound`
classification, the full dependency path
(`lru <- cryoglyph <- iced_wgpu <- iced_renderer <- iced`), the analysis that it
is reachable only through the iced rendering stack with no workspace dependency,
the revisit condition, and an explanation of why it post-dated the previous
refresh. That is more than the remediation asks for.

The auditor evidently treated RFC 027 as the ledger. RFC 027 is the *refresh
that established* the ledger; the living document is the note. **No action.**

**A1's loaded-tensor count is 11, not 13.** The two extra are
`conv_layers.0.layer_norm.{weight,bias}` — which A2 correctly says are *unread*.
So A1 and A2 disagree with each other by two tensors. The 18.4 MB figure is
unaffected (those two tensors are ~4 KB), and no remediation changes.

**The test-count discrepancy corroborates rather than contradicts.** The audit
reports 219 passed / 1 failed / 9 ignored; a clean run here gives **261 passed /
0 failed / 9 ignored**. `cargo test --workspace` stops at the first failing test
target, so their run never reached the remaining ~42 tests. The 9 ignored match
exactly. Their failure was real.

## Confirmed structurally, rate not reproduced

**B7 — the global-locale race is real; the 7 % figure is not reproduced here.**
`crates/i18n/src/lib.rs:75` is a process-global `static LOCALE_ID: AtomicU8`, and
`crates/ui/layout/src/footer/view.rs` has four `#[test]` functions making six
`set_locale` calls, one of which iterates every locale while others assert
English strings. On the default multi-threaded harness that is a genuine logical
race.

**30 consecutive runs of `cargo test -p arama-ui-layout` produced 0 failures.**
At a true 7 % rate, seeing zero in 30 has probability ≈ 0.11 — unlikely but not
disqualifying, and flakiness rates are machine- and load-dependent. **The finding
stands on its structure; the rate is unverified.** The remediation is unchanged
either way, and this project has already fixed this exact race twice in `app`.

## What was not independently checked

Everything not listed above, including most Medium and Low findings, the
performance measurements in axis 3, and the comparison statements against
czkawka / digiKam / immich. They are recorded as the auditor's, not as this
project's.

## Related

[[rfc-046-model-fidelity]] · [[rfc-047-indexing-cancellation]] ·
[[rfc-048-library-scale]] · `rfcs/notes/audit-warning-burn-down.md` ·
`rfcs/notes/diagnostic-log-grows-without-bound.md`
