# A shrinking dependency graph can break the Windows build

**Found:** 2026-09-03, six commits after it started. **Fixed:** `b2363ec`
(T048), verified by a green `windows` job.

## The failure

`wgpu-hal` fails to compile on Windows with ten errors of one shape:

```
required for `&windows::Win32::Graphics::Direct3D12::ID3D12Heap`
to implement `Param<ID3D12Heap, windows_core::InterfaceType>`
```

Two `windows-core` versions resolving against each other. Nothing in arama's own
code is involved.

## Why it happens

| Package | requires |
|---|---|
| `gpu-allocator` | `windows = ">=0.53,<=0.58"` — **a range** |
| `wgpu-hal` | `windows 0.58` |

`gpu-allocator` hands `wgpu-hal` an `ID3D12Heap`, which only compiles if both
resolve to the same `windows`. The range lets cargo satisfy `gpu-allocator` with
*any* of several versions, and which one it picks depends on **what else is in
the graph**.

**So removing dependencies can break the build.** snora 0.42.0 stopped enabling
`iced`'s `svg` feature, dropping 26 packages (`resvg`, `usvg`, `lyon*`,
`rustybuzz`, `png`, `gif`, `zune-*`). With the requirements that had been holding
`windows` at 0.58 gone, cargo re-resolved `gpu-allocator` down to 0.56 while
`wgpu-hal` stayed at 0.58.

The lockfile diff was **one line**.

## The fix

```sh
cargo update -p gpu-allocator
```

Re-resolves that one package's edge without changing its version. `trash 5.2.6`
correctly stays on `windows 0.56.0` — it pins `^0.56` and exchanges no types with
`wgpu-hal`.

**Do not** use `cargo update -p windows@0.56.0 --precise 0.58.0`: it moves every
consumer of that node at once, and `trash` cannot follow. It fails outright.

**A `windows = "0.58"` pin in arama's manifest was considered and rejected.** It
would need an unused direct dependency to constrain anything, and it would become
the *cause* of the next conflict when `wgpu-hal` moves to a newer `windows`.

## The part worth remembering

**Every local gate stayed green for all six broken commits.** `cargo build`,
`cargo test`, `clippy`, `fmt` — all clean on Linux, all blind to this.

The only thing that saw it was `native-smoke.yaml`'s `windows` job, which went
red on the first commit that broke it and stayed red while five more landed on
top. **The gate worked; nobody read it.**

A useful local approximation exists if a Windows machine does not:
`cargo check --target x86_64-pc-windows-gnu` reproduced the exact error set. It
is `-gnu` not `-msvc` and performs no linking, so it is a signal rather than a
substitute — but it turns a week into a minute.

## Related

`rfcs/notes/snora-0.42-upgrade-assessment.md` ·
`rfcs/notes/audit-warning-burn-down.md` (the same upgrade resolved
RUSTSEC-2026-0206 by removal)
