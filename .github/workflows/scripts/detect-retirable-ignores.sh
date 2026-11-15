#!/usr/bin/env bash
# T066, implementing RFC 049 §4, §5 and §8.
#
# Two read-only checks against the real tree, neither of which can fail
# the build - see §3/§9 below for why.
#
# 1. Retirable-ignore detection (RFC 049 §4). Copies every file `cargo
#    update` needs (via `git archive HEAD`, so .git-exclude/ and target/
#    are never touched or even visible) into a scratch directory, runs an
#    unconstrained `cargo update` there, deletes that copy's own
#    .cargo/audit.toml (so the following audit is unfiltered - the real
#    repo's ignore config must not hide what we're trying to see), and
#    asks cargo-audit which advisories the updated lockfile still
#    carries. Any identity that is in the real ignore list but no longer
#    in that unfiltered post-update set has had its retiring condition
#    met and should be removed.
#
#    This can only see a semver-compatible update (RFC 049 §4.1) - a
#    retirement that needs a semver-major bump, like T067's localcache
#    0.21 -> 0.26, is invisible to an unconstrained `cargo update` by
#    construction, and a quiet result here must not be read as "nothing
#    retirable" - it says so explicitly below.
#
# 2. Ignore/ledger sync check (RFC 049 §8). Every identity in the real
#    .cargo/audit.toml's ignore array must be named, by identity, in
#    rfcs/notes/audit-warning-burn-down.md - including a grouped mention,
#    which ledger-identities.awk (beside this script) expands.
#
# The real Cargo.lock is never touched: every `cargo update` above runs
# inside a `mktemp -d` scratch copy, removed on exit.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel)"
cd "$REPO_ROOT"

AUDIT_TOML=".cargo/audit.toml"
LEDGER="rfcs/notes/audit-warning-burn-down.md"

# Every identity currently in .cargo/audit.toml's ignore array - the
# array only, not the explanatory comments above it, which also name
# plenty of now-resolved identities that must not be treated as current.
ignored_identities() {
    awk '/^ignore = \[/ { flag = 1 }
         flag { print; if (/^\]/) exit }' "$AUDIT_TOML" \
        | grep -oE 'RUSTSEC-[0-9]{4}-[0-9]+' | sort -u
}

mapfile -t IGNORED < <(ignored_identities)

echo "== T066: retirable-ignore detection (RFC 049 §4) =="
echo "Currently ignored (${#IGNORED[@]}): ${IGNORED[*]}"

WORKDIR="$(mktemp -d)"
trap 'rm -rf "$WORKDIR"' EXIT

git archive HEAD | tar -x -C "$WORKDIR"
( cd "$WORKDIR" && cargo update --quiet )
rm -f "$WORKDIR/$AUDIT_TOML"

AUDIT_JSON="$(mktemp)"
( cd "$WORKDIR" && cargo audit --json -f Cargo.lock ) >"$AUDIT_JSON" 2>/dev/null || true

present_after_update="$(jq -r '
    ([.vulnerabilities.list[]?.advisory.id]
     + [.warnings[]?[]?.advisory.id]) | unique | .[]
' "$AUDIT_JSON")"
rm -f "$AUDIT_JSON"

retirable=()
for id in "${IGNORED[@]}"; do
    if ! grep -qxF "$id" <<<"$present_after_update"; then
        retirable+=("$id")
    fi
done

if [ "${#retirable[@]}" -eq 0 ]; then
    echo "No retirable ignores found. (This only detects a semver-compatible" \
         "update clearing an ignore - RFC 049 §4.1. A retirement that needs" \
         "a semver-major bump, like T067's localcache 0.21 -> 0.26, is" \
         "invisible to this check by design; a quiet result here is not" \
         "evidence that nothing is retirable that way.)"
else
    echo "RETIRABLE - no longer present after an unconstrained 'cargo update':"
    printf '  - %s\n' "${retirable[@]}"
fi

echo
echo "== T066: ignore/ledger sync check (RFC 049 §8) =="
mapfile -t LEDGER_IDS < <(awk -f "$SCRIPT_DIR/ledger-identities.awk" "$LEDGER" | sort -u)

missing=()
for id in "${IGNORED[@]}"; do
    found=0
    for lid in "${LEDGER_IDS[@]}"; do
        if [ "$id" = "$lid" ]; then
            found=1
            break
        fi
    done
    if [ "$found" -eq 0 ]; then
        missing+=("$id")
    fi
done

if [ "${#missing[@]}" -eq 0 ]; then
    echo "All ${#IGNORED[@]} ignored identities are named in $LEDGER."
else
    echo "OUT OF SYNC - ignored in $AUDIT_TOML but not named in $LEDGER:"
    printf '  - %s\n' "${missing[@]}"
fi

if [ -n "${GITHUB_STEP_SUMMARY:-}" ]; then
    {
        echo "### T066 — retirable-ignore / ledger-sync notice"
        echo
        if [ "${#retirable[@]}" -eq 0 ]; then
            echo "- No retirable ignores found (semver-compatible updates only)."
        else
            echo "- **Retirable:** ${retirable[*]}"
        fi
        if [ "${#missing[@]}" -eq 0 ]; then
            echo "- Ledger sync: all ${#IGNORED[@]} ignored identities are named."
        else
            echo "- **Ledger sync out of date, missing:** ${missing[*]}"
        fi
    } >>"$GITHUB_STEP_SUMMARY"
fi

# §3/§9: a notice, never a gate - this must always exit 0, regardless of
# what it finds, so it can never turn Audit red.
exit 0
