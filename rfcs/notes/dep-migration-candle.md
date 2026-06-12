# Migration report: candle-core / candle-nn / candle-transformers 0.9.2 → 0.10.2

**Verdict: no migration effort required for the CPU inference path.
Update is a drop-in.**

## What changed

### candle-core

| | |
|---|---|
| Items removed | 0 |
| Items added | 1 (`TokenizerFromGguf`) |

`TokenizerFromGguf` enables loading tokenizer data embedded inside
GGUF model files. arama uses PyTorch/SafeTensors format exclusively;
this addition is irrelevant.

The new `0.10.2` manifest adds `tokenizers` as an optional dependency
(for `TokenizerFromGguf`). It is gated behind a feature flag and is not
compiled unless explicitly requested.

### candle-nn

| | |
|---|---|
| Items removed | 0 |
| Items added | 1 (`remove_mean`) |

`remove_mean` is a normalization helper. arama does not use it.

### candle-transformers — CLIP model

The CLIP module (`models/clip/mod.rs`) is identical between 0.9.2 and
0.10.2: same struct definitions, same method signatures, same variant
names.

| Symbol | In 0.9.2 | In 0.10.2 |
|---|---|---|
| `ClipConfig` | ✓ | ✓ |
| `ClipModel` | ✓ | ✓ |
| `ClipTextConfig` | ✓ | ✓ |
| `ClipVisionConfig` | ✓ | ✓ |
| `Activation::QuickGelu` | ✓ | ✓ |

### candle-core symbols used by arama-ai

| Symbol | In 0.9.2 | In 0.10.2 |
|---|---|---|
| `Module` (trait) | ✓ | ✓ |
| `Tensor` | ✓ | ✓ |
| `DType` | ✓ | ✓ |
| `Device` | ✓ | ✓ |
| `IndexOp` | ✓ | ✓ |

### candle-nn symbols used by arama-ai

| Symbol | In 0.9.2 | In 0.10.2 |
|---|---|---|
| `VarBuilder` | ✓ | ✓ |
| `Conv1d` / `conv1d` / `conv1d_no_bias` | ✓ | ✓ |
| `LayerNorm` / `layer_norm` | ✓ | ✓ |
| `Linear` / `linear` | ✓ | ✓ |
| `Conv1dConfig` | ✓ | ✓ |

## How to apply

In `crates/ai/Cargo.toml`:

```toml
# was:
candle-core = "0.9"
candle-nn = "0.9"
candle-transformers = "0.9"

# becomes:
candle-core = "0.10"
candle-nn = "0.10"
candle-transformers = "0.10"
```

Apply the same change to the `[target.'cfg(...)'.dependencies]` platform
sections for macOS and CUDA (if present). Then `cargo update`.

No source changes needed.

## Runtime note

This migration covers source-level API compatibility only. The model
weight files (`.safetensors`, PyTorch `.pth`) are unchanged; no
re-download is necessary. If unexpected numerical differences appear
during inference, run the similarity pipeline against a known reference
directory and compare scores. In practice, minor candle versions have
not historically changed inference numerics on the CPU path.
