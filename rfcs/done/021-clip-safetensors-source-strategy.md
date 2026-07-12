# RFC 021 - CLIP SafeTensors source strategy

**Status.** Implemented (Unreleased)
**Tracks.** Follow-up after RFC 020: decide whether arama should continue
runtime PyTorch-to-SafeTensors conversion for CLIP or move to a pinned
SafeTensors artifact source.
**Touches.** `crates/ai/Cargo.toml`,
`crates/ai/src/model/model_container.rs`,
`crates/ai/src/model/model_container/clip.rs`,
`crates/ui/main/src/core/components/setup/downloader/util.rs`,
`crates/ui/widgets/src/dialog/settings_dialog/tab/ai_settings/update.rs`,
`crates/ui/layout/src/footer/model_loader.rs`,
`rfcs/notes/audit-warning-burn-down.md`, `CHANGELOG.md`.

## Summary

RFC 020 modernized first-party Candle use to 0.11, but `pt2safetensors` 0.1.3
still owns a transitive `candle-core` 0.10.2 dependency. That converter remains
because arama currently downloads CLIP from the pinned
`openai/clip-vit-base-patch32` Hugging Face revision as `pytorch_model.bin`,
verifies its SHA-256 digest, converts it locally, and then loads the generated
`model.safetensors`.

This RFC proposes a design decision before implementation:

1. Prefer eliminating runtime PyTorch conversion if a trustworthy pinned
   SafeTensors artifact can be selected and verified.
2. Keep the current converter path if no acceptable SafeTensors source exists.
3. Require embedding-regression evidence before changing the CLIP artifact
   source.
4. Treat `pt2safetensors` removal as an outcome, not as the primary goal.

## Why

The current design has useful safety properties:

- arama pins the CLIP repository revision;
- arama verifies the PyTorch artifact SHA-256 before conversion;
- arama stores and loads SafeTensors locally after conversion;
- the OpenAI CLIP repository is the canonical model source already documented
  by arama.

But it also has costs:

- first run must perform model conversion after download;
- conversion failures are part of setup reliability;
- `pt2safetensors` pins `candle-core` 0.10 and `safetensors` 0.7, leaving a
  duplicate Candle stack after RFC 020 moved first-party Candle use to 0.11;
- the converter path keeps the `paste` audit warning connected to both
  first-party Candle 0.11 and transitive Candle 0.10 paths.

The pinned OpenAI CLIP Hugging Face revision used today still publishes
`pytorch_model.bin` and does not present a `model.safetensors` file in the
model tree observed during planning. A replacement SafeTensors source would
therefore be a source/provenance decision, not a filename change.

## Design

### Part A - Artifact source policy

An implementation may remove `pt2safetensors` only if it selects a CLIP
SafeTensors source that satisfies all of these requirements:

- repository and revision are pinned;
- primary artifact SHA-256 is pinned in source;
- source provenance is documented in a permanent note or RFC implementation
  section;
- license compatibility is checked against the current OpenAI CLIP model use;
- model architecture remains `clip-vit-base-patch32` compatible with the
  existing Candle `ClipConfig::vit_base_patch32()` loader;
- the artifact can be loaded directly by current `VarBuilder` code without
  embedding-shape changes.

Acceptable source shapes include:

- an upstream repository that publishes equivalent SafeTensors weights with
  clear provenance;
- an owner-managed mirrored SafeTensors artifact generated from the currently
  pinned PyTorch file, if the mirror process and digest are documented;
- no change, if neither source shape is trustworthy enough.

### Part B - Regression evidence

Changing the CLIP artifact source must include focused regression evidence:

- the model loads successfully on the default CPU path;
- existing image/video similarity tests still pass;
- at least one fixed fixture image produces a stable embedding shape and finite
  values;
- if the implementation can preserve a small expected-score fixture without
  storing large model data in the repository, it should do so;
- CUDA and Metal evidence should be recorded as run or not run, matching the
  RFC 020 pattern.

The implementation does not need to prove bit-for-bit equality against the old
converted file unless it claims equivalence. If it claims equivalence, it must
show how the equivalence was tested.

### Part C - Download and setup behavior

If arama moves CLIP to a direct SafeTensors source:

- `SourceUrl::PyTorch` should be removed only if no other model uses it;
- `ModelContainer::ensure_safetensors()` should become a no-op or disappear;
- setup download progress should save the primary artifact directly to
  `model.safetensors`;
- conversion-specific error copy should be removed or narrowed to legacy
  cleanup paths;
- existing first-run checksum-failure behavior must remain unchanged.

If arama keeps runtime conversion:

- leave `pt2safetensors` and the duplicate Candle 0.10 path documented in the
  audit note;
- consider improving conversion-specific error reporting only if it is needed
  for setup clarity;
- do not force a dependency replacement merely to reduce lockfile duplication.

### Part D - Cache and compatibility

Existing user caches contain embeddings, not model source metadata. A CLIP
artifact-source change should not change cache schema. If embeddings may differ,
the implementation must document whether users should rebuild the AI cache or
whether the current cache remains acceptable.

The default implementation should avoid automatic cache invalidation unless
tests or manual evidence show that embeddings are incompatible enough to make
existing similarity results misleading.

## Touches in detail

### `crates/ai/Cargo.toml`

Remove `pt2safetensors` only after a direct SafeTensors source is selected.
Otherwise leave the dependency unchanged.

### `crates/ai/src/model/model_container.rs`

Remove or simplify `SourceUrl::PyTorch`, `pytorch_path()`, and
`ensure_safetensors()` only if no runtime conversion path remains.

### `crates/ai/src/model/model_container/clip.rs`

Update CLIP source URL, revision, and SHA-256 only after artifact provenance is
settled. Keep the current pinned OpenAI PyTorch source if the design outcome is
to retain conversion.

### Setup and settings UI callers

Remove conversion-only error paths only when conversion is removed. Otherwise
keep the current recoverable setup behavior.

### `rfcs/notes/audit-warning-burn-down.md`

Update the note with the observed dependency graph after implementation. Do not
claim the `paste` warning is resolved unless `cargo audit` confirms it.

## Non-goals

- No change to CLIP model architecture.
- No move to a larger, multilingual, or different embedding model.
- No cache schema migration.
- No broad AI scoring redesign.
- No unpinned model download.
- No pre-release dependency adoption.
- No release action or RFC lifecycle movement.

## Risks

- A non-canonical SafeTensors artifact could change embeddings or provenance
  trust. Mitigation: require source/revision/digest documentation and focused
  embedding evidence.
- Removing conversion may make rollback harder for existing installs.
  Mitigation: preserve setup failure behavior and document whether old local
  model files remain usable.
- Keeping conversion preserves duplicate Candle dependencies. Mitigation:
  record that as an intentional boundary until a trustworthy artifact source is
  approved.
- Cache invalidation could be over-applied. Mitigation: avoid automatic cache
  invalidation unless evidence shows existing embeddings are misleading.

## Test plan

Required default gates:

```sh
cargo fmt --all --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo audit
```

Dependency graph checks:

```sh
cargo tree -i pt2safetensors
cargo tree -i candle-core@0.10.2
cargo tree -i candle-core@0.11.0
cargo tree -i paste@1.0.15
```

Artifact checks, if the CLIP source changes:

- verify the new artifact SHA-256 against the pinned constant;
- load CLIP on the CPU path;
- run existing AI/video regression tests;
- record CUDA and Metal checks as run or not run;
- document any cache rebuild recommendation.
