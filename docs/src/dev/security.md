# Security Boundaries

Arama processes user-selected media locally. Its most important external trust
boundaries are executable acquisition, AI model acquisition, and the local
process environment used to select tools.

## Protected assets

- private image/video contents and filenames;
- filesystem access available to the arama process;
- AI models, cache entries, thumbnails, and settings;
- integrity of video probing, extraction, and similarity results.

Arama has no telemetry or analytics. Network access is used for explicit AI
model setup, not for ffmpeg acquisition.

## ffmpeg executable trust boundary

Downloaded executable bytes are more sensitive than ordinary data: ffmpeg and
ffprobe receive paths to private media and run with the application's local
permissions.

Arama does not download or install ffmpeg executables on any supported
platform. The user or package manager owns acquisition. For example, on
macOS, Homebrew can provide the pair:

```sh
brew install ffmpeg
```

Discovery selects `ffmpeg` and `ffprobe` together from one logical candidate:
the inherited `PATH`, a user-selected directory, or the native Homebrew prefix
on macOS. Both commands are probed off the UI thread with bounded output and a
finite deadline. Their exact release/build tokens must match. Arama does not
invoke a package manager, `curl`, a shell, `xattr`, `codesign`, or privileged
operations.

A user-controlled `PATH` remains part of the user's local execution trust
environment; version-pair validation is compatibility checking, not
cryptographic authentication of locally installed programs.

### Legacy managed sidecars

Older versions may have downloaded files into `.arama-local/bin/`. Current
discovery ignores this location on every platform and never deletes it
automatically. This prevents silent continued execution while leaving
migration/removal under the user's control.

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

Automated Linux-host tests cover bounded probing, process timeouts, pair
validation, model authentication/joining/recovery, and cross-platform policy
decision seams. They do not substitute for native Windows or macOS execution. Use the
[release smoke checklist](./testing.md#release-smoke-with-the-ui) and record
each available architecture as pass, fail, not run, or environment-dependent.
