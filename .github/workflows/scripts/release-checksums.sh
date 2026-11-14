#!/usr/bin/env bash
# Writes and verifies a release's SHA256SUMS file (T075). Two subcommands,
# run as two separate steps of the `release` job in
# release-executable.yaml, so each can be tested on its own without a tag:
#
#   release-checksums.sh write <dir>
#     Writes <dir>/SHA256SUMS over every other regular file directly inside
#     <dir>, sorted by name, in plain `sha256sum` output (`<hex>  <name>`,
#     no directory part in <name> - the same format every Linux and macOS
#     `sha256sum`/`shasum -a 256 -c` reads).
#
#   release-checksums.sh verify <dir> <assets.json>
#     <assets.json> is a release's asset list, as `gh release view <tag>
#     --json assets` prints it. For every file in <dir> (SHA256SUMS
#     included), the asset of that name must exist on the release and its
#     `digest` must equal `sha256:<the file's own local hash>`. Also fails
#     if the release carries an asset with no matching file in <dir>.
set -euo pipefail

usage() {
    echo "usage: release-checksums.sh write <dir>" >&2
    echo "       release-checksums.sh verify <dir> <assets.json>" >&2
    exit 1
}

cmd_write() {
    local dir="${1:?write needs a directory}"
    if [ ! -d "$dir" ]; then
        echo "release-checksums: $dir is not a directory" >&2
        exit 1
    fi

    local sums="$dir/SHA256SUMS"
    local files
    files="$(find "$dir" -maxdepth 1 -type f ! -name "$(basename "$sums")" -printf '%f\n' | sort)"
    if [ -z "$files" ]; then
        echo "release-checksums: $dir has no files to checksum" >&2
        exit 1
    fi

    (
        cd "$dir"
        : >SHA256SUMS
        while IFS= read -r name; do
            sha256sum -- "$name" >>SHA256SUMS
        done <<<"$files"
    )

    # The file it just wrote must itself verify - a self-check, not an
    # assumption that the loop above did the right thing.
    if ! (cd "$dir" && sha256sum -c SHA256SUMS >/dev/null); then
        echo "release-checksums: SHA256SUMS failed its own verification immediately after being written" >&2
        exit 1
    fi
    echo "release-checksums: wrote $sums ($(printf '%s\n' "$files" | wc -l) file(s))"
}

cmd_verify() {
    local dir="${1:?verify needs a directory}"
    local assets_json="${2:?verify needs an assets.json path}"
    if [ ! -d "$dir" ]; then
        echo "release-checksums: $dir is not a directory" >&2
        exit 1
    fi
    if [ ! -f "$assets_json" ]; then
        echo "release-checksums: $assets_json does not exist" >&2
        exit 1
    fi

    local failed=0

    # Every file in <dir> - SHA256SUMS included - must have a matching
    # asset on the release, with a real (non-null, non-absent) digest that
    # equals the file's own local hash. A missing or null digest is never
    # treated as a pass, even if everything else about the asset matches.
    local name local_hex expected asset_digest
    while IFS= read -r name; do
        local_hex="$(sha256sum -- "$dir/$name" | awk '{print $1}')"
        expected="sha256:$local_hex"
        asset_digest="$(jq -r --arg name "$name" '[.[] | select(.name == $name)][0].digest // "null"' "$assets_json")"
        if [ "$asset_digest" = "null" ]; then
            echo "release-checksums: $name has no matching asset on the release, or its digest is null/absent" >&2
            failed=1
        elif [ "$asset_digest" != "$expected" ]; then
            echo "release-checksums: $name digest mismatch - local $expected, release $asset_digest" >&2
            failed=1
        fi
    done < <(find "$dir" -maxdepth 1 -type f -printf '%f\n' | sort)

    # Every asset on the release must correspond to a file in <dir> - an
    # asset nothing locally accounts for is just as much a mismatch as a
    # wrong digest.
    local asset_name
    while IFS= read -r asset_name; do
        if [ ! -f "$dir/$asset_name" ]; then
            echo "release-checksums: asset $asset_name is on the release but not in $dir" >&2
            failed=1
        fi
    done < <(jq -r '.[].name' "$assets_json")

    if [ "$failed" -ne 0 ]; then
        exit 1
    fi
    echo "release-checksums: verified $(find "$dir" -maxdepth 1 -type f | wc -l) file(s) against $assets_json"
}

case "${1:-}" in
    write)
        shift
        cmd_write "$@"
        ;;
    verify)
        shift
        cmd_verify "$@"
        ;;
    *)
        usage
        ;;
esac
