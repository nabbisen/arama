#!/usr/bin/env bash
# Task 034 §3.2: flags widget calls that render a bare string literal
# instead of routing through arama_i18n::t(...)/t_with(...).
#
# This is a heuristic pattern match, not a parser - two sweeps (package
# 124 and Task 034 itself) found untranslated strings that survived
# review because nobody was looking for them; a narrow, repeatable check
# is worth having even though it cannot be exhaustive. Read both lists
# below before trusting a clean result, and before trusting a red one.
#
# What this catches: a bare double-quoted string starting with an ASCII
# letter, passed directly as the first argument to `text(`, `button(`,
# `text_input(`, or `tooltip(` (the widget constructors this project's
# two sweeps actually found violations in), or to rfd's `.set_title(`
# (the one native-dialog title site Task 034 found). Restricted to the
# widget call itself, not every string literal in a file - a blanket
# literal-string grep is what produced most of the false-positive noise
# during Task 034's own re-derivation (log/panic/test/doc-comment text).
#
# What this does NOT catch:
#   - a literal built via format!(...) or assigned to a `let` first and
#     used later (most of Task 034's own findings were exactly this shape)
#   - user-facing text reaching any widget constructor not listed above
#   - native OS API calls other than rfd's `set_title`
#   - a string that is technically inside `mod tests` but the file
#     structures its tests unusually (this script assumes one trailing
#     `mod tests { ... }` block per file, matching this codebase's
#     established convention - see CLAUDE.md-adjacent precedent in
#     every reviewed package to date)
#
# This is a net, not a proof. It exists to catch the repeated shape that
# produced both of this project's sweeps - a raw literal handed straight
# to a widget constructor - not to replace a human sweep entirely.
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

fail=0

while IFS= read -r -d '' file; do
    # Track brace depth from the start of a top-level `mod tests {` line
    # to its matching close, and suppress matches in that range - a
    # plain grep would otherwise flag this script's own test fixtures
    # and every other project's `mod tests` block full of assertion
    # strings, which is exactly the noise Task 034 had to filter out by
    # hand while re-deriving its own sweep.
    matches=$(awk '
        BEGIN { in_test = 0; depth = 0 }
        /^mod tests[[:space:]]*\{/ && !in_test {
            in_test = 1
            depth = 1
            next
        }
        in_test {
            depth += gsub(/\{/, "{")
            depth -= gsub(/\}/, "}")
            if (depth <= 0) { in_test = 0 }
            next
        }
        /(text|button|text_input|tooltip)\([[:space:]]*"[A-Za-z]/ { print FNR": "$0; next }
        /\.set_title\([[:space:]]*"[A-Za-z]/ { print FNR": "$0 }
    ' "$file")

    if [ -n "$matches" ]; then
        fail=1
        while IFS= read -r line; do
            printf '%s:%s\n' "$file" "$line"
        done <<< "$matches"
    fi
done < <(find app/src crates/ui -name '*.rs' -print0)

if [ "$fail" -ne 0 ]; then
    printf '\nuntranslated-strings check failed: widget call(s) above pass a bare string literal instead of t(...)/t_with(...).\n' >&2
    printf 'If a match is a false positive (not user-facing, or already handled another way), say so in the review rather than silencing this check.\n' >&2
    exit 1
fi

printf 'untranslated-strings check passed\n'
