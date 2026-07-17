# Security Boundaries

Arama processes user-selected media locally. Its most important external trust
boundaries are executable acquisition, AI model acquisition, and the local
process environment used to select tools.

## Protected assets

- private image/video contents and filenames;
- filesystem access available to the arama process;
- AI models, cache entries, thumbnails, and settings;
- integrity of video probing, extraction, and similarity results.

Arama has no telemetry or analytics. Network access is used for explicit model
setup and, on Linux/Windows, verified managed ffmpeg setup.

## ffmpeg executable trust boundary

Downloaded executable bytes are more sensitive than ordinary data: ffmpeg and
ffprobe receive paths to private media and run with the application's local
permissions.

### Linux and Windows

Arama-managed artifacts come from pinned `yt-dlp/FFmpeg-Builds` release asset
identities. Every supported artifact has a committed 64-character lowercase
SHA-256 digest. Downloading, bounded buffering, digest verification,
extraction, and atomic pair publication are one authority; callers cannot ask
the public API to install an arbitrary pre-positioned archive.

The previous installed pair is retained until the complete downloaded archive
authenticates and both tools are published together. Digest, extraction, or
activation failure does not expose a partial replacement.

### macOS

Arama does not download or install ffmpeg executables. The user or package
manager owns acquisition. Homebrew is the documented route:

```sh
brew install ffmpeg
```

Discovery selects `ffmpeg` and `ffprobe` together from one logical candidate:
the inherited `PATH`, `/opt/homebrew/bin` on Apple Silicon, or
`/usr/local/bin` on Intel. Both commands are probed off the UI thread with
bounded output and a finite deadline. Their exact release/build tokens must
match. Arama does not invoke Homebrew, `curl`, a shell, `xattr`, `codesign`, or
privileged operations.

A user-controlled `PATH` remains part of the user's local execution trust
environment; version-pair validation is compatibility checking, not
cryptographic authentication of locally installed programs.

### Legacy macOS sidecars

Older versions may have downloaded unpinned macOS files into
`.arama-local/bin/`. Current macOS discovery ignores these files and never
deletes them automatically. This prevents silent continued execution while
leaving migration/removal under the user's control.

## AI model trust boundary

CLIP and wav2vec2 specifications contain validated identifiers, pinned source
revisions/digests, and positive byte limits. One registry-owned worker per
model downloads into an operation-owned staging directory, authenticates the
complete model/config set, writes a manifest tied to the immutable
specification, and atomically publishes the directory.

Callers join a generation-specific retained result. An OS-backed per-model
lock serializes cooperating processes across recovery, transfer, conversion,
and publication. Lock contention has a 30-second bound. Recovery order uses a
monotonic per-model sequence persisted while holding that lock; it does not
claim power-loss durability because the file and containing directory are not
explicitly synchronized. Corrupt or exhausted sequence state fails closed.

Once an authenticated final is Ready, inability to remove stale stage/backup
directories is diagnostic-only. The final remains usable and later lifecycle
calls retry cleanup without downloading again. The diagnostic currently goes
to stderr and includes the stale path. Recovery-critical errors before Ready
remain fatal.

The lock/sequence protocol coordinates versions that implement it. An older
process that ignores the lock is outside that guarantee. Readers do not take
the model lock; current replacement occurs only when no matching Ready final
exists. Future live replacement of a Ready revision should use a versioned
directory plus atomic active-generation pointer or explicit reader
coordination.

## Operational reporting

User-actionable setup/discovery failures are shown in setup or Settings.
Per-file media failures are summarized without aborting unrelated files.
Static cleanup/invariant warnings may remain developer diagnostics when the
visible fallback is truthful and safe. Repeated stale-model cleanup warnings
are a candidate for a future structured diagnostic sink if field experience
shows that stderr is insufficient.

## Release evidence

Automated Linux-host tests cover digest enforcement, bounded I/O, process
timeouts, paired publication, model joining/recovery, and the macOS policy
decision seams. They do not substitute for native macOS execution. Use the
[release smoke checklist](./testing.md#release-smoke-with-the-ui) and record
each available architecture as pass, fail, not run, or environment-dependent.
