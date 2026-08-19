# Installation

arama is available through three distribution routes. Choose a platform
executable when a matching asset exists, install the published package graph
with Cargo, or build a source release archive.

## Prerequisites

| Requirement | Notes |
|---|---|
| **Internet connection** | Required once for AI models |
| **ffmpeg for video** | Install a user-managed `ffmpeg`/`ffprobe` pair before using video features |
| **~750 MB disk space** | Both AI models and initial cache headroom |
| **Writable platform data/config/cache directories** | arama stores settings, models, and cache data in your operating system's standard per-user locations, not next to its executable — see [Data locations](#data-locations) |

The Cargo-install and source-build routes also require a stable Rust toolchain
from [rustup.rs](https://rustup.rs/). The workspace uses Rust 2024 edition and
declares Rust 1.91 as its verified source-build baseline.

## Route 1: Platform executable asset

GitHub release assets contain an owner-built executable inside one wrapping
directory. Choose only an asset matching the platform and compute variant:

| Asset variant | Platform | Compute |
|---|---|---|
| `Linux-x64-gnu-cpu` | Linux x86_64 | CPU |
| `Linux-x64-gnu-gpu-cuda` | Linux x86_64 | NVIDIA CUDA |
| `macOS-aarch64` | Apple Silicon macOS | CPU/Metal |
| `Windows-x64-cpu` | Windows x86_64 | CPU |
| `Windows-x64-gpu-cuda` | Windows x86_64 | NVIDIA CUDA |

Asset names use:

```text
arama@<asset-variant>-<tag>.tar.gz  # Linux
arama@<asset-variant>-<tag>.zip     # macOS and Windows
```

Project tags use `X.Y.Z` (for example, `0.36.2`) without a `v` prefix.

Extract the archive and enter its same-named wrapping directory. For example:

```sh
tar xzf arama@Linux-x64-gnu-cpu-X.Y.Z.tar.gz
cd arama@Linux-x64-gnu-cpu-X.Y.Z
./arama
```

CUDA assets require a compatible NVIDIA driver and CUDA environment. Use the
CPU asset when CUDA is unavailable. If no asset matches the platform, use the
Cargo-install or source-build route instead.

## Route 2: Install from crates.io

The `arama` package and its internal packages are published on crates.io.
Cargo downloads and builds that graph locally; this is not a pre-built binary.

> **This route can lag.** crates.io publication happens at stable release
> points rather than at every release, so the version available there may be
> older than the source archive and executable assets.
>
> **Check the [crates.io page](https://crates.io/crates/arama) for what is
> actually published.** Anything earlier than **0.37.0** predates the
> external-ffmpeg change described below and behaves differently — it manages
> its own ffmpeg rather than using a pair you installed. Use Route 1 or
> Route 3 if you need current behaviour.

```sh
cargo install arama --locked
arama
```

The executable is normally installed in Cargo's binary directory (commonly
`~/.cargo/bin`). Runtime data does not need to live there — see
[Data locations](#data-locations).

## Route 3: Build the source archive

The source archive intentionally has no wrapping directory: project files are
stored at archive root. Its version also has no `v` prefix. Create the
destination before extraction:

```sh
mkdir arama-X.Y.Z
tar xzf arama-X.Y.Z.tar.gz -C arama-X.Y.Z
cd arama-X.Y.Z
```

Build and run:

```sh
cargo build -p arama --release
cargo run -p arama --release
```

The compiled binary is `target/release/arama` (or
`target/release/arama.exe` on Windows). It can also be run directly.

The source route supports the project's broader source-build platform set:

| Platform | Architecture | Status |
|---|---|---|
| Linux | x86_64, aarch64 | Supported |
| macOS | x86_64, aarch64 (Apple Silicon) | Supported |
| Windows | x86_64 | Supported |

## Video prerequisite

Arama does not download or install executable ffmpeg files on any platform.
Install a paired `ffmpeg` and `ffprobe` toolchain through a source you trust.
For example, Homebrew provides the pair on macOS:

```sh
brew install ffmpeg
```

Arama accepts a compatible pair found together on the inherited `PATH` or in a
directory selected in **Settings → AI**. It also checks Homebrew's native
default prefix (`/opt/homebrew/bin` on Apple Silicon, `/usr/local/bin` on
Intel) because apps launched from Finder may not inherit the interactive
shell's complete `PATH`.

Arama validates that both commands start successfully and report the same
release/build token. It never runs Homebrew, changes quarantine attributes,
applies ad-hoc signatures, or asks for elevated privileges. Image-only use is
available without ffmpeg.

## Data locations

Runtime data lives in your operating system's standard per-user locations —
never next to the executable, and independent of which installation route you
used or where you extracted it:

| Platform | Settings | Models | Cache |
|---|---|---|---|
| Windows | `%APPDATA%\arama` | `%LOCALAPPDATA%\arama\data` | `%LOCALAPPDATA%\arama\cache` |
| macOS | `~/Library/Application Support/arama` | `~/Library/Application Support/arama` | `~/Library/Caches/arama` |
| Linux | `$XDG_CONFIG_HOME/arama` (or `~/.config/arama`) | `$XDG_DATA_HOME/arama` (or `~/.local/share/arama`) | `$XDG_CACHE_HOME/arama` (or `~/.cache/arama`) |

Models and cache deliberately use the **local**, non-roaming location on
Windows — a several-hundred-megabyte model should never synchronise into a
roaming profile. Settings use the roaming location, since a small JSON file
following you between machines on the same domain is the desired behaviour.

Within the models directory: model weight files directly, and a `bin/`
subfolder that ffmpeg discovery excludes from automatic candidates. Within
the cache directory: `cache-v2.sqlite` (the embedding/thumbnail database)
and `thumbnail/` (generated 224×224 JPEG thumbnails).

Older arama versions (before 0.40.0) may have left ffmpeg files in a
`.arama-local/bin/` directory next to the executable — a different, older
location than the `bin/` subfolder above. Current versions exclude that
location from automatic discovery and from explicit selection, and never
delete those files automatically; after installing a user-managed pair, you
may remove the legacy files manually if you no longer need them.

## Upgrading from a version before 0.40.0

Versions before 0.40.0 stored settings, models and cache next to the
executable instead of in the locations above. **On first launch after
upgrading, arama moves your existing settings, models and cache to the new
locations automatically — nothing is discarded and no re-indexing is
required.** If you had been launching arama from more than one directory
before upgrading, note that settings previously followed whichever directory
you launched from; after upgrading there is a single configuration, and the
one adopted is whichever the migration finds first.

If a relocation cannot be completed (for example, because the new location
cannot be created), arama tells you rather than starting with your data
silently left behind.

## Updating

- **Executable asset:** extract the new asset into a fresh writable directory.
- **Cargo install:** run `cargo install arama --locked --force` after the new
  registry release is available.
- **Source archive:** extract the new archive into a fresh destination and
  rebuild it.

Because settings, models and cache all live outside the executable's own
directory, they are automatically shared across every installation on the
same machine — extracting a new asset into a fresh directory does not start
with an empty cache. The v2 cache database is reused as-is. Old v1 cache
databases are no longer imported; thumbnails and embeddings are rebuildable,
so the application creates current cache entries lazily as directories are
indexed.
