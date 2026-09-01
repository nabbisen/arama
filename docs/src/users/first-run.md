# First Run

The first time arama starts it checks the required CLIP model and the optional
video capabilities. When required setup is missing, it shows a **setup
wizard** before opening the main interface.

## What gets downloaded

| Item | Source | Size (approx.) |
|---|---|---|
| CLIP model (`clip-vit-base-patch32`) | HuggingFace (`openai/`) | ~578 MB |
| wav2vec2 model (`wav2vec2-base-960h`) | HuggingFace (`facebook/`) | ~361 MB |

Model downloads may run in parallel. The total model data is roughly 938 MB
when both CLIP and wav2vec2 are installed. Subsequent runs reuse authenticated
complete model generations.

Downloaded model files are checked against pinned HuggingFace revisions and
SHA-256 digests before arama accepts them. If verification fails, the setup
item stops with an error and the partial file is discarded.

On every supported platform, arama never downloads ffmpeg. Setup shows it as
an external prerequisite with a re-check action. Install `ffmpeg` and
`ffprobe` through a source you trust, then make the pair available together on
`PATH` or select their directory in **Settings → AI**. On macOS, discovery
also checks the native Homebrew prefix. Re-check performs local discovery
only; it does not invoke a package manager or make an ffmpeg network request.

## If a download fails

- Check your internet connection and retry. Authenticated complete model
  generations are reused rather than fetched again.
- If the HuggingFace servers are temporarily unavailable, wait and
  retry. No rate-limiting or account is required.
- If setup reports a checksum mismatch, retry later. This usually means the
  download was corrupted before arama accepted it, or a pinned remote artifact
  was replaced upstream.
- On corporate networks, large model downloads may be blocked. Contact your
  network administrator rather than bypassing checksum validation.

## Skipping video support

On every supported platform, CLIP readiness is sufficient to continue with
image-only use. Wav2vec2 and ffmpeg remain optional video capabilities
available from **Settings → AI**. Their absence does not reopen setup after
restart. You can also disable video in **Settings → General**.

## After setup

Once the wizard finishes, arama opens the main **Explorer** page. The
left panel shows a directory tree rooted at the current working
directory. Select any folder to begin indexing.
