# Migration report: lucide-icons 0.576.0 → 1.17.0

**Verdict: no migration effort required. Update is a drop-in.**

## What changed

### Versioning scheme

The crate switched from tracking the upstream Lucide icon-set release
number (0.576) to conventional semver (1.17.0 corresponds to Lucide
icon set 1.17). The major-version jump is cosmetic; it reflects the
versioning reset, not a breaking API change.

### Icon additions and removals

| | Count |
|---|---|
| Icons in 0.576.0 | 1,658 |
| Icons in 1.17.0 | 1,716 |
| Unchanged | 1,638 |
| **Removed** | **20** |
| Added | 78 |

The 20 removed icons are all third-party brand/social-media icons that
Lucide dropped from their official set:

`icon_chrome`, `icon_codepen`, `icon_codesandbox`, `icon_dribbble`,
`icon_facebook`, `icon_figma`, `icon_framer`, `icon_github`,
`icon_gitlab`, `icon_instagram`, `icon_linkedin`, `icon_pocket`,
`icon_rail_symbol`, `icon_slack`, `icon_square_chart_gantt`,
`icon_text_select`, `icon_trello`, `icon_twitch`, `icon_twitter`,
`icon_youtube`.

**None of these are used in arama.** Cross-referencing every `icon_*`
call across the UI crates confirmed zero overlap.

### Non-icon API

No changes to the module-level public API (`lib.rs`, `icon.rs`). The
`iced` and `serde` features are identical. Every icon function
signature is unchanged: `pub fn icon_*<'a>() -> iced::widget::Text<'a>`.

## How to apply

In `Cargo.toml` (workspace):

```toml
# was:
lucide-icons = { version = "0", features = ["iced"] }

# becomes:
lucide-icons = { version = "1", features = ["iced"] }
```

Then `cargo update -p lucide-icons`.

No source changes needed.
