# Cache Serialization Dependency Decision

This note records RFC 023's implementation decision for the remaining
`localcache` -> `bincode` audit-warning owner.

## Decision

Retain the current `localcache` 0.20.0 cache engine and its bincode-backed
payload codec for now.

No arama cache payload codec, namespace, payload version, cache schema,
workspace patch, or cache-engine replacement changes in this implementation
batch.

## Evidence

Observed on 2026-07-11.

Current dependency owner path:

```text
bincode 2.0.1 <- localcache 0.20.0 <- arama-cache
```

`cargo info localcache@0.20.0` reports 0.20.0 as the current published crate
version in this environment.

The installed `localcache` 0.20.0 crate source shows:

- `json = ["dep:serde_json"]` is available as an optional feature;
- `bincode = "2"` is declared as an unconditional dependency;
- `Codec::Bincode` is the default codec;
- `Codec::Json` is behind the `json` feature;
- bincode serialization uses `bincode::config::legacy()` for wire-format
  compatibility.

A local upstream checkout at
`/home/nabbisen/Desktop/__dev__/dev-crates-lib/localcache-rs/localcache-rs-git`
was inspected read-only. It was clean and matched the same dependency shape:
`bincode` remains unconditional and no bincode-free codec/dependency route is
present in that checkout.

## Rationale

Switching arama to `localcache::Codec::Json` in 0.20.0 would change cache
payload encoding without removing the `bincode` audit warning, because
`localcache` still compiles `bincode` unconditionally. That would take cache
compatibility risk without achieving the dependency goal.

Adding a workspace patch to arama would make release policy ambiguous unless
the owner explicitly accepts a patched-upstream release strategy. This batch
therefore includes no workspace patch. Any future patch should be treated as
validation-only unless the owner approves it for release.

Replacing `localcache` is also intentionally deferred. It would reopen cache
schema, freshness, read-pool, thumbnail, summary, pruning, and migration
behavior for one remaining audit warning. That requires a separate reviewed
cache-engine replacement proposal if the lower-risk upstream route fails.

## Future Removal Criteria

The remaining `bincode` warning can be removed only after one of these paths is
reviewed:

- a published `localcache` release makes the bincode codec optional and arama
  can compile a selected cache codec without `bincode` in the dependency graph;
- a temporary localcache workspace patch is explicitly approved by the owner as
  release-intended, with matching dependency and cache compatibility evidence;
- a separate cache-engine replacement RFC proves schema, migration/rebuild,
  freshness, read-pool, cache page, prune, and dependency behavior.

If a future implementation selects JSON or another non-bincode codec, it must
include representative payload-size evidence for at least one image CLIP vector
payload and one video payload containing both CLIP and wav2vec2 vectors.
