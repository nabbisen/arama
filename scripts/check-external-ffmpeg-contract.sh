#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

payload_pattern='(^|/)(ffmpeg|ffprobe)(\.exe)?$|(^|/)[^/]*ffmpeg[^/]*\.(7z|tar|tar\.gz|tar\.xz|tgz|txz|zip)$'
managed_source_pattern='yt-dlp/FFmpeg-Builds|ffmpeg-master-latest|DownloadArtifact|FfmpegDistribution|download_and_install|download_artifact|supported_artifacts|unpack_archive'

fail() {
    printf 'external FFmpeg contract check failed: %s\n' "$1" >&2
    exit 1
}

check_listing() {
    local label=$1
    local listing=$2
    if printf '%s\n' "$listing" | rg -n "$payload_pattern"; then
        fail "$label contains an FFmpeg executable or archive payload"
    fi
}

if rg -n "$managed_source_pattern" app crates env --glob '*.rs'; then
    fail 'production Rust source contains managed FFmpeg acquisition identifiers'
fi

if rg -n '^(reqwest|sha2|tar|xz2|zip)[[:space:]]*=' crates/engine/sidecar/Cargo.toml; then
    fail 'arama-sidecar declares an HTTP, digest, or archive acquisition dependency'
fi

packages=(
    arama-cache
    arama-i18n
    arama-env
    arama-sidecar
    arama-theme
    arama-ai
    arama-ui-widgets
    arama-ui-main
    arama-ui-layout
    arama
)

for package in "${packages[@]}"; do
    listing=$(cargo package -p "$package" --list --allow-dirty --locked)
    check_listing "cargo package -p $package --list" "$listing"
done

while (($#)); do
    case $1 in
        --archive)
            (($# >= 2)) || fail '--archive requires a path'
            archive=$2
            case $archive in
                *.tar.gz|*.tgz) listing=$(tar tzf "$archive") ;;
                *.tar.xz|*.txz) listing=$(tar tJf "$archive") ;;
                *.tar) listing=$(tar tf "$archive") ;;
                *.zip) listing=$(unzip -Z1 "$archive") ;;
                *) fail "unsupported archive format: $archive" ;;
            esac
            check_listing "$archive" "$listing"
            shift 2
            ;;
        --binary)
            (($# >= 2)) || fail '--binary requires a path'
            binary=$2
            if strings "$binary" | rg -n "$managed_source_pattern"; then
                fail "$binary contains managed FFmpeg acquisition identifiers"
            fi
            shift 2
            ;;
        *)
            fail "unknown argument: $1"
            ;;
    esac
done

printf 'external FFmpeg contract check passed\n'
