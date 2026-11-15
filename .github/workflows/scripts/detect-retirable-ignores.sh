#!/usr/bin/env bash
# T066, implementing RFC 049 §4, §5 and §8.
#
# Two read-only checks against the real tree, neither of which can fail
# the build - see §3/§9 below for why, and review 174 (R1/R2) for the two
# ways an *earlier* version of this script could still turn `Audit` red
# or give actively wrong advice when one of its own steps failed.
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
set -uo pipefail
# Deliberately not `-e`: every step that can fail for reasons outside
# this script's control (a network hiccup during `cargo update`'s index
# fetch, the advisory database being briefly unreachable, a missing
# tool) is checked explicitly below and turned into a "could not run"
# notice via `could_not_run`, not an uncontrolled non-zero exit. Review
# 174 R1 found that the previous version relied on `set -e` alone for
# this and was wrong: everything before the single trailing `exit 0` was
# one bad network call away from turning `Audit` red, which §3/§9
# explicitly prohibit.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel)"
cd "$REPO_ROOT" || exit 0

AUDIT_TOML=".cargo/audit.toml"
LEDGER="rfcs/notes/audit-warning-burn-down.md"

# Review 174 R1: a notice that could not run says so, in the log and in
# $GITHUB_STEP_SUMMARY, and still exits 0 - it must never look like
# either "nothing retirable" or an actual job failure.
could_not_run() {
    echo "== T066: notice could not run =="
    echo "$1"
    if [ -n "${GITHUB_STEP_SUMMARY:-}" ]; then
        {
            echo "### T066 — retirable-ignore / ledger-sync notice"
            echo
            echo "- **Notice could not run:** $1"
        } >>"$GITHUB_STEP_SUMMARY"
    fi
    exit 0
}

for tool in git tar cargo jq awk sort grep mktemp; do
    command -v "$tool" >/dev/null 2>&1 || could_not_run "required tool '$tool' not found on PATH"
done

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

WORKDIR="$(mktemp -d)" || could_not_run "mktemp failed to create a scratch directory"
trap 'rm -rf "$WORKDIR"' EXIT

if ! git archive HEAD | tar -x -C "$WORKDIR"; then
    could_not_run "'git archive HEAD | tar -x' failed to populate the scratch copy"
fi

UPDATE_ERR="$(mktemp)"
if ! ( cd "$WORKDIR" && cargo update --quiet ) 2>"$UPDATE_ERR"; then
    err="$(tail -n 20 "$UPDATE_ERR")"
    rm -f "$UPDATE_ERR"
    could_not_run "'cargo update' failed on the scratch copy: $err"
fi
# cargo update's own warnings (not failures) still matter to a reader
# debugging a quiet run, so they go to the real log, not /dev/null.
cat "$UPDATE_ERR" >&2
rm -f "$UPDATE_ERR"

rm -f "$WORKDIR/$AUDIT_TOML"

AUDIT_JSON="$(mktemp)"
AUDIT_ERR="$(mktemp)"
( cd "$WORKDIR" && cargo audit --json -f Cargo.lock ) >"$AUDIT_JSON" 2>"$AUDIT_ERR"
audit_exit=$?
cat "$AUDIT_ERR" >&2

# Review 174 R2: cargo-audit's own exit code is 1 whenever it finds ANY
# vulnerability even without --deny - that is an expected, successful
# run, not a failure, so the exit code alone cannot distinguish "ran and
# found things" from "never ran at all" (a DB fetch failure, a missing
# binary). Check the report's own shape instead: a real report always
# carries a `.database` object and a positive dependency count. Without
# that, trusting an empty/malformed JSON previously meant every
# currently-ignored identity looked "no longer present" and got reported
# RETIRABLE - the worst possible wrong advice this notice could give.
if ! jq -e '(.database | type) == "object"
            and ((.lockfile["dependency-count"] // 0) > 0)' \
    "$AUDIT_JSON" >/dev/null 2>&1; then
    err="$(tail -n 20 "$AUDIT_ERR")"
    rm -f "$AUDIT_JSON" "$AUDIT_ERR"
    could_not_run "'cargo audit' did not produce a valid report (exit $audit_exit): $err"
fi
rm -f "$AUDIT_ERR"

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

# §3/§9: a notice, never a gate - every reachable exit path above is
# either `could_not_run` (which itself exits 0) or this one, so the
# script is guaranteed to exit 0 regardless of what it finds.
exit 0
