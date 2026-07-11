# CLIP SafeTensors Source Decision

This note records RFC 021's implementation decision for the CLIP model source.

## Decision

Retain the current runtime PyTorch-to-SafeTensors conversion path for CLIP.

`pt2safetensors` remains an intentional dependency because the currently pinned
canonical OpenAI CLIP source is a PyTorch artifact, and no trustworthy direct
SafeTensors source or owner-managed mirror has been selected.

## Evidence

Observed on 2026-07-11.

Current arama CLIP source:

- repository: `openai/clip-vit-base-patch32`
- revision: `3d74acf9a28c67741b2f4f2ea7635f0aaf6f0268`
- artifact URL:
  `https://huggingface.co/openai/clip-vit-base-patch32/resolve/3d74acf9a28c67741b2f4f2ea7635f0aaf6f0268/pytorch_model.bin?download=true`
- pinned SHA-256:
  `a63082132ba4f97a80bea76823f544493bffa8082296d62d71581a4feff1576f`

Inspected Hugging Face file tree:

`https://huggingface.co/openai/clip-vit-base-patch32/tree/3d74acf9a28c67741b2f4f2ea7635f0aaf6f0268`

The tree identifies revision `3d74acf` and lists `pytorch_model.bin`; the
visible file list does not show `model.safetensors`.

Local dependency evidence:

```text
pt2safetensors 0.1.3 <- arama-ai
candle-core 0.10.2 <- pt2safetensors 0.1.3 <- arama-ai
```

## Rationale

Removing `pt2safetensors` would require changing the CLIP artifact source or
introducing an owner-managed converted mirror. That is a supply-chain decision,
not a dependency cleanup. The current source has known provenance, a pinned
revision, and a pinned checksum, while the conversion result is local.

The duplicate Candle 0.10 dependency is therefore accepted as the cost of
keeping the current artifact trust boundary until a better source is reviewed.

## Future Removal Criteria

`pt2safetensors` can be removed only after a follow-up selects a CLIP
SafeTensors source that provides:

- pinned repository or owner-managed mirror;
- pinned revision or immutable artifact identity;
- pinned SHA-256;
- documented provenance from the current OpenAI CLIP model or another accepted
  model source;
- license compatibility confirmation;
- CPU load evidence;
- image/video similarity regression evidence;
- CUDA and Metal evidence recorded as run or not run.
