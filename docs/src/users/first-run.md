# First Run

The first time arama starts it detects that the AI models and ffmpeg
are missing and shows a **setup wizard** before opening the main
interface.

## What gets downloaded

| Item | Source | Size (approx.) |
|---|---|---|
| CLIP model (`clip-vit-base-patch32`) | HuggingFace (`openai/`) | ~350 MB |
| wav2vec2 model (`wav2vec2-base-960h`) | HuggingFace (`facebook/`) | ~360 MB |
| ffmpeg binary | GitHub CDN (`yt-dlp/FFmpeg-Builds`) | ~80 MB (Linux/Windows) |

The downloads happen in parallel. A progress bar is shown for each item.
The total download is roughly 800 MB on a fresh installation; subsequent
runs skip setup entirely.

Downloaded model files are checked against pinned HuggingFace revisions
and SHA-256 digests before arama accepts them. Linux and Windows ffmpeg
downloads use pinned GitHub release asset IDs and are checked against the
SHA-256 digest published with the `yt-dlp/FFmpeg-Builds` release metadata.
If verification fails, the setup item stops with an error and the partial
file is discarded.

## If a download fails

- Check your internet connection and restart arama. The wizard resumes
  from where it left off — files that already downloaded are not
  re-fetched.
- If the HuggingFace servers are temporarily unavailable, wait and
  retry. No rate-limiting or account is required.
- If setup reports a checksum mismatch, retry later. This usually means the
  download was corrupted before arama accepted it, or a pinned remote artifact
  was replaced upstream.
- On corporate networks, large binary downloads may be blocked. Contact
  your network administrator or download the files manually and place
  them in `.arama-local/`.

## Skipping video support

If you do not need video analysis, you can disable the video media type
in **Settings → General** after setup completes. The wav2vec2 model is
still downloaded during setup but will not be used for inference.

## After setup

Once the wizard finishes, arama opens the main **Explorer** page. The
left panel shows a directory tree rooted at the current working
directory. Select any folder to begin indexing.
