# Installation

arama is distributed as a Cargo workspace source archive. You build it
yourself with the Rust toolchain — there are no pre-built binaries at
this stage.

## Prerequisites

| Requirement | Notes |
|---|---|
| **Rust toolchain** | Install via [rustup.rs](https://rustup.rs/); the stable channel is sufficient |
| **Internet connection** | Required once, for the AI model and ffmpeg download at first launch |
| **~500 MB disk space** | Models (~400 MB), ffmpeg binary, thumbnail cache |

Rust 2024 edition is required; `rustup` will select the correct toolchain
automatically from `rust-toolchain.toml` if present, or from the workspace
edition in `Cargo.toml`.

## Steps

### 1. Extract the archive

```sh
tar xzf arama-vX.Y.Z.tar.gz
cd arama-vX.Y.Z
```

### 2. Build

```sh
cargo build -p arama --release
```

The compiled binary ends up at `target/release/arama` (or
`target/release/arama.exe` on Windows).

### 3. Run

```sh
cargo run -p arama --release
# or run the binary directly:
./target/release/arama
```

`cargo run` is the simplest option during initial setup because the
working directory matters: arama stores its data relative to the
**executable's location** (see below).

## Data locations

All runtime data lives next to the executable:

| Path | Contents |
|---|---|
| `.arama-local/` | AI models, ffmpeg binary |
| `.arama-local/bin/` | ffmpeg executable |
| `.arama-cache/` | SQLite embedding cache, thumbnails |
| `.arama-cache/cache-v2.sqlite` | Embedding and thumbnail metadata |
| `.arama-cache/thumbnail/` | Generated 224×224 JPEG thumbnails |

The application settings file (`settings.json` managed by
`app-json-settings`) is also written relative to the executable
directory.

## Supported platforms

| Platform | Architecture | Status |
|---|---|---|
| Linux | x86\_64, aarch64 | Supported |
| macOS | x86\_64, aarch64 (Apple Silicon) | Supported |
| Windows | x86\_64 | Supported |

## Updating

Extract the new archive to a fresh directory and run `cargo build`. The
`.arama-cache/` directory from a previous version can be reused — the
application runs a one-time migration from the v1 cache format on first
launch and records the result in `CHANGELOG.md`.
