#!/bin/sh
# CodeEraser plugin starter (ADR-007 unique distribution path). The
# plugin repo ships only this script; the real `ce` binary resolves in
# order: (1) a copy in CLAUDE_PLUGIN_DATA *or on PATH* whose SHA256
# matches the pin in bin/manifest.env — since v0.7.3 the installer
# leaves one on PATH and a byte-identical fetch is pure waste; (2) a
# fresh download verified against that pin, placed atomically; (3) an
# UNVERIFIED PATH `ce` (air-gapped stance, also the only path while
# pins are empty, and it says so out loud). A pin mismatch REFUSES the
# artifact loudly and keeps nothing, so a tampered one cannot hide
# behind a local copy. Fail-open (R3): no binary => one line, exit 0.
#
# EVERY human line here goes to STDERR. The plan's hook protocol is
# `exit 2 + stderr` OR `exit 0 + {"hookSpecificOutput":…}`, and this
# script always exec's into a hook whose stdout must be that JSON and
# nothing else — a notice printed before it made the whole stream
# unparseable, and the project's own readers (which parse the entire
# stdout) silently dropped the decision.
set -u

here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
# CE_MANIFEST_FILE is a test seam: the e2e battery points it at a
# fixture manifest so the shipped pins stay inert during tests.
. "${CE_MANIFEST_FILE:-$here/manifest.env}"

plat_key() {
    case "$(uname -s)" in
        MINGW*|MSYS*|CYGWIN*) echo "x86_64-windows" ;;
        Darwin) case "$(uname -m)" in
                    arm64) echo "aarch64-macos" ;;
                    *) echo "x86_64-macos" ;;
                esac ;;
        *) case "$(uname -m)" in
               x86_64) echo "x86_64-linux" ;;
               aarch64) echo "aarch64-linux" ;;
               *) echo "unsupported" ;;
           esac ;;
    esac
}

have_hasher() {
    command -v sha256sum >/dev/null 2>&1 || command -v shasum >/dev/null 2>&1
}

sha_of() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | cut -d' ' -f1
    else
        shasum -a 256 "$1" | cut -d' ' -f1
    fi
}

fetch() { # $1=url $2=dest — file:// is the hermetic test transport
    case "$1" in
        file://*) cp "${1#file://}" "$2" ;;
        # bounded on purpose (portals that never finish must fail the
        # hook, not hang it); --proto-redir because --proto guards only
        # the FIRST request and the bytes arrive over GitHub's redirect
        *) curl -fsSL --proto '=https' --proto-redir '=https' --max-time 120 --max-filesize 104857600 -o "$2" "$1" ;;
    esac
}

run_path_ce() {
    if command -v ce >/dev/null 2>&1; then
        # A pin EXISTS for this platform yet we are here: the verified
        # copy could not be produced. An unverified exec must not be
        # indistinguishable from a pin-checked one, so it says so. An
        # EMPTY pin is the documented air-gapped/preview stance (or a
        # platform this release never built) and stays quiet.
        [ -z "$pin" ] || echo "codeeraser: pin unverified for $key — running PATH ce unchecked" >&2
        exec ce "$@"
    fi
    echo "codeeraser: no ce binary (no verified copy, no PATH ce) — install with 'cargo install --path cli' or place a pinned binary in CLAUDE_PLUGIN_DATA" >&2
    exit 0
}

key=$(plat_key)
ext=""
case "$key" in x86_64-windows) ext=".exe" ;; esac
pin=""
if [ "$key" != "unsupported" ]; then
    eval "pin=\${CE_SHA256_$(echo "$key" | tr 'a-z-' 'A-Z_')_CE:-}"
fi
data="${CLAUDE_PLUGIN_DATA:-}"

# ce-core rides the same pin flow (from v0.2.0): placed as a plain
# `ce-core` sibling in the data dir — exactly where ce's resolver
# probes — BEFORE ce execs. EVERY core failure degrades, none dies:
# hooks never need the core (judgment families report their own
# handshake miss), so killing the starter over it takes the guard
# down for a component the guard does not use. A pin mismatch still
# refuses the artifact as loudly as ce's own and keeps NOTHING —
# a tampered core must not hide behind a PATH ce-core.
ensure_core() {
    corepin=""
    if [ "$key" != "unsupported" ]; then
        eval "corepin=\${CE_SHA256_$(echo "$key" | tr 'a-z-' 'A-Z_')_CECORE:-}"
    fi
    if [ -z "$corepin" ] || [ -z "$data" ]; then return 0; fi
    coretgt="$data/ce-core$ext"
    # Same rule as ce below: a PATH ce-core that MATCHES the pin is the
    # artifact the download would place (the installer leaves one), so
    # it BECOMES the bind target. Unmatched PATH cores stay refused —
    # e2e state 8 is the standing guard on that.
    pathcore=$(command -v ce-core 2>/dev/null || true)
    [ -n "$pathcore" ] && [ "$(sha_of "$pathcore" 2>/dev/null)" = "$corepin" ] && coretgt="$pathcore"
    # A pin EXISTS for this platform, so BIND ce to the pinned path
    # instead of leaving its resolver free. Removing a mismatching
    # core (below) or failing to fetch one used to leave the resolver
    # walking on to a PATH `ce-core` — unverified, and the exact
    # "tampered core hiding behind a PATH ce-core" this header
    # forbids. Pointing at an absent file degrades honestly: the
    # judgment families report a handshake miss, which is what a
    # missing verified core IS. An explicit CE_CORE_BIN wins (source
    # installs pick their own core deliberately).
    CE_CORE_BIN="${CE_CORE_BIN:-$coretgt}"
    export CE_CORE_BIN
    if [ -x "$coretgt" ]; then
        if [ "$(sha_of "$coretgt")" = "$corepin" ]; then
            return 0
        fi
        # A MISMATCHING copy already on disk is the same artifact the
        # download path refuses outright, so it gets the same answer:
        # removed here, before any return can leave it in place for
        # ce's resolver to pick up as a plain sibling. The old code
        # only rejected a bad DOWNLOAD, so one prior file write pinned
        # an attacker's core into every later session.
        echo "codeeraser: REFUSING on-disk ce-core — SHA256 mismatch, removing $coretgt" >&2
        rm -f "$coretgt"
    fi
    if [ -n "${CE_AIRGAPPED:-}" ]; then return 0; fi
    coreurl="${CE_BOOTSTRAP_BASE_URL:-$CE_BASE_URL}/ce-core-$CE_MANIFEST_VERSION-$key$ext"
    # PID-suffixed: two sessions sharing one CLAUDE_PLUGIN_DATA used to
    # curl into the SAME path, and the interleaved bytes then failed the
    # hash — a security-grade "REFUSING" for a benign race, plus an
    # rm -f that deleted the sibling's file mid-verify.
    coretmp="$coretgt.download.$$"
    if ! fetch "$coreurl" "$coretmp"; then
        rm -f "$coretmp"
        echo "codeeraser: ce-core download failed ($coreurl) — judgment families degrade until it lands" >&2
        return 0
    fi
    coregot=$(sha_of "$coretmp")
    if [ "$coregot" != "$corepin" ]; then
        rm -f "$coretmp"
        echo "codeeraser: REFUSING downloaded ce-core — SHA256 mismatch for $coreurl" >&2
        echo "codeeraser: expected $corepin" >&2
        echo "codeeraser: actual   $coregot" >&2
        # Refuse the artifact, then DEGRADE — the on-disk mismatch
        # branch above already takes exactly this stance and this one
        # did not. `exit 1` here killed the starter before it ever
        # exec'd, so one tampered (or merely stale-CDN) core blocked
        # every hook on the machine for a component no hook consults.
        # Nothing was placed, and CE_CORE_BIN is already bound to the
        # now-absent verified path, so ce's resolver still cannot walk
        # on to an unverified PATH ce-core: the judgment families
        # report a handshake miss, which is the truth.
        return 0
    fi
    chmod +x "$coretmp"
    mv -f "$coretmp" "$coretgt"
}

# No pin or no data dir: the download path is not configured — PATH ce.
if [ -z "$pin" ] || [ -z "$data" ]; then
    run_path_ce "$@"
fi

# A host with neither hasher cannot verify anything, and R3 says
# DEGRADE, not die: the old code produced an empty hash, compared it
# against the pin, and so re-downloaded the binary and exited 1 on
# every single hook.
if ! have_hasher; then
    echo "codeeraser: no sha256sum/shasum here — cannot verify the pin" >&2
    run_path_ce "$@"
fi

target="$data/ce-$CE_MANIFEST_VERSION-$key$ext"
# One rule, two places: whichever pin-identical copy already exists
# answers — the installer leaves one on PATH, and the same bytes are
# not worth fetching twice. Unmatched ones still face verification.
for cand in "$target" "$(command -v ce 2>/dev/null || true)"; do
    [ -n "$cand" ] && [ -x "$cand" ] && [ "$(sha_of "$cand" 2>/dev/null)" = "$pin" ] || continue
    ensure_core
    exec "$cand" "$@"
done

if [ -n "${CE_AIRGAPPED:-}" ]; then
    # Manual-placement mode: never download; a stale/absent copy just
    # falls back to PATH (the placement instructions live in README).
    run_path_ce "$@"
fi

mkdir -p "$data"
url="${CE_BOOTSTRAP_BASE_URL:-$CE_BASE_URL}/ce-$CE_MANIFEST_VERSION-$key$ext"
tmp="$target.download.$$" # PID-suffixed — see ensure_core
if ! fetch "$url" "$tmp"; then
    rm -f "$tmp"
    echo "codeeraser: download failed ($url) — falling back to PATH ce" >&2
    run_path_ce "$@"
fi
got=$(sha_of "$tmp")
if [ "$got" != "$pin" ]; then
    rm -f "$tmp"
    echo "codeeraser: REFUSING downloaded binary — SHA256 mismatch for $url" >&2
    echo "codeeraser: expected $pin" >&2
    echo "codeeraser: actual   $got" >&2
    exit 1
fi
chmod +x "$tmp"
mv -f "$tmp" "$target"
ensure_core
exec "$target" "$@"
