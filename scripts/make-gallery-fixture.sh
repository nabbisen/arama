#!/usr/bin/env bash
# Builds the gallery fixture (T073): synthetic images only, never personal
# photographs, so the gallery and the Similar Pairs dialog have something to
# show results on. Deterministic - every primitive below is a fixed
# ImageMagick procedure, no randomness, and every PNG write excludes the
# date/time chunks ImageMagick otherwise embeds - so re-running this script
# on the same ImageMagick version reproduces byte-identical output (checked:
# two builds, two seconds apart, `cmp`-identical file by file).
#
# Usage:
#   scripts/make-gallery-fixture.sh [output-dir]
#
# Default output-dir: .git-exclude/fixtures/gallery (git-ignored, not in the
# repository - see .git-exclude/fixtures/README.md for how this fits the
# rest of the fixtures). This script starts with `rm -rf "$out_dir"`, so an
# output-dir is refused unless it is absent, empty, or already carries this
# script's own GENERATED_WITH.txt marker (review 166, R2: resolving under
# .git-exclude/ alone was not enough - `.git-exclude/fixtures` itself, one
# path segment short of the default, resolves there too, and would delete
# every fixture, not just the gallery this script made).
set -euo pipefail

cd "$(dirname "$0")/.."
project_root="$(pwd)"

out_dir="${1:-.git-exclude/fixtures/gallery}"

command -v magick >/dev/null 2>&1 || {
    echo "make-gallery-fixture: ImageMagick's 'magick' command is required but not found" >&2
    exit 1
}

resolved_out_dir="$(realpath -m -- "$out_dir")"
case "$resolved_out_dir" in
    "$project_root/.git-exclude"/*) ;;
    *)
        echo "make-gallery-fixture: output-dir ($out_dir) does not resolve under $project_root/.git-exclude - refusing to rm -rf it" >&2
        exit 1
        ;;
esac
out_dir="$resolved_out_dir"

# Being under .git-exclude/ is not enough on its own: .git-exclude/fixtures
# itself (one path segment short of the default) resolves there too, and
# holds every other fixture besides the gallery - deleting it would take
# the ~940 MB models fixture with it. Delete only what this script made:
# refuse unless the directory is absent, empty, or already carries this
# script's own GENERATED_WITH.txt from a prior run (review 166, R2).
if [ -d "$out_dir" ]; then
    first_entry="$(find "$out_dir" -mindepth 1 -print -quit)"
    if [ -n "$first_entry" ] && [ ! -f "$out_dir/GENERATED_WITH.txt" ]; then
        echo "make-gallery-fixture: output-dir ($out_dir) already exists, is not empty, and has no GENERATED_WITH.txt from a prior run of this script - refusing to rm -rf it" >&2
        exit 1
    fi
fi

rm -rf "$out_dir"
mkdir -p "$out_dir"

# PNG write, with the date/time chunks excluded - without this,
# ImageMagick embeds the filesystem write time of the moment the command
# ran, so a rebuilt fixture would never match its own SHA256SUMS.
magick_png() {
    magick -define png:exclude-chunks=date,time "$@"
}

# Three distinct motifs. Each gets a canonical PNG and two near-duplicate
# variants (lossy recompression, downscale-then-upscale blur) - visually
# almost identical to the canonical image, the way a phone's "edited" or
# "re-saved" copy of the same photo would be, which is what a similarity
# detector is meant to catch.
make_motif() {
    local name="$1"
    shift
    local canonical="$out_dir/$name-01-canonical.png"
    magick_png -size 512x512 "$@" "$canonical"
    # Near-duplicate 1: re-encoded as a lossy JPEG at reduced quality.
    # (JPEG output carries no such timestamp chunk - confirmed byte-identical
    # across two runs without any extra flag.)
    magick "$canonical" -quality 70 "$out_dir/$name-02-recompressed.jpg"
    # Near-duplicate 2: downscaled then upscaled back - same dimensions,
    # visibly softer, pixel-different from the canonical.
    magick_png "$canonical" -resize 400x400 -resize 512x512 "$out_dir/$name-03-resampled.png"
}

make_motif "blue-circle" \
    radial-gradient:blue-white \
    -fill '#1d3557' -draw 'circle 256,256 256,128'

make_motif "red-square" \
    gradient:red-yellow \
    -fill '#780000' -draw 'rectangle 128,128 384,384'

make_motif "green-triangle" \
    radial-gradient:green-black \
    -fill '#d8f3dc' -draw 'polygon 256,96 416,384 96,384'

# Two standalone, clearly distinct one-offs - broadens the "distinct" side
# beyond the three motifs above, which are each a near-duplicate group.
# Pure procedural patterns, no text: a font-dependent `-annotate` would make
# the result depend on which fonts happen to be installed, which is exactly
# the same non-reproducibility problem as the date chunks above, just from a
# different source.
magick_png -size 512x512 pattern:checkerboard "$out_dir/checkerboard.png"
# `plasma:` is ImageMagick's own pseudo-random generator - without a fixed
# `-seed` it is not reproducible at all (confirmed: two runs differed byte 259
# onward); `-seed 42` makes it exactly as deterministic as every other
# primitive here (confirmed: two runs `cmp`-identical).
magick_png -size 512x512 -seed 42 plasma:fractal -colorspace Gray "$out_dir/plasma-fractal.png"

generator_version="$(magick -version | head -1)"
printf 'generated by scripts/make-gallery-fixture.sh\n%s\n' "$generator_version" \
    >"$out_dir/GENERATED_WITH.txt"

echo "gallery fixture written to $out_dir"
find "$out_dir" -type f \( -name '*.png' -o -name '*.jpg' \) | sort
