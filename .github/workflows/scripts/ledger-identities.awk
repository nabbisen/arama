# T066: extracts every RUSTSEC identity textually present in
# rfcs/notes/audit-warning-burn-down.md, including one named only inside a
# grouped comma- or "through"-list immediately after a full identity -
# the two shapes the ledger actually uses:
#
#   "RUSTSEC-2026-0367 (...), 0368 (...), 0369 (...)."
#   "RUSTSEC-2026-0338 through 0342."
#
# A naive per-identity grep misses every bare suffix in both shapes - the
# false-miss T066's handoff names explicitly. This does not parse
# arbitrary future prose; it expands exactly these two documented shapes,
# bounded to the sentence (up to the next ".") that follows the anchor, so
# an unrelated number elsewhere in the same paragraph is never pulled in.
#
# RS="" makes each blank-line-delimited paragraph one record; its internal
# line-wrap newlines are folded to spaces first, since the grouped lists
# here wrap across lines like any other prose.
BEGIN { RS = "" }
{
    para = $0
    gsub(/\n/, " ", para)
    rest = para
    while (match(rest, /RUSTSEC-[0-9]{4}-[0-9]{3,5}/)) {
        anchor = substr(rest, RSTART, RLENGTH)
        print anchor
        year = substr(anchor, 9, 4)
        anchor_num = substr(anchor, 14)
        tail_start = RSTART + RLENGTH
        tail = substr(rest, tail_start)

        period = index(tail, ".")
        if (period > 0) {
            group = substr(tail, 1, period - 1)
        } else {
            group = tail
        }

        # Skip one leading parenthetical describing the anchor itself
        # (e.g. "RUSTSEC-2026-0367 (SUID/SGID preservation...), 0368 ...")
        # before checking whether a group list follows.
        check = group
        if (match(check, /^[ \t]*\([^()]*\)/)) {
            check = substr(check, RSTART + RLENGTH)
        }

        if (check ~ /^[ \t]*(,|through)/) {
            g = check
            prev = anchor_num
            while (match(g, /(,|through)[ \t]*[0-9]{3,5}/)) {
                piece = substr(g, RSTART, RLENGTH)
                is_through = (piece ~ /through/)
                num = piece
                gsub(/[^0-9]/, "", num)
                if (is_through && prev != "") {
                    for (n = prev + 0; n <= num + 0; n++) {
                        printf "RUSTSEC-%s-%04d\n", year, n
                    }
                } else {
                    printf "RUSTSEC-%s-%s\n", year, num
                }
                prev = num
                g = substr(g, RSTART + RLENGTH)
            }
        }

        rest = substr(rest, tail_start)
    }
}
