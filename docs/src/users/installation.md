# Installation

arama is available through three distribution routes. Choose a platform
executable when a matching asset exists, install the published package graph
with Cargo, or build a source release archive.

## Prerequisites

| Requirement | Notes |
|---|---|
| **Internet connection** | Required once for AI models; Linux/Windows also fetch a verified ffmpeg build |
| **ffmpeg on macOS** | Install a user-managed `ffmpeg`/`ffprobe` pair before using video features |
| **~800 MB disk space** | Both AI models, optional managed ffmpeg on Linux/Windows, and initial cache headroom |
| **Writable executable directory** | arama stores settings, models, managed Linux/Windows ffmpeg, and cache data alongside its executable |

The Cargo-install and source-build routes also require a stable Rust toolchain
from [rustup.rs](https://rustup.rs/). The workspace uses Rust 2024 edition and
MSRV 1.90.

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

```sh
cargo install arama --locked
arama
```

The executable is normally installed in Cargo's binary directory (commonly
`~/.cargo/bin`). That directory must be writable because arama creates its
runtime data beside the executable.

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

## macOS video prerequisite

Arama does not download or install executable ffmpeg files on macOS. Install a
paired `ffmpeg` and `ffprobe` toolchain yourself. Homebrew is the recommended
route on both Apple Silicon and Intel macOS:

```sh
brew install ffmpeg
```

Arama accepts a compatible pair found together on the inherited `PATH`. It
also checks Homebrew's native default prefix (`/opt/homebrew/bin` on Apple
Silicon, `/usr/local/bin` on Intel) because apps launched from Finder may not
inherit the interactive shell's complete `PATH`.

Arama validates that both commands start successfully and report the same
release/build token. It never runs Homebrew, changes quarantine attributes,
applies ad-hoc signatures, or asks for elevated privileges. Image-only use is
available without ffmpeg.

## Data locations

All runtime data lives next to the executable selected by the installation
route:

| Path | Contents |
|---|---|
| `.arama-local/` | AI models and, on Linux/Windows, managed ffmpeg files |
| `.arama-local/bin/` | Verified managed ffmpeg pair on Linux/Windows; not trusted for discovery on macOS |
| `.arama-cache/` | SQLite embedding cache and thumbnails |
| `.arama-cache/cache-v2.sqlite` | Embedding and thumbnail metadata |
| `.arama-cache/thumbnail/` | Generated 224×224 JPEG thumbnails |

The application settings file (`settings.json`, managed by
`app-json-settings`) is also written relative to the executable directory.

Older arama versions may have left macOS ffmpeg files in
`.arama-local/bin/`. Current versions deliberately ignore those legacy files
because their downloaded identity was not pinned. Arama does not delete them;
after installing a user-managed pair, you may remove the legacy files
manually if you no longer need them.

## Updating

- **Executable asset:** extract the new asset into a fresh writable directory.
- **Cargo install:** run `cargo install arama --locked --force` after the new
  registry release is available.
- **Source archive:** extract the new archive into a fresh destination and
  rebuild it.

The `.arama-cache/` directory from a current v2-cache version can be reused.
Old v1 cache databases are no longer imported; thumbnails and embeddings are
rebuildable, so the application creates current cache entries lazily as
directories are indexed.
