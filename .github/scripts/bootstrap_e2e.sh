#!/bin/sh
# Bootstrap chain e2e (M7-P1 acceptance + the v0.2.0 core legs):
# drives plugin/bin/ce.sh through its six states with the just-built
# REAL ce binary as the payload and file:// as the transport
# (hermetic — the https leg is curl's contract, not ours; stated in
# the CI step name). States:
#   0 regression : empty pins -> PATH ce answers (air-gapped stance)
#   1 fresh      : pin + source -> download, verify, place, exec
#   2 warm       : source deleted -> cached verified copy answers
#   3 tampered   : wrong pin -> exit 1, loud refusal, nothing placed
#   4 core along : pinned ce-core placed as a plain sibling in data
#   5 bad core   : tampered ce-core refused as loudly as a tampered ce
#   6 quiet fail : a failed download falls back to PATH and keeps its
#                  notice OFF stdout (the hook stream is JSON only)
#   7 stale core : a MISMATCHING ce-core already on disk is refused and
#                  REMOVED, not left behind for ce's resolver
#   8 core bind  : with a pin in force ce is BOUND to the pinned core
#                  path, never free to fall through to a PATH ce-core
#   9 path reuse : a PATH ce that MATCHES the pin answers with nothing
#                  downloaded and no unverified notice (v0.7.3)
#  10 core reuse : state 8's mirror — a PATH ce-core that MATCHES the
#                  pin is bound instead of fetched
# Usage: bootstrap_e2e.sh <path-to-built-ce> <path-to-ce.sh>
set -eu

CE_BIN=$1
[ -f "$CE_BIN" ] || CE_BIN="$1.exe"
STARTER=$2
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

# Independent recomputation of ce.sh's platform key (lockstep on
# purpose: if the product's key grammar drifts, state 1 goes red).
case "$(uname -s)" in
    MINGW*|MSYS*|CYGWIN*) key="x86_64-windows"; ext=".exe" ;;
    Darwin) key="aarch64-macos"; ext="" ;;
    *) key="x86_64-linux"; ext="" ;;
esac
envkey=$(echo "$key" | tr 'a-z-' 'A-Z_')
want=$("$CE_BIN" --version)

sha_of() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | cut -d' ' -f1
    else
        shasum -a 256 "$1" | cut -d' ' -f1
    fi
}

fail() { echo "bootstrap_e2e: FAIL state $1: $2" >&2; exit 1; }

# --- state 0: empty pins fall back to PATH ce -------------------------
mkdir -p "$work/pathbin"
cp "$CE_BIN" "$work/pathbin/ce$ext"
cat >"$work/manifest0.env" <<EOF
CE_MANIFEST_VERSION="0.0.0-test"
CE_BASE_URL="file:///nonexistent"
CE_SHA256_${envkey}_CE=""
EOF
out=$(PATH="$work/pathbin:$PATH" CE_MANIFEST_FILE="$work/manifest0.env" \
      CLAUDE_PLUGIN_DATA="$work/data0" sh "$STARTER" --version)
[ "$out" = "$want" ] || fail 0 "PATH fallback answered '$out' not '$want'"

# --- state 1: fresh pinned download ----------------------------------
mkdir -p "$work/src"
payload="ce-0.0.0-test-$key$ext"
cp "$CE_BIN" "$work/src/$payload"
pin=$(sha_of "$work/src/$payload")
cat >"$work/manifest1.env" <<EOF
CE_MANIFEST_VERSION="0.0.0-test"
CE_BASE_URL="file:///nonexistent"
CE_SHA256_${envkey}_CE="$pin"
EOF
out=$(CE_MANIFEST_FILE="$work/manifest1.env" \
      CE_BOOTSTRAP_BASE_URL="file://$work/src" \
      CLAUDE_PLUGIN_DATA="$work/data1" sh "$STARTER" --version)
[ "$out" = "$want" ] || fail 1 "downloaded binary answered '$out' not '$want'"
[ -f "$work/data1/$payload" ] || fail 1 "verified binary not placed"

# --- state 2: warm — source gone, cached copy answers ----------------
rm "$work/src/$payload"
out=$(CE_MANIFEST_FILE="$work/manifest1.env" \
      CE_BOOTSTRAP_BASE_URL="file://$work/src" \
      CLAUDE_PLUGIN_DATA="$work/data1" sh "$STARTER" --version)
[ "$out" = "$want" ] || fail 2 "warm cache answered '$out' not '$want'"

# --- state 3: tampered — wrong pin refuses loudly, places nothing ----
cp "$CE_BIN" "$work/src/$payload"
cat >"$work/manifest3.env" <<EOF
CE_MANIFEST_VERSION="0.0.0-test"
CE_BASE_URL="file:///nonexistent"
CE_SHA256_${envkey}_CE="0000000000000000000000000000000000000000000000000000000000000000"
EOF
rc=0
err=$(CE_MANIFEST_FILE="$work/manifest3.env" \
      CE_BOOTSTRAP_BASE_URL="file://$work/src" \
      CLAUDE_PLUGIN_DATA="$work/data3" sh "$STARTER" --version 2>&1 >/dev/null) || rc=$?
[ "$rc" -ne 0 ] || fail 3 "tampered download did not refuse (rc=0)"
case "$err" in *"SHA256 mismatch"*) ;; *) fail 3 "refusal not loud: $err" ;; esac
[ ! -f "$work/data3/$payload" ] || fail 3 "tampered binary was placed"
# any .download* leftover, not just the un-suffixed name: the temp
# carries a PID so two sessions cannot curl into one path
left=$(ls "$work/data3" 2>/dev/null | grep -c '\.download' || true)
[ "$left" = "0" ] || fail 3 "tmp download not cleaned ($left left)"

# --- state 4: core rides along — pinned ce-core lands as a plain
# sibling in the data dir (exactly where ce's resolver probes); the
# payload is ce itself standing in for ce-core (any executable works,
# only placement + verification are under test) --------------------------
corepayload="ce-core-0.0.0-test-$key$ext"
cp "$CE_BIN" "$work/src/$corepayload"
corepin=$(sha_of "$work/src/$corepayload")
cat >"$work/manifest4.env" <<EOF
CE_MANIFEST_VERSION="0.0.0-test"
CE_BASE_URL="file:///nonexistent"
CE_SHA256_${envkey}_CE="$pin"
CE_SHA256_${envkey}_CECORE="$corepin"
EOF
out=$(CE_MANIFEST_FILE="$work/manifest4.env" \
      CE_BOOTSTRAP_BASE_URL="file://$work/src" \
      CLAUDE_PLUGIN_DATA="$work/data4" sh "$STARTER" --version)
[ "$out" = "$want" ] || fail 4 "ce with core leg answered '$out' not '$want'"
[ -f "$work/data4/ce-core$ext" ] || fail 4 "verified ce-core not placed as plain sibling"
[ "$(sha_of "$work/data4/ce-core$ext")" = "$corepin" ] || fail 4 "placed ce-core sha drifted"

# --- state 5: tampered core refuses as loudly as a tampered ce -------
cat >"$work/manifest5.env" <<EOF
CE_MANIFEST_VERSION="0.0.0-test"
CE_BASE_URL="file:///nonexistent"
CE_SHA256_${envkey}_CE="$pin"
CE_SHA256_${envkey}_CECORE="1111111111111111111111111111111111111111111111111111111111111111"
EOF
rc=0
err=$(CE_MANIFEST_FILE="$work/manifest5.env" \
      CE_BOOTSTRAP_BASE_URL="file://$work/src" \
      CLAUDE_PLUGIN_DATA="$work/data5" sh "$STARTER" --version 2>&1 >/dev/null) || rc=$?
[ "$rc" -ne 0 ] || fail 5 "tampered ce-core did not refuse (rc=0)"
case "$err" in *"REFUSING downloaded ce-core"*) ;; *) fail 5 "core refusal not loud: $err" ;; esac
[ ! -f "$work/data5/ce-core$ext" ] || fail 5 "tampered ce-core was placed"

# --- state 6: a failed download degrades to PATH, and its notice
# stays OFF stdout. The starter always exec's into a hook whose stdout
# must be the decision JSON alone; a human line printed there made the
# whole stream unparseable and the project's own readers (which parse
# the ENTIRE stdout) dropped the decision silently ------------------
# The pin here is deliberately BOGUS. From v0.7.3 a PATH ce whose sha
# matches the pin is reused instead of downloaded (state 9), and this
# state's PATH copy IS the pinned artifact byte-for-byte — with the
# real pin the starter would take the verified leg and this state
# would stop exercising the failed-download degrade it is named for.
cat >"$work/manifest6.env" <<EOF
CE_MANIFEST_VERSION="0.0.0-test"
CE_BASE_URL="file:///nonexistent"
CE_SHA256_${envkey}_CE="2222222222222222222222222222222222222222222222222222222222222222"
EOF
out=$(PATH="$work/pathbin:$PATH" CE_MANIFEST_FILE="$work/manifest6.env" \
      CE_BOOTSTRAP_BASE_URL="file://$work/gone" \
      CLAUDE_PLUGIN_DATA="$work/data6" sh "$STARTER" --version 2>/dev/null)
[ "$out" = "$want" ] || fail 6 "stdout carried more than the payload's own output: '$out'"

# --- state 7: a MISMATCHING ce-core already on disk is refused AND
# removed — the download leg always refused a bad core, but a bad one
# already placed survived every early return and stayed exactly where
# ce's resolver probes for a plain `ce-core` sibling ----------------
mkdir -p "$work/data7"
cp "$CE_BIN" "$work/data7/ce-core$ext"   # content does not match the pin below
rm -f "$work/src/$corepayload"           # and the download leg cannot replace it
cat >"$work/manifest7.env" <<EOF
CE_MANIFEST_VERSION="0.0.0-test"
CE_BASE_URL="file:///nonexistent"
CE_SHA256_${envkey}_CE="$pin"
CE_SHA256_${envkey}_CECORE="1111111111111111111111111111111111111111111111111111111111111111"
EOF
err=$(PATH="$work/pathbin:$PATH" CE_MANIFEST_FILE="$work/manifest7.env" \
      CE_BOOTSTRAP_BASE_URL="file://$work/src" \
      CLAUDE_PLUGIN_DATA="$work/data7" sh "$STARTER" --version 2>&1 >/dev/null) || true
case "$err" in *"REFUSING on-disk ce-core"*) ;; *) fail 7 "stale core not refused: $err" ;; esac
[ ! -f "$work/data7/ce-core$ext" ] || fail 7 "mismatching ce-core left in place"

# --- state 8: with a core pin in force, ce is BOUND to the pinned
# path. Removing a bad core (state 7) or failing to fetch one used to
# leave ce's resolver free to walk on to a PATH `ce-core` — unverified,
# the very substitution the header forbids. Airgapped so no download
# can rescue it, and an unverified ce-core planted on PATH as the
# thing that must NOT be picked; `ce doctor` names the core it tried.
mkdir -p "$work/pathcore" "$work/data8"
cp "$CE_BIN" "$work/pathcore/ce-core$ext"   # the unverified core to REFUSE
cp "$CE_BIN" "$work/data8/$payload"         # a VERIFIED ce, so ensure_core runs
out=$(PATH="$work/pathcore:$PATH" CE_MANIFEST_FILE="$work/manifest7.env" \
      CE_AIRGAPPED=1 CLAUDE_PLUGIN_DATA="$work/data8" \
      sh "$STARTER" doctor . 2>&1) || true
case "$out" in
    *"data8/ce-core"*) ;;
    *) fail 8 "ce resolved a core other than the pinned path: $out" ;;
esac

# --- state 9: a PATH ce whose sha MATCHES the pin is REUSED, never
# re-downloaded (v0.7.3). The installer puts ce on the machine PATH,
# so fetching a byte-identical second copy is pure waste. The proof is
# negative on BOTH sides, because stdout alone cannot tell this leg
# from the unverified fallback: nothing lands in the data dir, and the
# "pin unverified" notice run_path_ce always prints never appears.
mkdir -p "$work/data9"
: >"$work/out9"
err=$(PATH="$work/pathbin:$PATH" CE_MANIFEST_FILE="$work/manifest1.env" \
      CE_BOOTSTRAP_BASE_URL="file://$work/gone" \
      CLAUDE_PLUGIN_DATA="$work/data9" sh "$STARTER" --version 2>&1 >"$work/out9") || true
out=$(cat "$work/out9")
[ "$out" = "$want" ] || fail 9 "verified PATH ce answered '$out' not '$want'"
[ -z "$err" ] || fail 9 "verified leg was not silent (that is the unverified one): $err"
placed=$(ls "$work/data9" | wc -l)
[ "$placed" -eq 0 ] || fail 9 "a second copy was fetched despite a pin-identical PATH ce"

# --- state 10: the mirror of state 8. There an UNVERIFIED PATH
# ce-core had to be refused; here a PATH ce-core that MATCHES the pin
# is accepted and bound, so the installer's core is reused too.
# Airgapped, and the data dir holds no core, so only the PATH one can
# answer — `ce doctor` names the core it actually reached.
mkdir -p "$work/pathcore10" "$work/data10"
cp "$CE_BIN" "$work/pathcore10/ce-core$ext"
core10=$(sha_of "$work/pathcore10/ce-core$ext")
cp "$CE_BIN" "$work/data10/$payload"
cat >"$work/manifest10.env" <<EOF
CE_MANIFEST_VERSION="0.0.0-test"
CE_BASE_URL="file:///nonexistent"
CE_SHA256_${envkey}_CE="$pin"
CE_SHA256_${envkey}_CECORE="$core10"
EOF
out=$(PATH="$work/pathcore10:$PATH" CE_MANIFEST_FILE="$work/manifest10.env" \
      CE_AIRGAPPED=1 CLAUDE_PLUGIN_DATA="$work/data10" \
      sh "$STARTER" doctor . 2>&1) || true
# Read exactly like state 8, inverted: when the bind target is the
# absent data-dir core, doctor says "cannot start <data>/ce-core".
# Its ABSENCE is the proof the PATH core was bound instead — the
# stand-in core is ce itself, so a successful spawn prints ce's own
# output and never names a path.
case "$out" in
    *"data10/ce-core"*) fail 10 "pin-identical PATH ce-core was not reused: $out" ;;
esac
[ ! -f "$work/data10/ce-core$ext" ] || fail 10 "a core was placed despite a verified PATH one"

echo "bootstrap_e2e: PASS (11 states, key=$key)"
