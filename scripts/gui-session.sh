#!/usr/bin/env bash
# One shared session for real-UI GUI evidence runs (T073), required by the
# owner's GUI test rules (a working-copy file, not in this repository):
# "Start every real-UI session with the project's shared session script
# (isolation, window or viewport size, capture, teardown)."
#
# It does not drive the app: clicks, AT-SPI actions and key sequences stay
# in each task's own script, which sources this one. This script only
# answers "is it safe to start", "where does the app run, isolated from
# everything real", "what size, obtained how", "which window is it",
# "how do I capture it", and "how do I clean up".
#
# Linux + niri only (this project's own development setup); say so rather
# than pretending otherwise on another platform or compositor.
#
# Every function below returns non-zero (via `return 1`, never just letting
# a failed check fall through) the moment anything it checks fails - this
# matters because this script is usually *sourced*, where `set -e` is not
# active (see the bottom of this file). A caller must check the result:
#
# Usage, sourced by a task's own script:
#   source scripts/gui-session.sh
#   gui_session_preflight || exit 1
#   gui_session_start || exit 1              # binary, size, fixture via env (below)
#   gui_session_focus_check || exit 1        # before every keystroke helper
#   gui_session_capture "01-initial-state" || exit 1
#   gui_session_stop || exit 1
#
# `gui_session_decoy_start`/`gui_session_decoy_stop` start and stop a second,
# throwaway, isolated instance of the binary - for demonstrating preflight's
# "already running" refusal only, never for evidence of the app itself.
#
# Usage, run directly (a self-test / demonstration, used by this task's own
# acceptance evidence): see `main` at the bottom.
#
# Configuration (environment variables, all optional except GUI_SESSION_BINARY):
#   GUI_SESSION_BINARY    Path to the arama binary under test. Required.
#   GUI_SESSION_WIDTH     Logical px. Default 1280.
#   GUI_SESSION_HEIGHT    Logical px. Default 720.
#   GUI_SESSION_FIXTURE   Path to a directory to copy into the isolated
#                         ARAMA_DATA_HOME before launch (a pre-seeded
#                         settings.json, a model directory, a gallery). Not
#                         set: the session starts with nothing, same as a
#                         first run.
#   GUI_SESSION_KEEP_PROFILE  "1" to keep the scratch profile after
#                             teardown (for debugging a failure). Default:
#                             removed.
#
# After `gui_session_start`, these are set for the caller to use:
#   GUI_SESSION_DIR          the scratch directory for this session
#   GUI_SESSION_EVIDENCE_DIR captures and session.txt land here
#   GUI_SESSION_NIRI_SOCKET  the nested compositor's own IPC socket -
#                            every `niri msg` call about the session under
#                            test must set NIRI_SOCKET to this, not the
#                            outer desktop's
#   GUI_SESSION_WINDOW_ID    the arama window's id inside the nested session
#   GUI_SESSION_ARAMA_PID    the launched arama process's own pid (from `$!`
#                            at launch - never rediscovered later by name)
#   GUI_SESSION_BACKEND      "wayland" or "x11" - which this session's
#                            WAYLAND_DISPLAY/DISPLAY point arama at
#
# `gui_session_start` and `gui_session_decoy_start` each install their own
# `EXIT` trap for emergency cleanup. Each chains to whatever `EXIT` trap the
# calling shell already had (captured with `trap -p` before installing its
# own), so sourcing this script does not silently discard a caller's own
# cleanup trap - but a caller that *itself* changes its `EXIT` trap after
# calling `gui_session_start` will lose this script's cleanup in turn; call
# `gui_session_stop`/`gui_session_decoy_stop` explicitly rather than relying
# on trap ordering across more than one layer.

GUI_SESSION_SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly GUI_SESSION_SCRIPT_DIR

gui_session_log() {
    printf '[gui-session] %s\n' "$1" >&2
}

gui_session_fail() {
    printf '[gui-session] refused: %s\n' "$1" >&2
    return 1
}

# ---------------------------------------------------------------------------
# 1. Preflight
# ---------------------------------------------------------------------------

# Refuse to start unless every condition holds. Does not touch anything;
# only inspects. Call before `gui_session_start`.
gui_session_preflight() {
    local binary="${GUI_SESSION_BINARY:?set GUI_SESSION_BINARY to the arama binary under test}"
    if [ ! -x "$binary" ]; then
        gui_session_fail "GUI_SESSION_BINARY ($binary) is not an executable file"
        return 1
    fi

    local proc_name
    proc_name="$(basename "$binary")"
    if pgrep -x "$proc_name" >/dev/null 2>&1; then
        gui_session_fail "a running '$proc_name' instance already exists (pid(s): $(pgrep -x "$proc_name" | tr '\n' ' '))"
        return 1
    fi

    # RFC 041: these two legacy folders are resolved relative to the
    # *executable*, read for migration regardless of ARAMA_DATA_HOME - a
    # stale one beside the binary under test leaks into every session
    # that binary runs, no matter how the rest of the environment is
    # isolated.
    local binary_dir legacy_local legacy_cache
    binary_dir="$(cd "$(dirname "$binary")" && pwd)"
    legacy_local="$binary_dir/.arama-local"
    legacy_cache="$binary_dir/.arama-cache"
    if [ -e "$legacy_local" ]; then
        gui_session_fail "a legacy '$legacy_local' exists beside the binary under test - remove it or use a clean copy of the binary"
        return 1
    fi
    if [ -e "$legacy_cache" ]; then
        gui_session_fail "a legacy '$legacy_cache' exists beside the binary under test - remove it or use a clean copy of the binary"
        return 1
    fi

    if ! command -v niri >/dev/null 2>&1; then
        gui_session_fail "niri is not installed - this script is niri-only"
        return 1
    fi
    if [ -z "${WAYLAND_DISPLAY:-}" ]; then
        gui_session_fail "no outer WAYLAND_DISPLAY set - a nested niri session needs a Wayland host to nest inside"
        return 1
    fi

    # Owner's rule (2026-11-14, after incident report 164): nothing is
    # created in the home directory. `gui_session_start` always computes its
    # own in-project values for these five variables, but if the calling
    # shell already has one of them exported to something outside the
    # project - a leftover from an earlier unisolated command, for instance
    # - that is caught here, before anything else runs, rather than trusting
    # it to be overwritten later. Unset is fine at this point: `start` sets
    # it. Set-and-wrong is not.
    gui_session__reject_homes_outside_project || return 1

    gui_session_log "preflight ok: no running '$proc_name', no stale legacy folders, niri available, outer Wayland session present"
}

# Checks only the five isolation variables that are *already set* in the
# current environment - does not require them (unset is fine; a caller that
# has not reached `gui_session_start`/`gui_session_decoy_start` yet has not
# set them). Any one of them, if set, must resolve inside the project.
# Returns immediately on the first violation - never merely prints and
# continues, so a caller checking this function's own return value never
# sees success after a real violation (review 165, B1).
gui_session__reject_homes_outside_project() {
    local var path resolved
    for var in ARAMA_DATA_HOME XDG_CONFIG_HOME XDG_STATE_HOME XDG_CACHE_HOME XDG_DATA_HOME; do
        path="${!var:-}"
        [ -n "$path" ] || continue
        resolved="$(realpath -m -- "$path")"
        case "$resolved" in
            "$GUI_SESSION_SCRIPT_DIR"/*) ;;
            *)
                gui_session_fail "$var ($path) resolves outside the project ($GUI_SESSION_SCRIPT_DIR) - refusing before any launch"
                return 1
                ;;
        esac
    done
}

# The one place the binary is ever executed from this script - by
# `gui_session_start`'s real session and by `gui_session_decoy_start`'s
# throwaway instance alike. Unlike the preflight check above, this requires
# all five to be set, not merely valid-if-set: a launch with any of them
# missing falls back to arama's real platform defaults (RFC 041's
# `local_dir()`, and the toolkit's own `$HOME`-based XDG defaults), which is
# exactly how incident report 164 happened - a launch outside this script,
# with none of them set. There is no other way to start the binary from this
# script than through this function, and it cannot reach the exec line after
# any failed check, whatever the caller's shell options (review 165, B1).
#
# Writes the launched pid to $2 via `$!` - the caller reads it from there,
# never by searching for the process afterward (review 165, B2): a launch
# wrapped in an explicit `(...)` subshell (to scope DISPLAY/WAYLAND_DISPLAY)
# cannot hand back `$!` as a shell variable, so a pid file is the only way
# out of that subshell.
gui_session__launch_binary() {
    local binary="$1"
    local pidfile="$2"
    local var
    for var in ARAMA_DATA_HOME XDG_CONFIG_HOME XDG_STATE_HOME XDG_CACHE_HOME XDG_DATA_HOME; do
        if [ -z "${!var:-}" ]; then
            gui_session_fail "refusing to launch '$binary': $var is not set - every launch through this script must be isolated"
            return 1
        fi
    done
    gui_session__reject_homes_outside_project || return 1
    "$binary" >>"${GUI_SESSION_LAUNCH_LOG:-/dev/null}" 2>&1 &
    echo $! >"$pidfile"
}

# ---------------------------------------------------------------------------
# 2+3. Isolation and launch
# ---------------------------------------------------------------------------

# A fixture under the project's own `.git-exclude/fixtures/` (the only tree
# with a SHA256SUMS contract) is verified against it before use - a
# corrupted, substituted, or partially-unlisted fixture is refused, not
# silently pre-seeded. A fixture outside that tree (an ad hoc directory
# during development) has no checksum file to verify against and is used
# as-is, logged as such.
gui_session__verify_fixture_checksums() {
    local fixture="$1"
    local fixtures_root="$GUI_SESSION_SCRIPT_DIR/.git-exclude/fixtures"
    local sums="$fixtures_root/SHA256SUMS"

    case "$fixture" in
        "$fixtures_root" | "$fixtures_root"/*) ;;
        *)
            gui_session_log "fixture $fixture is outside .git-exclude/fixtures - no SHA256SUMS to verify against, using as-is"
            return 0
            ;;
    esac

    if [ ! -f "$sums" ]; then
        gui_session_fail "fixture $fixture is under .git-exclude/fixtures but no SHA256SUMS exists at $sums"
        return 1
    fi

    local rel="${fixture#"$fixtures_root"/}"
    local filtered
    filtered="$(grep -F "  ./$rel/" "$sums" || true)"
    if [ -z "$filtered" ]; then
        gui_session_fail "SHA256SUMS has no entries under ./$rel/ - fixture is unrecorded"
        return 1
    fi

    # Every file actually present under the fixture must have a listed
    # entry - otherwise a stray, unlisted file (dropped in by hand) would be
    # pre-seeded unverified (review 165, item 3). `sha256sum -c` alone only
    # checks the files it is given; it says nothing about extras.
    local actual_count listed_count
    actual_count="$(find "$fixture" -type f | wc -l)"
    listed_count="$(printf '%s\n' "$filtered" | wc -l)"
    if [ "$actual_count" -ne "$listed_count" ]; then
        gui_session_fail "fixture $fixture has $actual_count file(s) on disk but SHA256SUMS lists $listed_count under ./$rel/ - an unlisted or missing file is present"
        return 1
    fi

    if ! (cd "$fixtures_root" && printf '%s\n' "$filtered" | sha256sum -c - >/dev/null); then
        gui_session_fail "fixture checksum verification failed for $fixture against $sums"
        return 1
    fi
    gui_session_log "fixture checksums verified against $sums ($listed_count file(s))"
}

# The scale of a specific output, by name - never "the first output", since
# a multi-monitor desktop's first output need not be the one holding the
# nested compositor's own window (review 165, item 6).
gui_session__scale_for_output() {
    local output_name="$1"
    niri msg -j outputs | python3 -c "
import json, sys
outputs = json.load(sys.stdin)
o = outputs.get('$output_name')
if o:
    print(o['logical']['scale'])
"
}

# Installs `gui_session__emergency_cleanup` as the `EXIT` trap, first saving
# whatever `EXIT` trap the calling shell already had so the cleanup handler
# can chain to it (review 165, item 7) - sourcing this script must not
# silently discard a caller's own trap.
#
# Installs once per shell (review 166, R1): a second call in the same shell
# - `gui_session_decoy_start` followed by `gui_session_start`, say - would
# otherwise find *this very handler* already installed and save it as
# "prior", and at exit the handler would `eval` itself, forever. Guarded by
# a plain flag, not by inspecting the trap's own text, since this handler's
# own name appearing in `trap -p EXIT`'s output is exactly the case to
# avoid matching against.
#
# The prior command is extracted with `eval "set -- $prior"` in a
# subshell (so the `set --` cannot clobber the caller's own positional
# parameters) rather than by stripping a literal `trap -- '` prefix and
# `' EXIT` suffix - the latter breaks on a prior command that itself
# contains a single quote, since `trap -p` escapes those as `'\''` rather
# than leaving them for a plain substring strip to undo.
gui_session__install_exit_trap() {
    if [ "${GUI_SESSION__TRAP_INSTALLED:-0}" = "1" ]; then
        return 0
    fi
    GUI_SESSION__TRAP_INSTALLED=1

    local prior
    prior="$(trap -p EXIT)"
    if [ -n "$prior" ]; then
        GUI_SESSION__PRIOR_EXIT_TRAP="$(eval "set -- $prior" && printf '%s' "$3")"
    else
        GUI_SESSION__PRIOR_EXIT_TRAP=""
    fi
    trap gui_session__emergency_cleanup EXIT
}

# Best-effort, called only if the shell running `gui_session_start` or
# `gui_session_decoy_start` exits (success or failure) without
# `gui_session_stop`/`gui_session_decoy_stop` having already run -
# otherwise a script that fails partway through leaves the nested
# compositor and, worse, a real arama instance running, which then fails
# the *next* session's preflight too. Does not remove the scratch profile:
# a failure's evidence is left for inspection, same as
# `GUI_SESSION_KEEP_PROFILE=1`'s own intent.
#
# Kills only the exact pids this script itself launched and recorded -
# never `pkill`/`pgrep` by process name (review 165, B2): a name-based kill
# here could hit an arama the owner opened for real, mid-session, on their
# own desktop.
gui_session__emergency_cleanup() {
    if [ "${GUI_SESSION__DECOY_STOPPED:-0}" != "1" ]; then
        if [ -n "${GUI_SESSION_DECOY_PID:-}" ]; then
            kill -9 "$GUI_SESSION_DECOY_PID" 2>/dev/null || true
        fi
        if [ -n "${GUI_SESSION_DECOY_NIRI_PID:-}" ]; then
            kill -9 "$GUI_SESSION_DECOY_NIRI_PID" 2>/dev/null || true
        fi
    fi
    if [ "${GUI_SESSION__STOPPED:-0}" != "1" ]; then
        if [ -n "${GUI_SESSION_ARAMA_PID:-}" ]; then
            kill -9 "$GUI_SESSION_ARAMA_PID" 2>/dev/null || true
        fi
        if [ -n "${GUI_SESSION_DIR:-}" ] && [ -f "$GUI_SESSION_DIR/niri-nested.pid" ]; then
            kill -9 "$(cat "$GUI_SESSION_DIR/niri-nested.pid")" 2>/dev/null || true
        fi
    fi
    if [ -n "${GUI_SESSION__PRIOR_EXIT_TRAP:-}" ]; then
        eval "$GUI_SESSION__PRIOR_EXIT_TRAP"
    fi
}

gui_session_start() {
    local binary="${GUI_SESSION_BINARY:?set GUI_SESSION_BINARY to the arama binary under test}"
    local width="${GUI_SESSION_WIDTH:-1280}"
    local height="${GUI_SESSION_HEIGHT:-720}"

    local stamp
    stamp="$(date +%Y%m%d-%H%M%S)-$$"
    GUI_SESSION_DIR="$GUI_SESSION_SCRIPT_DIR/.git-exclude/tmp/gui-session-$stamp"
    GUI_SESSION_EVIDENCE_DIR="$GUI_SESSION_DIR/evidence"
    local profile_dir="$GUI_SESSION_DIR/profile"
    mkdir -p "$GUI_SESSION_EVIDENCE_DIR" \
        "$profile_dir/data" "$profile_dir/config" "$profile_dir/state" \
        "$profile_dir/cache" "$profile_dir/share"
    export GUI_SESSION_DIR GUI_SESSION_EVIDENCE_DIR
    gui_session__install_exit_trap

    if [ -n "${GUI_SESSION_FIXTURE:-}" ]; then
        if [ ! -d "$GUI_SESSION_FIXTURE" ]; then
            gui_session_fail "GUI_SESSION_FIXTURE ($GUI_SESSION_FIXTURE) is not a directory"
            return 1
        fi
        local fixture_abs
        fixture_abs="$(cd "$GUI_SESSION_FIXTURE" && pwd)"
        gui_session__verify_fixture_checksums "$fixture_abs" || return 1
        gui_session_log "pre-seeding ARAMA_DATA_HOME from $GUI_SESSION_FIXTURE"
        # `--reflink=auto`: a free, copy-on-write duplicate on a filesystem
        # that supports it (e.g. btrfs, xfs), an ordinary copy otherwise -
        # this fixture can be ~940 MB (review 165, item 8).
        cp -a --reflink=auto "$fixture_abs"/. "$profile_dir/data"/
    fi

    # --- isolation: every directory arama or the toolkit reads ---------
    export ARAMA_DATA_HOME="$profile_dir/data"
    export XDG_CONFIG_HOME="$profile_dir/config"
    export XDG_STATE_HOME="$profile_dir/state"
    export XDG_CACHE_HOME="$profile_dir/cache"
    export XDG_DATA_HOME="$profile_dir/share"

    # --- nested compositor, configured to fullscreen its one window on its
    # own first configure - never resized after the fact (review 165, B4:
    # the rule is "never resize a running window", so the window's final
    # size must come from how it opens, not from a command run against it
    # once it exists).
    local nested_config="$GUI_SESSION_DIR/niri-nested.kdl"
    cat >"$nested_config" <<'EOF'
window-rule {
    open-fullscreen true
}
EOF

    local nested_log="$GUI_SESSION_DIR/niri-nested.log"
    (
        cd "$GUI_SESSION_DIR"
        env -u ARAMA_DATA_HOME -u XDG_CONFIG_HOME -u XDG_STATE_HOME -u XDG_CACHE_HOME -u XDG_DATA_HOME \
            niri -c "$nested_config" >"$nested_log" 2>&1 &
        echo $! >"$GUI_SESSION_DIR/niri-nested.pid"
    )
    local nested_pid
    nested_pid="$(cat "$GUI_SESSION_DIR/niri-nested.pid")"

    local nested_socket=""
    local waited=0
    while [ -z "$nested_socket" ] && [ "$waited" -lt 50 ]; do
        sleep 0.1
        nested_socket="$(grep -oP 'IPC listening on: \K\S+' "$nested_log" 2>/dev/null || true)"
        waited=$((waited + 1))
    done
    if [ -z "$nested_socket" ]; then
        gui_session_fail "nested niri did not report an IPC socket within 5s - see $nested_log"
        return 1
    fi
    GUI_SESSION_NIRI_SOCKET="$nested_socket"
    export GUI_SESSION_NIRI_SOCKET

    # The nested Wayland socket name (e.g. "wayland-2") is assigned by
    # the compositor, not chosen here - never assumed to be "wayland-1".
    local nested_wayland_socket
    nested_wayland_socket="$(grep -oP 'listening on Wayland socket: \K\S+' "$nested_log" 2>/dev/null || true)"
    if [ -z "$nested_wayland_socket" ]; then
        gui_session_fail "nested niri did not report its Wayland socket name - see $nested_log"
        return 1
    fi

    local outer_window_json
    outer_window_json="$(niri msg -j windows | python3 -c "
import json, sys
windows = json.load(sys.stdin)
for w in windows:
    if w['pid'] == $nested_pid:
        print(json.dumps(w))
        break
")"
    if [ -z "$outer_window_json" ]; then
        gui_session_fail "could not find the nested niri's own window in the outer session"
        return 1
    fi
    local outer_window_id outer_workspace_id
    outer_window_id="$(printf '%s' "$outer_window_json" | python3 -c 'import json,sys; print(json.load(sys.stdin)["id"])')"
    outer_workspace_id="$(printf '%s' "$outer_window_json" | python3 -c 'import json,sys; print(json.load(sys.stdin)["workspace_id"])')"

    # The scale of the *specific* output the nested window actually opened
    # on - not assumed to be the first output niri happens to list (review
    # 165, item 6).
    local outer_output_name
    outer_output_name="$(niri msg -j workspaces | python3 -c "
import json, sys
for w in json.load(sys.stdin):
    if w['id'] == $outer_workspace_id:
        print(w.get('output') or '')
        break
")"
    if [ -z "$outer_output_name" ]; then
        gui_session_fail "could not determine which output the nested niri's window opened on"
        return 1
    fi
    local scale
    scale="$(gui_session__scale_for_output "$outer_output_name")"
    if [ -z "$scale" ]; then
        gui_session_fail "could not read the display scale of output '$outer_output_name'"
        return 1
    fi

    local outer_width outer_height
    outer_width="$(python3 -c "print(round($width / $scale))")"
    outer_height="$(python3 -c "print(round($height / $scale))")"

    niri msg action focus-window --id "$outer_window_id" >/dev/null
    niri msg action set-column-width -- "$outer_width" >/dev/null
    niri msg action set-window-height --id "$outer_window_id" -- "$outer_height" >/dev/null
    sleep 0.3

    local inner_width inner_height
    inner_width="$(NIRI_SOCKET="$GUI_SESSION_NIRI_SOCKET" niri msg -j outputs | python3 -c '
import json, sys
outputs = json.load(sys.stdin)
for o in outputs.values():
    print(o["logical"]["width"])
    break
')"
    inner_height="$(NIRI_SOCKET="$GUI_SESSION_NIRI_SOCKET" niri msg -j outputs | python3 -c '
import json, sys
outputs = json.load(sys.stdin)
for o in outputs.values():
    print(o["logical"]["height"])
    break
')"
    if [ "$inner_width" != "$width" ] || [ "$inner_height" != "$height" ]; then
        gui_session_fail "nested compositor is ${inner_width}x${inner_height}, expected ${width}x${height} - the outer resize did not land exactly; do not fall back to resizing the app window instead"
        return 1
    fi
    gui_session_log "nested compositor sized at launch: ${width}x${height} (outer window ${outer_width}x${outer_height} logical at scale $scale on output $outer_output_name)"

    # --- launch arama inside the nested session, already at its final size --
    GUI_SESSION_BACKEND="${GUI_SESSION_BACKEND:-wayland}"
    export GUI_SESSION_BACKEND
    local arama_log="$GUI_SESSION_DIR/arama.log"
    local proc_name
    proc_name="$(basename "$binary")"

    local nested_x11_display=""
    if [ "$GUI_SESSION_BACKEND" = "x11" ]; then
        # Likewise assigned by the compositor, not chosen here.
        nested_x11_display="$(grep -oP 'listening on X11 socket: :\K\d+' "$nested_log" 2>/dev/null || true)"
        if [ -z "$nested_x11_display" ]; then
            gui_session_fail "nested niri did not report its X11 socket - see $nested_log"
            return 1
        fi
        nested_x11_display=":$nested_x11_display"
    fi

    # This nested compositor is private to this one session, so a window
    # count of exactly zero before the launch and exactly one after is
    # enough on its own to identify arama's window - no title, class or pid
    # needed (review 165, B3; replaces the earlier X11 title-substring
    # match, itself a replacement for an X11-class match that cannot work
    # at all: arama's X11 window sets no `WM_CLASS`, confirmed empirically,
    # T073). Read via niri's own IPC only, never an X11 client query
    # directly - niri spawns its XWayland satellite lazily, and a direct
    # query (e.g. `xdotool`) against a not-yet-ready satellite socket can
    # segfault, not just answer slowly (T073).
    local windows_before
    windows_before="$(NIRI_SOCKET="$GUI_SESSION_NIRI_SOCKET" niri msg -j windows | python3 -c 'import json, sys; print(len(json.load(sys.stdin)))')"
    if [ "$windows_before" -ne 0 ]; then
        gui_session_fail "nested compositor already has $windows_before window(s) before launch - expected 0, cannot identify arama's window unambiguously"
        return 1
    fi

    GUI_SESSION_LAUNCH_LOG="$arama_log"
    local pidfile="$GUI_SESSION_DIR/arama.pid"
    if [ "$GUI_SESSION_BACKEND" = "x11" ]; then
        (
            unset WAYLAND_DISPLAY
            export DISPLAY="$nested_x11_display"
            gui_session__launch_binary "$binary" "$pidfile"
        ) || return 1
    else
        (
            unset DISPLAY
            export WAYLAND_DISPLAY="$nested_wayland_socket"
            gui_session__launch_binary "$binary" "$pidfile"
        ) || return 1
    fi

    local arama_pid=""
    [ -f "$pidfile" ] && arama_pid="$(cat "$pidfile")"
    if [ -z "$arama_pid" ]; then
        gui_session_fail "launch produced no pid - see $arama_log"
        return 1
    fi
    GUI_SESSION_ARAMA_PID="$arama_pid"
    export GUI_SESSION_ARAMA_PID

    local window_id="" window_pid="" windows_json count
    waited=0
    while [ -z "$window_id" ] && [ "$waited" -lt 100 ]; do
        sleep 0.1
        windows_json="$(NIRI_SOCKET="$GUI_SESSION_NIRI_SOCKET" niri msg -j windows)"
        count="$(printf '%s' "$windows_json" | python3 -c 'import json, sys; print(len(json.load(sys.stdin)))')"
        if [ "$count" -gt 1 ]; then
            gui_session_fail "nested compositor has $count windows - expected exactly 1, cannot identify which is ours"
            return 1
        fi
        if [ "$count" -eq 1 ]; then
            window_id="$(printf '%s' "$windows_json" | python3 -c 'import json, sys; print(json.load(sys.stdin)[0]["id"])')"
            window_pid="$(printf '%s' "$windows_json" | python3 -c 'import json, sys; print(json.load(sys.stdin)[0]["pid"])')"
        fi
        waited=$((waited + 1))
    done
    if [ -z "$window_id" ]; then
        gui_session_fail "'$proc_name' started (pid $arama_pid) but no window appeared within 10s - see $arama_log"
        return 1
    fi
    # On Wayland the window's own pid must be arama's launched pid - an
    # extra assertion on top of the window-count check above (review 165,
    # B3: "apply the same check on Wayland, in addition to the pid match").
    # Not checked on X11: niri reports the XWayland bridge's pid for an
    # X11 window, never the app's own.
    if [ "$GUI_SESSION_BACKEND" != "x11" ] && [ "$window_pid" != "$arama_pid" ]; then
        gui_session_fail "the nested compositor's one window has pid $window_pid, not the launched instance's pid $arama_pid"
        return 1
    fi
    GUI_SESSION_WINDOW_ID="$window_id"
    export GUI_SESSION_WINDOW_ID

    # The window opened fullscreen by the nested compositor's own config
    # (above) - checked here, not commanded: a size that is wrong means the
    # config did not land as expected, which is a refusal, not something to
    # paper over with a resize (review 165, B4).
    local reported_width reported_height
    reported_width="$(printf '%s' "$windows_json" | python3 -c 'import json, sys; print(json.load(sys.stdin)[0]["layout"]["window_size"][0])')"
    reported_height="$(printf '%s' "$windows_json" | python3 -c 'import json, sys; print(json.load(sys.stdin)[0]["layout"]["window_size"][1])')"
    if [ "$reported_width" != "$width" ] || [ "$reported_height" != "$height" ]; then
        gui_session_fail "arama's window opened at ${reported_width}x${reported_height}, expected ${width}x${height} - open-fullscreen did not land exactly"
        return 1
    fi

    gui_session_log "session ready: pid=$GUI_SESSION_ARAMA_PID window=$GUI_SESSION_WINDOW_ID backend=$GUI_SESSION_BACKEND size=${width}x${height}"

    local arama_version
    arama_version="$(sed -n '/^\[workspace.package\]/,/^\[/p' "$GUI_SESSION_SCRIPT_DIR/Cargo.toml" | grep -m1 '^version' | sed -E 's/version[^"]*"([^"]*)".*/\1/')"

    {
        printf 'backend: %s\n' "$GUI_SESSION_BACKEND"
        printf 'window_size_logical: %sx%s\n' "$width" "$height"
        printf 'outer_display_scale: %s (output %s)\n' "$scale" "$outer_output_name"
        printf 'isolation_arama_data_home: %s\n' "${ARAMA_DATA_HOME#"$GUI_SESSION_SCRIPT_DIR/"}"
        printf 'isolation_xdg_config_home: %s\n' "${XDG_CONFIG_HOME#"$GUI_SESSION_SCRIPT_DIR/"}"
        printf 'isolation_xdg_state_home: %s\n' "${XDG_STATE_HOME#"$GUI_SESSION_SCRIPT_DIR/"}"
        printf 'isolation_xdg_cache_home: %s\n' "${XDG_CACHE_HOME#"$GUI_SESSION_SCRIPT_DIR/"}"
        printf 'isolation_xdg_data_home: %s\n' "${XDG_DATA_HOME#"$GUI_SESSION_SCRIPT_DIR/"}"
        printf 'fixture: %s\n' "${GUI_SESSION_FIXTURE:-none}"
        printf 'binary: %s\n' "${binary#"$GUI_SESSION_SCRIPT_DIR/"}"
        printf 'binary_mtime: %s\n' "$(date -r "$binary" +%Y-%m-%dT%H:%M:%S 2>/dev/null || echo unknown)"
        printf 'arama_version: %s\n' "${arama_version:-unknown}"
        printf 'project_commit_at_session_time: %s\n' "$(git -C "$GUI_SESSION_SCRIPT_DIR" rev-parse --short HEAD 2>/dev/null || echo unknown)"
        printf 'started_at: %s\n' "$(date -Iseconds)"
    } >"$GUI_SESSION_EVIDENCE_DIR/session.txt"
}

# ---------------------------------------------------------------------------
# 4. Focus check - call before every keystroke helper
# ---------------------------------------------------------------------------

gui_session_focus_check() {
    if [ -z "${GUI_SESSION_NIRI_SOCKET:-}" ]; then
        gui_session_fail "gui_session_focus_check called before gui_session_start"
        return 1
    fi
    local focused_pid
    focused_pid="$(NIRI_SOCKET="$GUI_SESSION_NIRI_SOCKET" niri msg -j focused-window | python3 -c 'import json, sys; w = json.load(sys.stdin); print(w.get("pid", ""))' 2>/dev/null || true)"

    if [ "$GUI_SESSION_BACKEND" = "x11" ]; then
        # niri reports the XWayland bridge's pid for an X11 window, not
        # the app's own - compare the window id instead.
        local focused_id
        focused_id="$(NIRI_SOCKET="$GUI_SESSION_NIRI_SOCKET" niri msg -j focused-window | python3 -c 'import json, sys; w = json.load(sys.stdin); print(w.get("id", ""))' 2>/dev/null || true)"
        if [ "$focused_id" != "$GUI_SESSION_WINDOW_ID" ]; then
            gui_session_fail "focused window id ($focused_id) is not the instance under test ($GUI_SESSION_WINDOW_ID) - another window took focus"
            return 1
        fi
        return 0
    fi

    if [ "$focused_pid" != "$GUI_SESSION_ARAMA_PID" ]; then
        gui_session_fail "focused window pid ($focused_pid) is not the instance under test ($GUI_SESSION_ARAMA_PID) - another window took focus"
        return 1
    fi
}

# ---------------------------------------------------------------------------
# 5. Capture
# ---------------------------------------------------------------------------

gui_session_capture() {
    local name="${1:?gui_session_capture needs a name}"
    if [ -z "${GUI_SESSION_NIRI_SOCKET:-}" ]; then
        gui_session_fail "gui_session_capture called before gui_session_start"
        return 1
    fi
    local path="$GUI_SESSION_EVIDENCE_DIR/$name.png"
    NIRI_SOCKET="$GUI_SESSION_NIRI_SOCKET" niri msg action screenshot-window \
        --id "$GUI_SESSION_WINDOW_ID" --path "$path" >/dev/null
    gui_session_log "captured $path"
    printf '%s\n' "$path"
}

# ---------------------------------------------------------------------------
# 8. Teardown
# ---------------------------------------------------------------------------

# Kills only the pid `gui_session_start` recorded at launch - never by
# process name (review 165, B2). An arama the owner opens for real, during
# the session, is a different pid and is left alone.
gui_session_stop() {
    if [ -z "${GUI_SESSION_DIR:-}" ]; then
        gui_session_fail "gui_session_stop called before gui_session_start"
        return 1
    fi

    if [ -n "${GUI_SESSION_ARAMA_PID:-}" ]; then
        kill "$GUI_SESSION_ARAMA_PID" 2>/dev/null || true
        local waited=0
        while kill -0 "$GUI_SESSION_ARAMA_PID" 2>/dev/null && [ "$waited" -lt 50 ]; do
            sleep 0.1
            waited=$((waited + 1))
        done
        if kill -0 "$GUI_SESSION_ARAMA_PID" 2>/dev/null; then
            kill -9 "$GUI_SESSION_ARAMA_PID" 2>/dev/null || true
            sleep 0.2
        fi
        if kill -0 "$GUI_SESSION_ARAMA_PID" 2>/dev/null; then
            gui_session_fail "pid $GUI_SESSION_ARAMA_PID is still running after teardown"
            return 1
        fi
        gui_session_log "confirmed pid $GUI_SESSION_ARAMA_PID is gone"
    fi

    if [ -f "$GUI_SESSION_DIR/niri-nested.pid" ]; then
        kill "$(cat "$GUI_SESSION_DIR/niri-nested.pid")" 2>/dev/null || true
    fi

    if [ "${GUI_SESSION_KEEP_PROFILE:-0}" != "1" ]; then
        rm -rf "$GUI_SESSION_DIR/profile"
        gui_session_log "scratch profile removed; evidence kept at $GUI_SESSION_EVIDENCE_DIR"
    else
        gui_session_log "scratch profile kept at $GUI_SESSION_DIR/profile (GUI_SESSION_KEEP_PROFILE=1)"
    fi

    GUI_SESSION__STOPPED=1
}

# ---------------------------------------------------------------------------
# 9. Decoy instance - for preflight-refusal testing only
# ---------------------------------------------------------------------------

# A second, throwaway, isolated instance of the same binary, used solely to
# demonstrate preflight's "an instance is already running" refusal (T073
# §7) - never for evidence of the app itself. Goes through
# `gui_session__launch_binary` exactly like the real session does: isolated
# first, launched second, no other way (incident report 164).
gui_session_decoy_start() {
    local binary="${GUI_SESSION_BINARY:?set GUI_SESSION_BINARY to the arama binary under test}"

    local stamp
    stamp="$(date +%Y%m%d-%H%M%S)-$$"
    GUI_SESSION_DECOY_DIR="$GUI_SESSION_SCRIPT_DIR/.git-exclude/tmp/gui-session-decoy-$stamp"
    local profile_dir="$GUI_SESSION_DECOY_DIR/profile"
    mkdir -p "$profile_dir/data" "$profile_dir/config" "$profile_dir/state" \
        "$profile_dir/cache" "$profile_dir/share"
    gui_session__install_exit_trap

    local nested_config="$GUI_SESSION_DECOY_DIR/niri-nested.kdl"
    : >"$nested_config"
    local nested_log="$GUI_SESSION_DECOY_DIR/niri-nested.log"
    (
        cd "$GUI_SESSION_DECOY_DIR"
        env -u ARAMA_DATA_HOME -u XDG_CONFIG_HOME -u XDG_STATE_HOME -u XDG_CACHE_HOME -u XDG_DATA_HOME \
            niri -c "$nested_config" >"$nested_log" 2>&1 &
        echo $! >"$GUI_SESSION_DECOY_DIR/niri-nested.pid"
    )
    GUI_SESSION_DECOY_NIRI_PID="$(cat "$GUI_SESSION_DECOY_DIR/niri-nested.pid")"

    local nested_wayland_socket=""
    local waited=0
    while [ -z "$nested_wayland_socket" ] && [ "$waited" -lt 50 ]; do
        sleep 0.1
        nested_wayland_socket="$(grep -oP 'listening on Wayland socket: \K\S+' "$nested_log" 2>/dev/null || true)"
        waited=$((waited + 1))
    done
    if [ -z "$nested_wayland_socket" ]; then
        gui_session_fail "decoy's nested niri did not report its Wayland socket name - see $nested_log"
        return 1
    fi

    local pidfile="$GUI_SESSION_DECOY_DIR/arama.pid"
    (
        export ARAMA_DATA_HOME="$profile_dir/data"
        export XDG_CONFIG_HOME="$profile_dir/config"
        export XDG_STATE_HOME="$profile_dir/state"
        export XDG_CACHE_HOME="$profile_dir/cache"
        export XDG_DATA_HOME="$profile_dir/share"
        unset DISPLAY
        export WAYLAND_DISPLAY="$nested_wayland_socket"
        GUI_SESSION_LAUNCH_LOG="$GUI_SESSION_DECOY_DIR/arama.log"
        gui_session__launch_binary "$binary" "$pidfile"
    ) || return 1

    local pid=""
    [ -f "$pidfile" ] && pid="$(cat "$pidfile")"
    if [ -z "$pid" ]; then
        gui_session_fail "decoy launch produced no pid"
        return 1
    fi
    waited=0
    while ! kill -0 "$pid" 2>/dev/null && [ "$waited" -lt 50 ]; do
        sleep 0.1
        waited=$((waited + 1))
    done
    if ! kill -0 "$pid" 2>/dev/null; then
        gui_session_fail "decoy process (pid $pid) exited immediately after launch"
        return 1
    fi
    GUI_SESSION_DECOY_PID="$pid"
    gui_session_log "decoy running (pid $pid, isolated under ${GUI_SESSION_DECOY_DIR#"$GUI_SESSION_SCRIPT_DIR/"}) - for preflight-refusal testing only"
}

gui_session_decoy_stop() {
    if [ -z "${GUI_SESSION_DECOY_DIR:-}" ]; then
        gui_session_fail "gui_session_decoy_stop called before gui_session_decoy_start"
        return 1
    fi
    if [ -n "${GUI_SESSION_DECOY_PID:-}" ]; then
        kill -9 "$GUI_SESSION_DECOY_PID" 2>/dev/null || true
    fi
    if [ -n "${GUI_SESSION_DECOY_NIRI_PID:-}" ]; then
        kill -9 "$GUI_SESSION_DECOY_NIRI_PID" 2>/dev/null || true
    fi
    rm -rf "$GUI_SESSION_DECOY_DIR"
    GUI_SESSION__DECOY_STOPPED=1
    gui_session_log "decoy stopped and removed"
}

# ---------------------------------------------------------------------------
# Self-test / demonstration, when run directly rather than sourced
# ---------------------------------------------------------------------------

gui_session__is_sourced() {
    [ "${BASH_SOURCE[0]}" != "${0}" ]
}

main() {
    : "${GUI_SESSION_BINARY:?set GUI_SESSION_BINARY to the arama binary under test}"
    gui_session_preflight || exit 1
    gui_session_start || exit 1
    gui_session_capture "self-test" || exit 1
    gui_session_stop || exit 1
    gui_session_log "self-test complete: $GUI_SESSION_EVIDENCE_DIR"
}

if ! gui_session__is_sourced; then
    # Strict mode only on the direct-execution path - a sourced library
    # must not change its caller's shell options.
    set -euo pipefail
    main "$@"
fi
