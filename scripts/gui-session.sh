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
# Usage, sourced by a task's own script:
#   source scripts/gui-session.sh
#   gui_session_preflight
#   gui_session_start                      # binary, size, fixture via env (below)
#   gui_session_focus_check                # before every keystroke helper
#   gui_session_capture "01-initial-state"
#   gui_session_stop
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
#   GUI_SESSION_ARAMA_PID    the launched arama process's pid
#   GUI_SESSION_BACKEND      "wayland" or "x11" - which this session's
#                            WAYLAND_DISPLAY/DISPLAY point arama at

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
    [ -x "$binary" ] || gui_session_fail "GUI_SESSION_BINARY ($binary) is not an executable file"

    local proc_name
    proc_name="$(basename "$binary")"
    if pgrep -x "$proc_name" >/dev/null 2>&1; then
        gui_session_fail "a running '$proc_name' instance already exists (pid(s): $(pgrep -x "$proc_name" | tr '\n' ' '))"
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
    fi
    if [ -e "$legacy_cache" ]; then
        gui_session_fail "a legacy '$legacy_cache' exists beside the binary under test - remove it or use a clean copy of the binary"
    fi

    command -v niri >/dev/null 2>&1 || gui_session_fail "niri is not installed - this script is niri-only"
    [ -n "${WAYLAND_DISPLAY:-}" ] || gui_session_fail "no outer WAYLAND_DISPLAY set - a nested niri session needs a Wayland host to nest inside"

    # Owner's rule (2026-11-14, after incident report 164): nothing is
    # created in the home directory. `gui_session_start` always computes its
    # own in-project values for these five variables, but if the calling
    # shell already has one of them exported to something outside the
    # project - a leftover from an earlier unisolated command, for instance
    # - that is caught here, before anything else runs, rather than trusting
    # it to be overwritten later. Unset is fine at this point: `start` sets
    # it. Set-and-wrong is not.
    gui_session__reject_homes_outside_project

    gui_session_log "preflight ok: no running '$proc_name', no stale legacy folders, niri available, outer Wayland session present"
}

# Checks only the five isolation variables that are *already set* in the
# current environment - does not require them (unset is fine; a caller that
# has not reached `gui_session_start`/`gui_session_decoy_start` yet has not
# set them). Any one of them, if set, must resolve inside the project.
gui_session__reject_homes_outside_project() {
    local var path resolved
    for var in ARAMA_DATA_HOME XDG_CONFIG_HOME XDG_STATE_HOME XDG_CACHE_HOME XDG_DATA_HOME; do
        path="${!var:-}"
        [ -n "$path" ] || continue
        resolved="$(realpath -m -- "$path")"
        case "$resolved" in
            "$GUI_SESSION_SCRIPT_DIR"/*) ;;
            *) gui_session_fail "$var ($path) resolves outside the project ($GUI_SESSION_SCRIPT_DIR) - refusing before any launch" ;;
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
# script than through this function.
gui_session__launch_binary() {
    local binary="$1"
    local var
    for var in ARAMA_DATA_HOME XDG_CONFIG_HOME XDG_STATE_HOME XDG_CACHE_HOME XDG_DATA_HOME; do
        [ -n "${!var:-}" ] || gui_session_fail "refusing to launch '$binary': $var is not set - every launch through this script must be isolated"
    done
    gui_session__reject_homes_outside_project
    ("$binary" >>"${GUI_SESSION_LAUNCH_LOG:-/dev/null}" 2>&1 &)
}

# ---------------------------------------------------------------------------
# 2+3. Isolation and launch
# ---------------------------------------------------------------------------

# A fixture under the project's own `.git-exclude/fixtures/` (the only tree
# with a SHA256SUMS contract) is verified against it before use - a
# corrupted or substituted fixture is refused, not silently pre-seeded. A
# fixture outside that tree (an ad hoc directory during development) has no
# checksum file to verify against and is used as-is, logged as such.
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

    [ -f "$sums" ] || gui_session_fail "fixture $fixture is under .git-exclude/fixtures but no SHA256SUMS exists at $sums"

    local rel="${fixture#"$fixtures_root"/}"
    local filtered
    filtered="$(grep -F "  ./$rel/" "$sums" || true)"
    [ -n "$filtered" ] || gui_session_fail "SHA256SUMS has no entries under ./$rel/ - fixture is unrecorded"

    if ! (cd "$fixtures_root" && printf '%s\n' "$filtered" | sha256sum -c - >/dev/null); then
        gui_session_fail "fixture checksum verification failed for $fixture against $sums"
    fi
    gui_session_log "fixture checksums verified against $sums"
}

# Read the outer compositor's display scale dynamically - never assumed -
# since the nested compositor's own window must be sized in outer logical
# px to land on an exact inner physical resolution.
gui_session__outer_scale() {
    niri msg -j outputs | python3 -c '
import json, sys
outputs = json.load(sys.stdin)
for output in outputs.values():
    scale = output.get("logical", {}).get("scale")
    if scale:
        print(scale)
        break
'
}

# Best-effort, called only if the shell running `gui_session_start` exits
# (success or failure) without `gui_session_stop` having already run -
# otherwise a script that fails partway through `gui_session_start` (or
# is interrupted) leaves the nested compositor and, worse, a real arama
# instance running, which then fails the *next* session's preflight too.
# Does not remove the scratch profile: a failure's evidence is left for
# inspection, same as `GUI_SESSION_KEEP_PROFILE=1`'s own intent.
gui_session__emergency_cleanup() {
    if [ "${GUI_SESSION__DECOY_STOPPED:-0}" != "1" ]; then
        if [ -n "${GUI_SESSION_DECOY_NIRI_PID:-}" ]; then
            kill -9 "$GUI_SESSION_DECOY_NIRI_PID" 2>/dev/null || true
        fi
        if [ -n "${GUI_SESSION_BINARY:-}" ] && [ -n "${GUI_SESSION_DECOY_PID:-}" ]; then
            pkill -9 -x "$(basename "$GUI_SESSION_BINARY")" 2>/dev/null || true
        fi
    fi
    [ "${GUI_SESSION__STOPPED:-0}" = "1" ] && return 0
    if [ -n "${GUI_SESSION_ARAMA_PID:-}" ]; then
        kill -9 "$GUI_SESSION_ARAMA_PID" 2>/dev/null || true
    fi
    if [ -n "${GUI_SESSION_DIR:-}" ] && [ -f "$GUI_SESSION_DIR/niri-nested.pid" ]; then
        kill -9 "$(cat "$GUI_SESSION_DIR/niri-nested.pid")" 2>/dev/null || true
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
    trap gui_session__emergency_cleanup EXIT

    if [ -n "${GUI_SESSION_FIXTURE:-}" ]; then
        [ -d "$GUI_SESSION_FIXTURE" ] || gui_session_fail "GUI_SESSION_FIXTURE ($GUI_SESSION_FIXTURE) is not a directory"
        local fixture_abs
        fixture_abs="$(cd "$GUI_SESSION_FIXTURE" && pwd)"
        gui_session__verify_fixture_checksums "$fixture_abs"
        gui_session_log "pre-seeding ARAMA_DATA_HOME from $GUI_SESSION_FIXTURE"
        cp -a "$fixture_abs"/. "$profile_dir/data"/
    fi

    # --- isolation: every directory arama or the toolkit reads ---------
    export ARAMA_DATA_HOME="$profile_dir/data"
    export XDG_CONFIG_HOME="$profile_dir/config"
    export XDG_STATE_HOME="$profile_dir/state"
    export XDG_CACHE_HOME="$profile_dir/cache"
    export XDG_DATA_HOME="$profile_dir/share"

    # --- nested compositor, sized before anything runs inside it -------
    local nested_config="$GUI_SESSION_DIR/niri-nested.kdl"
    : >"$nested_config" # niri's default config is enough; no mode block -
    # the winit backend ignores an `output` mode (confirmed empirically,
    # T073) - the outer-window resize below is what actually sizes it.

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
    [ -n "$nested_socket" ] || gui_session_fail "nested niri did not report an IPC socket within 5s - see $nested_log"
    GUI_SESSION_NIRI_SOCKET="$nested_socket"
    export GUI_SESSION_NIRI_SOCKET

    # The nested Wayland socket name (e.g. "wayland-2") is assigned by
    # the compositor, not chosen here - never assumed to be "wayland-1".
    local nested_wayland_socket
    nested_wayland_socket="$(grep -oP 'listening on Wayland socket: \K\S+' "$nested_log" 2>/dev/null || true)"
    [ -n "$nested_wayland_socket" ] || gui_session_fail "nested niri did not report its Wayland socket name - see $nested_log"

    local outer_window_id
    outer_window_id="$(niri msg -j windows | python3 -c "
import json, sys
windows = json.load(sys.stdin)
for w in windows:
    if w['pid'] == $nested_pid:
        print(w['id'])
        break
")"
    [ -n "$outer_window_id" ] || gui_session_fail "could not find the nested niri's own window in the outer session"

    local scale
    scale="$(gui_session__outer_scale)"
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
    fi
    gui_session_log "nested compositor sized at launch: ${width}x${height} (outer window ${outer_width}x${outer_height} logical at scale $scale)"

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
        [ -n "$nested_x11_display" ] || gui_session_fail "nested niri did not report its X11 socket - see $nested_log"
        nested_x11_display=":$nested_x11_display"
    fi

    GUI_SESSION_LAUNCH_LOG="$arama_log"
    if [ "$GUI_SESSION_BACKEND" = "x11" ]; then
        (unset WAYLAND_DISPLAY; export DISPLAY="$nested_x11_display"; gui_session__launch_binary "$binary")
    else
        (unset DISPLAY; export WAYLAND_DISPLAY="$nested_wayland_socket"; gui_session__launch_binary "$binary")
    fi

    local arama_pid=""
    waited=0
    while [ -z "$arama_pid" ] && [ "$waited" -lt 100 ]; do
        sleep 0.1
        arama_pid="$(NIRI_SOCKET="$GUI_SESSION_NIRI_SOCKET" pgrep -x "$proc_name" || true)"
        waited=$((waited + 1))
    done
    [ -n "$arama_pid" ] || gui_session_fail "'$proc_name' did not start within 10s - see $arama_log"
    GUI_SESSION_ARAMA_PID="$arama_pid"
    export GUI_SESSION_ARAMA_PID

    # Window id: by pid directly on Wayland. On X11, niri reports the
    # XWayland bridge's pid for an X11 window, not the app's own, and
    # (confirmed empirically, T073: niri's own window JSON and
    # `xdotool getwindowclassname` both come back empty) arama's X11
    # window sets no WM_CLASS at all - so match on title instead, the
    # one identifier it does set ("Arama - Iced"). This queries niri's
    # own IPC only, never XWayland directly, so it carries none of the
    # lazily-spawned-satellite segfault risk a direct X11 client query
    # (e.g. `xdotool`) had (T073).
    local window_id=""
    waited=0
    while [ -z "$window_id" ] && [ "$waited" -lt 100 ]; do
        sleep 0.1
        if [ "$GUI_SESSION_BACKEND" = "x11" ]; then
            window_id="$(NIRI_SOCKET="$GUI_SESSION_NIRI_SOCKET" niri msg -j windows | python3 -c "
import json, sys
windows = json.load(sys.stdin)
for w in windows:
    if '$proc_name'.lower() in (w.get('title') or '').lower():
        print(w['id'])
        break
")"
        else
            window_id="$(NIRI_SOCKET="$GUI_SESSION_NIRI_SOCKET" niri msg -j windows | python3 -c "
import json, sys
windows = json.load(sys.stdin)
for w in windows:
    if w['pid'] == $arama_pid:
        print(w['id'])
        break
")"
        fi
        waited=$((waited + 1))
    done
    [ -n "$window_id" ] || gui_session_fail "'$proc_name' started (pid $arama_pid) but its window never appeared within 10s - see $arama_log"
    GUI_SESSION_WINDOW_ID="$window_id"
    export GUI_SESSION_WINDOW_ID

    # niri is a scrolling-tiling compositor: a newly opened window gets a
    # default column width, not the whole output. This is the one
    # nested-session-only window, so fullscreen it to the output's exact
    # resolution - not a resize of an already-running app window (the
    # rule's actual concern), but giving it the dedicated nested display
    # it was launched into, done once, immediately, before anything is
    # captured or measured.
    NIRI_SOCKET="$GUI_SESSION_NIRI_SOCKET" niri msg action fullscreen-window --id "$window_id" >/dev/null
    sleep 0.3

    gui_session_log "session ready: pid=$GUI_SESSION_ARAMA_PID window=$GUI_SESSION_WINDOW_ID backend=$GUI_SESSION_BACKEND size=${width}x${height}"

    {
        printf 'backend: %s\n' "$GUI_SESSION_BACKEND"
        printf 'window_size_logical: %sx%s\n' "$width" "$height"
        printf 'outer_display_scale: %s\n' "$scale"
        printf 'isolation_arama_data_home: %s\n' "${ARAMA_DATA_HOME#"$GUI_SESSION_SCRIPT_DIR/"}"
        printf 'isolation_xdg_config_home: %s\n' "${XDG_CONFIG_HOME#"$GUI_SESSION_SCRIPT_DIR/"}"
        printf 'isolation_xdg_state_home: %s\n' "${XDG_STATE_HOME#"$GUI_SESSION_SCRIPT_DIR/"}"
        printf 'isolation_xdg_cache_home: %s\n' "${XDG_CACHE_HOME#"$GUI_SESSION_SCRIPT_DIR/"}"
        printf 'isolation_xdg_data_home: %s\n' "${XDG_DATA_HOME#"$GUI_SESSION_SCRIPT_DIR/"}"
        printf 'fixture: %s\n' "${GUI_SESSION_FIXTURE:-none}"
        printf 'binary: %s\n' "$binary"
        printf 'binary_mtime: %s\n' "$(date -r "$binary" +%Y-%m-%dT%H:%M:%S 2>/dev/null || echo unknown)"
        printf 'project_commit: %s\n' "$(git -C "$GUI_SESSION_SCRIPT_DIR" rev-parse --short HEAD 2>/dev/null || echo unknown)"
        printf 'started_at: %s\n' "$(date -Iseconds)"
    } >"$GUI_SESSION_EVIDENCE_DIR/session.txt"
}

# ---------------------------------------------------------------------------
# 4. Focus check - call before every keystroke helper
# ---------------------------------------------------------------------------

gui_session_focus_check() {
    [ -n "${GUI_SESSION_NIRI_SOCKET:-}" ] || gui_session_fail "gui_session_focus_check called before gui_session_start"
    local focused_pid
    focused_pid="$(NIRI_SOCKET="$GUI_SESSION_NIRI_SOCKET" niri msg -j focused-window | python3 -c 'import json, sys; w = json.load(sys.stdin); print(w.get("pid", ""))' 2>/dev/null || true)"

    if [ "$GUI_SESSION_BACKEND" = "x11" ]; then
        # niri reports the XWayland bridge's pid for an X11 window, not
        # the app's own - compare the window id instead.
        local focused_id
        focused_id="$(NIRI_SOCKET="$GUI_SESSION_NIRI_SOCKET" niri msg -j focused-window | python3 -c 'import json, sys; w = json.load(sys.stdin); print(w.get("id", ""))' 2>/dev/null || true)"
        [ "$focused_id" = "$GUI_SESSION_WINDOW_ID" ] || gui_session_fail "focused window id ($focused_id) is not the instance under test ($GUI_SESSION_WINDOW_ID) - another window took focus"
        return 0
    fi

    [ "$focused_pid" = "$GUI_SESSION_ARAMA_PID" ] || gui_session_fail "focused window pid ($focused_pid) is not the instance under test ($GUI_SESSION_ARAMA_PID) - another window took focus"
}

# ---------------------------------------------------------------------------
# 5. Capture
# ---------------------------------------------------------------------------

gui_session_capture() {
    local name="${1:?gui_session_capture needs a name}"
    [ -n "${GUI_SESSION_NIRI_SOCKET:-}" ] || gui_session_fail "gui_session_capture called before gui_session_start"
    local path="$GUI_SESSION_EVIDENCE_DIR/$name.png"
    NIRI_SOCKET="$GUI_SESSION_NIRI_SOCKET" niri msg action screenshot-window \
        --id "$GUI_SESSION_WINDOW_ID" --path "$path" >/dev/null
    gui_session_log "captured $path"
    printf '%s\n' "$path"
}

# ---------------------------------------------------------------------------
# 8. Teardown
# ---------------------------------------------------------------------------

gui_session_stop() {
    [ -n "${GUI_SESSION_DIR:-}" ] || gui_session_fail "gui_session_stop called before gui_session_start"

    if [ -n "${GUI_SESSION_ARAMA_PID:-}" ]; then
        local proc_name
        proc_name="$(basename "${GUI_SESSION_BINARY:-arama}")"
        kill "$GUI_SESSION_ARAMA_PID" 2>/dev/null || true
        local waited=0
        while pgrep -x "$proc_name" >/dev/null 2>&1 && [ "$waited" -lt 50 ]; do
            sleep 0.1
            waited=$((waited + 1))
        done
        if pgrep -x "$proc_name" >/dev/null 2>&1; then
            pkill -9 -x "$proc_name" 2>/dev/null || true
        fi
        if pgrep -x "$proc_name" >/dev/null 2>&1; then
            gui_session_fail "'$proc_name' is still running after teardown"
        fi
        gui_session_log "confirmed zero '$proc_name' instances remain"
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
    local proc_name
    proc_name="$(basename "$binary")"

    local stamp
    stamp="$(date +%Y%m%d-%H%M%S)-$$"
    GUI_SESSION_DECOY_DIR="$GUI_SESSION_SCRIPT_DIR/.git-exclude/tmp/gui-session-decoy-$stamp"
    local profile_dir="$GUI_SESSION_DECOY_DIR/profile"
    mkdir -p "$profile_dir/data" "$profile_dir/config" "$profile_dir/state" \
        "$profile_dir/cache" "$profile_dir/share"
    trap gui_session__emergency_cleanup EXIT

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
    [ -n "$nested_wayland_socket" ] || gui_session_fail "decoy's nested niri did not report its Wayland socket name - see $nested_log"

    (
        export ARAMA_DATA_HOME="$profile_dir/data"
        export XDG_CONFIG_HOME="$profile_dir/config"
        export XDG_STATE_HOME="$profile_dir/state"
        export XDG_CACHE_HOME="$profile_dir/cache"
        export XDG_DATA_HOME="$profile_dir/share"
        unset DISPLAY
        export WAYLAND_DISPLAY="$nested_wayland_socket"
        GUI_SESSION_LAUNCH_LOG="$GUI_SESSION_DECOY_DIR/arama.log"
        gui_session__launch_binary "$binary"
    )

    local pid=""
    waited=0
    while [ -z "$pid" ] && [ "$waited" -lt 50 ]; do
        sleep 0.1
        pid="$(pgrep -x "$proc_name" || true)"
        waited=$((waited + 1))
    done
    [ -n "$pid" ] || gui_session_fail "decoy '$proc_name' did not start within 5s"
    GUI_SESSION_DECOY_PID="$pid"
    gui_session_log "decoy '$proc_name' running (pid $pid, isolated under ${GUI_SESSION_DECOY_DIR#"$GUI_SESSION_SCRIPT_DIR/"}) - for preflight-refusal testing only"
}

gui_session_decoy_stop() {
    [ -n "${GUI_SESSION_DECOY_DIR:-}" ] || gui_session_fail "gui_session_decoy_stop called before gui_session_decoy_start"
    local proc_name
    proc_name="$(basename "${GUI_SESSION_BINARY:-arama}")"
    pkill -9 -x "$proc_name" 2>/dev/null || true
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
    gui_session_preflight
    gui_session_start
    gui_session_capture "self-test"
    gui_session_stop
    gui_session_log "self-test complete: $GUI_SESSION_EVIDENCE_DIR"
}

if ! gui_session__is_sourced; then
    # Strict mode only on the direct-execution path - a sourced library
    # must not change its caller's shell options.
    set -euo pipefail
    main "$@"
fi
