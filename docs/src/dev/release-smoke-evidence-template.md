# Release Smoke Evidence Template

Copy this template for an owner-managed release-smoke run. It records what was
actually exercised in one environment; it does not replace automated gates and
does not authorize versioning, packaging, tagging, publishing, or pushing.

## Run context

| Field | Value |
|---|---|
| Release/version under consideration | `<version or unreleased revision>` |
| Date | `<YYYY-MM-DD>` |
| Platform | `<OS, architecture, relevant desktop environment>` |
| Build command or binary | `<command or artifact identity>` |
| Profile and local state | `<existing profile, temporary clean profile, cache/model state>` |
| Fixture directory | `<media types, approximate file count, non-sensitive description>` |
| Network/download availability | `<available, unavailable, or constrained>` |

Do not include secrets, private media names, or personal filesystem paths in
durable evidence.

## Result values

Use exactly one result for each check:

- `pass` — the expected behavior was observed;
- `fail` — the expected behavior was not observed;
- `not run` — the check was not attempted;
- `environment-dependent` — the current environment could not exercise the
  check, such as a clean-profile or online-download path.

## Results

The check definitions are maintained in the
[release-smoke checklist](./testing.md#release-smoke-with-the-ui). Keep IDs
stable when recording evidence and put diagnostic detail or issue/RFC links in
the final two columns.

| Smoke ID | Result | Notes | Follow-up |
|---|---|---|---|
| `SMOKE-SETUP-READY` | `<result>` | | |
| `SMOKE-SETUP-FIRST-RUN` | `<result>` | | |
| `SMOKE-SETUP-AI-SETTINGS` | `<result>` | | |
| `SMOKE-MACOS-FFMPEG-PATH` | `<result>` | | |
| `SMOKE-MACOS-FFMPEG-PREFIX` | `<result>` | | |
| `SMOKE-MACOS-FFMPEG-MISSING` | `<result>` | | |
| `SMOKE-MACOS-FFMPEG-LEGACY` | `<result>` | | |
| `SMOKE-MACOS-FFMPEG-MISMATCH` | `<result>` | | |
| `SMOKE-MACOS-FFMPEG-TIMEOUT` | `<result>` | | |
| `SMOKE-GALLERY-INDEX` | `<result>` | | |
| `SMOKE-GALLERY-SWITCH` | `<result>` | | |
| `SMOKE-GALLERY-FOCUS` | `<result>` | | |
| `SMOKE-SIMILARITY-PAIRS` | `<result>` | | |
| `SMOKE-SIMILARITY-SPARSE` | `<result>` | | |
| `SMOKE-CACHE-SUMMARY` | `<result>` | | |
| `SMOKE-CACHE-PRUNE` | `<result>` | | |
| `SMOKE-CACHE-RELOAD` | `<result>` | | |
| `SMOKE-CACHE-DELETE` | `<result>` | | |
| `SMOKE-SETTINGS-MEDIA` | `<result>` | | |
| `SMOKE-SETTINGS-THEME` | `<result>` | | |
| `SMOKE-SETTINGS-PERSIST` | `<result>` | | |
| `SMOKE-RESTART-VALID-ROOT` | `<result>` | | |
| `SMOKE-RESTART-INVALID-ROOT` | `<result>` | | |

## Run notes and follow-up

- Overall confidence: `<short assessment>`
- Failed or incomplete checks: `<IDs and concise reasons>`
- Follow-up issues, RFCs, or review packages: `<links or none>`
- Additional environment notes: `<notes or none>`
