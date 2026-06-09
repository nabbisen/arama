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

## If a download fails

- Check your internet connection and restart arama. The wizard resumes
  from where it left off — files that already downloaded are not
  re-fetched.
- If the HuggingFace servers are temporarily unavailable, wait and
  retry. No rate-limiting or account is required.
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
