#!/bin/sh
#
# version.sh — show or set the single workspace version for arama.
#
# The version lives in two places in the root Cargo.toml — both are
# updated atomically by --update:
#
#   1. [workspace.package]  version = "X.Y.Z"
#      (inherited by every member via version.workspace = true)
#
#   2. [workspace.dependencies]  arama-* = { version = "X.Y.Z", path = "…" }
#      (required alongside `path` so crates can be published; deps.rs
#      and docs.rs also need it to resolve the dependency graph)
#
# No external tools are required (no jq, no cargo metadata).
#
# Examples:
#   ./version.sh --list
#   ./version.sh --update 1.2.3
#   ./version.sh --update 1.2.3 --dry-run

CARGO_TOML=./Cargo.toml

show_help() {
    cat <<EOF
Usage: ${0##*/} [OPTIONS]

Options:
  -l, --list                Show the current workspace version.
  -u, --update VERSION      Set the workspace version to VERSION.
  -d, --dry-run             Show what would change, but do not modify files.
  -h, --help                Show this help and exit.

Updates two locations in ${CARGO_TOML}:
  - [workspace.package] version field (inherited by all members)
  - [workspace.dependencies] version fields for internal arama-* crates

Examples:
  ${0##*/} --list
  ${0##*/} --update 1.2.3
  ${0##*/} --update 1.2.3 --dry-run
EOF
    exit 0
}

LIST_MODE=0
UPDATE_MODE=0
DRY_RUN=0
NEW_VERSION=
NO_OPTION=1

while [ $# -gt 0 ]; do
    case "$1" in
        -l|--list)    LIST_MODE=1;   NO_OPTION=0; shift ;;
        -u|--update)  UPDATE_MODE=1; NO_OPTION=0; shift
                      if [ $# -eq 0 ]; then
                          printf 'Error: --update requires a version argument.\n' >&2
                          exit 1
                      fi
                      NEW_VERSION=$1; shift ;;
        -d|--dry-run) DRY_RUN=1;     NO_OPTION=0; shift ;;
        -h|--help)    show_help ;;
        *) printf 'Unknown option: %s\n' "$1" >&2; exit 1 ;;
    esac
done

[ "$NO_OPTION" -eq 1 ] && show_help

if [ ! -f "$CARGO_TOML" ]; then
    printf 'Error: %s not found (run from the workspace root).\n' "$CARGO_TOML" >&2
    exit 1
fi

# Read the version value from inside the [workspace.package] table.
current_version() {
    awk '
        /^\[/ { in_wp = ($0 ~ /^\[workspace\.package\]/) }
        in_wp && /^[[:space:]]*version[[:space:]]*=/ {
            gsub(/.*=[[:space:]]*"/, ""); gsub(/".*/, "")
            print; exit
        }
    ' "$CARGO_TOML"
}

CUR=$(current_version)

if [ "$LIST_MODE" -eq 1 ]; then
    printf 'Workspace version: %s\n' "${CUR:-<not found>}"
    [ "$UPDATE_MODE" -eq 0 ] && exit 0
fi

if [ "$UPDATE_MODE" -eq 1 ]; then
    if [ -z "$NEW_VERSION" ]; then
        printf 'Error: No new version supplied.\n' >&2
        exit 1
    fi
    if [ -z "$CUR" ]; then
        printf 'Error: could not find version in [workspace.package].\n' >&2
        exit 1
    fi

    if [ "$DRY_RUN" -eq 1 ]; then
        printf '%s -> %s (would modify %s)\n' "$CUR" "$NEW_VERSION" "$CARGO_TOML"
        exit 0
    fi

    tmp=$(mktemp) || exit 1
    awk -v cur="$CUR" -v nv="$NEW_VERSION" '
        # Track which top-level table we are in.
        /^\[/ {
            in_wp  = ($0 ~ /^\[workspace\.package\]/)
            in_wdep = ($0 ~ /^\[workspace\.dependencies\]/)
        }

        # [workspace.package]: rewrite the bare version line.
        in_wp && /^[[:space:]]*version[[:space:]]*=/ && !wp_done {
            print "version = \"" nv "\""
            wp_done = 1
            next
        }

        # [workspace.dependencies]: rewrite version = "CUR" inside
        # any internal arama-* entry on the same line.
        in_wdep && /^arama-/ {
            sub("version = \"" cur "\"", "version = \"" nv "\"")
        }

        { print }
    ' "$CARGO_TOML" > "$tmp" && mv "$tmp" "$CARGO_TOML"

    printf '%s -> %s (updated %s)\n' "$CUR" "$NEW_VERSION" "$CARGO_TOML"
fi

exit 0
