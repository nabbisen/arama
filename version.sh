#!/bin/sh
#
# version.sh — show or set arama's single workspace package version.
#
# Every member inherits [workspace.package].version. Internal dependency
# requirements are intentionally outside this helper, so adding or removing a
# workspace crate never requires changing the script.

CARGO_TOML=./Cargo.toml

show_help() {
    cat <<EOF
Usage: ${0##*/} [OPTIONS]

Options:
  -l, --list                Show the current workspace version.
  -u, --update VERSION      Set the workspace version to VERSION.
  -d, --dry-run             Show what would change, but do not modify files.
  -h, --help                Show this help and exit.

Updates [workspace.package].version in ${CARGO_TOML}. Member packages inherit
that value. The command does not modify workspace dependencies, Cargo.lock,
member manifests, the changelog, or the Git index.

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

if [ -z "$CUR" ]; then
    printf 'Error: could not find version in [workspace.package].\n' >&2
    exit 1
fi

if [ "$LIST_MODE" -eq 1 ]; then
    printf 'Workspace version: %s\n' "$CUR"
    [ "$UPDATE_MODE" -eq 0 ] && exit 0
fi

if [ "$UPDATE_MODE" -eq 1 ]; then
    if [ -z "$NEW_VERSION" ]; then
        printf 'Error: No new version supplied.\n' >&2
        exit 1
    fi

    if [ "$DRY_RUN" -eq 1 ]; then
        printf 'Workspace version: %s -> %s\n' "$CUR" "$NEW_VERSION"
        printf 'Would modify %s only.\n' "$CARGO_TOML"
        exit 0
    fi

    # Keep the replacement beside Cargo.toml for a same-filesystem rename.
    # Copy its metadata first so the new inode retains the manifest mode.
    tmp=$(mktemp "${CARGO_TOML}.tmp.XXXXXX") || exit 1
    trap 'rm -f "$tmp"' 0 1 2 15
    cp -p "$CARGO_TOML" "$tmp" || exit 1

    awk -v nv="$NEW_VERSION" '
        /^\[/ { in_wp = ($0 ~ /^\[workspace\.package\]/) }
        in_wp && /^[[:space:]]*version[[:space:]]*=/ && !done {
            print "version = \"" nv "\""
            done = 1
            next
        }
        { print }
        END { if (!done) exit 1 }
    ' "$CARGO_TOML" > "$tmp" || exit 1

    mv "$tmp" "$CARGO_TOML" || exit 1
    trap - 0 1 2 15

    printf '%s -> %s (updated %s)\n' "$CUR" "$NEW_VERSION" "$CARGO_TOML"
fi

exit 0
