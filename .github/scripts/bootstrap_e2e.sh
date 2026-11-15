#!/bin/sh
# Bootstrap chain e2e (M7-P1 acceptance + the v0.2.0 core legs):
# drives plugin/bin/ce.sh through its twenty-eight states with the just-built
# REAL ce binary as the payload and file:// as the transport
# (hermetic — the https leg is curl's contract, not ours; stated in
# the CI step name). States:
#   0 regression : empty pins -> PATH ce answers (air-gapped stance)
#   1 fresh      : pin + source -> download, verify, place, exec
#   2 warm       : source deleted -> cached verified copy answers
#   3 tampered   : wrong pin -> exit 1, loud refusal, nothing placed
#   4 core along : pinned ce-core placed in data under its versioned
#                  asset name `ce-core-<ver>-<key>`
#   5 bad core   : tampered ce-core refused as loudly as a tampered ce
#   6 quiet fail : a failed download falls back to PATH and keeps its
#                  notice OFF stdout (the hook stream is JSON only)
#   7 stale core : a mismatching core at this version's own name is
#                  refused and removed
#   8 core bind  : with a pin in force ce is BOUND to the pinned core
#                  path, never free to fall through to a PATH ce-core
#   9 path reuse : a PATH ce that MATCHES the pin answers with nothing
#                  downloaded and no unverified notice (v0.7.3)
#  10 core reuse : state 8's mirror — a PATH ce-core that MATCHES the
#                  pin is bound instead of fetched
#  11 stamp fast : a verified leg leaves a bound stamp, and the next
#                  call answers through it with NO hashing — a poisoned
#                  sha256sum/shasum on PATH is never heard (O52: the
#                  per-hook chain cost 2 s on Windows)
#  12 stamp stale: a binary newer than the stamp re-runs the full
#                  chain — the poison is heard (R3 degrades to PATH
#                  ce, never the bound copy), the real hasher re-stamps
#  13 health     : SessionStart's `health` always verifies, stamp or
#                  not (once per session)
#  14 no stamp   : the unverified PATH leg never stamps
#  15 hand match : a copy placed by hand at the pinned path that MATCHES
#                  the pin is the verified leg — exec'd silently, stamped
#  16 hand reject: a hand-placed copy that does not match is refused BY
#                  NAME, never exec'd or stamped; PATH ce answers instead
#  17 core works : state 10 with the REAL ce-core as the PATH core: a
#                  measuring command through the starter answers, and
#                  the stamp names the core file that exists (1.9.2 D1:
#                  Git Bash's `command -v` drops `.exe`; 1.9.0-1.9.1
#                  bound that string and every hook went inert)
#  18 borrowed   : CLAUDE_PLUGIN_DATA names another plugin's folder and
#                  this plugin's own folder beside it holds a verified
#                  copy: that copy answers, the other folder is untouched
#  19 foreign    : another plugin's folder alone: nothing is written into
#                  it, the PATH leg answers as with no data folder
#  20 own suffix : this plugin's folder under another marketplace suffix
#                  is its own: the download lands in it, as state 1
#  21 native path: state 18 with the value spelled as a native Windows
#                  path (backslashes): the folder of ours is still found
#  22 handed on : with another plugin's folder and one of ours beside it,
#                  the ce the starter execs is given our folder
#  23 handed none: with another plugin's folder alone, the ce the starter
#                  execs sees the variable unset
#  24 empty leaf : another plugin's folder spelled with two trailing
#                  separators names no folder: a folder of ours beside it
#                  is not picked, a stamp planted in it is not followed,
#                  and the exec'd ce sees the variable unset
#  25 coexist    : two plugin versions share one data folder: Y places its
#                  core at its own name, X's core and stamp stay as they
#                  were, and X re-verifies with no transport at all
#  26 plain core : the unversioned `ce-core` of a starter up to 1.10.0 that
#                  does not match this pin is left in place, and ce is
#                  bound to the versioned path, never to it
#  27 sweep      : a `health` run removes another version's stamp and pair
#                  untouched for 14 days, keeps a version with a fresh
#                  stamp, and removes the plain core once no stamp binds
#                  it; a non-`health` command sweeps nothing, an
#                  air-gapped `health` run removes nothing, and with no
#                  core pin the plain core stays
# Every other state's data folder is `codeeraser-<n>`: the starter uses a
# folder only when its name is the plugin's or begins with it and `-`.
# Usage: bootstrap_e2e.sh <path-to-built-ce> <path-to-ce.sh> <path-to-built-ce-core>
set -eu
# This suite's whole subject is the PIN resolution chain; an ambient
# CE_CORE_BIN outranks the pin inside ce and silently rebinds state 8's
# core (a dev shell exporting it made the suite red only there).
unset CE_CORE_BIN

CE_BIN=$1
[ -f "$CE_BIN" ] || CE_BIN="$1.exe"
STARTER=$2
CORE_BIN=${3:?bootstrap_e2e: the third argument names the built ce-core (state 17)}
[ -f "$CORE_BIN" ] || CORE_BIN="$3.exe"
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

# Independent recomputation of ce.sh's platform key (lockstep on
# purpose: if the product's key grammar drifts, state 1 goes red) —
# the five-target roster of update::version::TARGETS (O70).
case "$(uname -s)" in
    MINGW*|MSYS*|CYGWIN*) key="x86_64-windows"; ext=".exe" ;;
    Darwin) case "$(uname -m)" in
                arm64) key="aarch64-macos" ;;
                *) key="x86_64-macos" ;;
            esac; ext="" ;;
    *) case "$(uname -m)" in
           x86_64) key="x86_64-linux" ;;
           aarch64) key="aarch64-linux" ;;
           *) key="unsupported" ;;
       esac; ext="" ;;
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
      CLAUDE_PLUGIN_DATA="$work/codeeraser-0" sh "$STARTER" --version)
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
# a partial a killed run left two days ago is swept by the download;
# a live sibling's fresh one is not (v2.35 G9, P-A6)
mkdir -p "$work/codeeraser-1"
: > "$work/codeeraser-1/$payload.download.99999"
touch -t 202001010000 "$work/codeeraser-1/$payload.download.99999"
: > "$work/codeeraser-1/$payload.download.88888"
out=$(CE_MANIFEST_FILE="$work/manifest1.env" \
      CE_BOOTSTRAP_BASE_URL="file://$work/src" \
      CLAUDE_PLUGIN_DATA="$work/codeeraser-1" sh "$STARTER" --version)
[ "$out" = "$want" ] || fail 1 "downloaded binary answered '$out' not '$want'"
[ -f "$work/codeeraser-1/$payload" ] || fail 1 "verified binary not placed"
[ ! -f "$work/codeeraser-1/$payload.download.99999" ] || fail 1 "a day-old partial was not swept"
[ -f "$work/codeeraser-1/$payload.download.88888" ] || fail 1 "a fresh partial was swept"
rm -f "$work/codeeraser-1/$payload.download.88888"

# --- state 2: warm — source gone, cached copy answers ----------------
rm "$work/src/$payload"
out=$(CE_MANIFEST_FILE="$work/manifest1.env" \
      CE_BOOTSTRAP_BASE_URL="file://$work/src" \
      CLAUDE_PLUGIN_DATA="$work/codeeraser-1" sh "$STARTER" --version)
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
      CLAUDE_PLUGIN_DATA="$work/codeeraser-3" sh "$STARTER" --version 2>&1 >/dev/null) || rc=$?
[ "$rc" -ne 0 ] || fail 3 "tampered download did not refuse (rc=0)"
case "$err" in *"SHA256 mismatch"*) ;; *) fail 3 "refusal not loud: $err" ;; esac
[ ! -f "$work/codeeraser-3/$payload" ] || fail 3 "tampered binary was placed"
# any .download* leftover, not just the un-suffixed name: the temp
# carries a PID so two sessions cannot curl into one path
left=$(ls "$work/codeeraser-3" 2>/dev/null | grep -c '\.download' || true)
[ "$left" = "0" ] || fail 3 "tmp download not cleaned ($left left)"

# --- state 4: core rides along — pinned ce-core lands in the data dir
# under the release asset's name, one per version (ce is bound to it by
# CE_CORE_BIN); the payload is ce itself standing in for ce-core (any
# executable works, only placement + verification are under test) ------
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
      CLAUDE_PLUGIN_DATA="$work/codeeraser-4" sh "$STARTER" --version)
[ "$out" = "$want" ] || fail 4 "ce with core leg answered '$out' not '$want'"
[ -f "$work/codeeraser-4/$corepayload" ] || fail 4 "verified ce-core not placed at $corepayload"
[ "$(sha_of "$work/codeeraser-4/$corepayload")" = "$corepin" ] || fail 4 "placed ce-core sha drifted"
[ ! -e "$work/codeeraser-4/ce-core$ext" ] || fail 4 "a plain ce-core was placed"

# --- state 5: a tampered core DOWNLOAD refuses the artifact loudly and
# then DEGRADES (audit #419): exit 1 here used to take every hook down
# for a component no hook consults, while the on-disk branch already
# refused-and-continued. Still loud, nothing placed, ce still answers -
# CE_CORE_BIN stays bound to the absent verified path, so judgment
# families report a handshake miss instead of an unverified PATH core.
cat >"$work/manifest5.env" <<EOF
CE_MANIFEST_VERSION="0.0.0-test"
CE_BASE_URL="file:///nonexistent"
CE_SHA256_${envkey}_CE="$pin"
CE_SHA256_${envkey}_CECORE="1111111111111111111111111111111111111111111111111111111111111111"
EOF
rc=0
out=$(CE_MANIFEST_FILE="$work/manifest5.env" \
      CE_BOOTSTRAP_BASE_URL="file://$work/src" \
      CLAUDE_PLUGIN_DATA="$work/codeeraser-5" sh "$STARTER" --version 2>"$work/err5") || rc=$?
err=$(cat "$work/err5")
[ "$rc" -eq 0 ] || fail 5 "the guard must survive a bad core download (rc=$rc)"
[ "$out" = "$want" ] || fail 5 "ce did not run degraded: $out"
case "$err" in *"REFUSING downloaded ce-core"*) ;; *) fail 5 "core refusal not loud: $err" ;; esac
[ ! -f "$work/codeeraser-5/$corepayload" ] || fail 5 "tampered ce-core was placed"

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
      CLAUDE_PLUGIN_DATA="$work/codeeraser-6" sh "$STARTER" --version 2>/dev/null)
[ "$out" = "$want" ] || fail 6 "stdout carried more than the payload's own output: '$out'"

# --- state 7: a mismatching core at this version's own name is refused
# and removed — the download leg always refused a bad core, but a bad
# one already placed survived every early return and stayed exactly
# where the next run would bind it ------------------------------------
mkdir -p "$work/codeeraser-7"
cp "$CE_BIN" "$work/codeeraser-7/$corepayload"   # content does not match the pin below
rm -f "$work/src/$corepayload"           # and the download leg cannot replace it
cat >"$work/manifest7.env" <<EOF
CE_MANIFEST_VERSION="0.0.0-test"
CE_BASE_URL="file:///nonexistent"
CE_SHA256_${envkey}_CE="$pin"
CE_SHA256_${envkey}_CECORE="1111111111111111111111111111111111111111111111111111111111111111"
EOF
err=$(PATH="$work/pathbin:$PATH" CE_MANIFEST_FILE="$work/manifest7.env" \
      CE_BOOTSTRAP_BASE_URL="file://$work/src" \
      CLAUDE_PLUGIN_DATA="$work/codeeraser-7" sh "$STARTER" --version 2>&1 >/dev/null) || true
case "$err" in *"REFUSING on-disk ce-core"*) ;; *) fail 7 "stale core not refused: $err" ;; esac
[ ! -f "$work/codeeraser-7/$corepayload" ] || fail 7 "mismatching ce-core left in place"

# --- state 8: with a core pin in force, ce is BOUND to the pinned
# path. Removing a bad core (state 7) or failing to fetch one used to
# leave ce's resolver free to walk on to a PATH `ce-core` — unverified,
# the very substitution the header forbids. Airgapped so no download
# can rescue it, and an unverified ce-core planted on PATH as the
# thing that must NOT be picked; `ce doctor` names the core it tried.
mkdir -p "$work/pathcore" "$work/codeeraser-8"
cp "$CE_BIN" "$work/pathcore/ce-core$ext"   # the unverified core to REFUSE
cp "$CE_BIN" "$work/codeeraser-8/$payload"         # a VERIFIED ce, so ensure_core runs
out=$(PATH="$work/pathcore:$PATH" CE_MANIFEST_FILE="$work/manifest7.env" \
      CE_AIRGAPPED=1 CLAUDE_PLUGIN_DATA="$work/codeeraser-8" \
      sh "$STARTER" doctor . 2>&1) || true
case "$out" in
    *"codeeraser-8/ce-core-0.0.0-test-$key"*) ;;
    *) fail 8 "ce resolved a core other than the pinned path: $out" ;;
esac

# --- state 9: a PATH ce whose sha MATCHES the pin is REUSED, never
# re-downloaded (v0.7.3). The installer puts ce on the machine PATH,
# so fetching a byte-identical second copy is pure waste. The proof is
# negative on BOTH sides, because stdout alone cannot tell this leg
# from the unverified fallback: nothing lands in the data dir, and the
# "pin unverified" notice run_path_ce always prints never appears.
mkdir -p "$work/codeeraser-9"
: >"$work/out9"
err=$(PATH="$work/pathbin:$PATH" CE_MANIFEST_FILE="$work/manifest1.env" \
      CE_BOOTSTRAP_BASE_URL="file://$work/gone" \
      CLAUDE_PLUGIN_DATA="$work/codeeraser-9" sh "$STARTER" --version 2>&1 >"$work/out9") || true
out=$(cat "$work/out9")
[ "$out" = "$want" ] || fail 9 "verified PATH ce answered '$out' not '$want'"
[ -z "$err" ] || fail 9 "verified leg was not silent (that is the unverified one): $err"
# the verified leg leaves its bound STAMP there (state 11), which is
# not a copy: only ce-* files would be one
placed=$(ls "$work/codeeraser-9" | grep -vc '^bound-' || true)
[ "$placed" -eq 0 ] || fail 9 "a second copy was fetched despite a pin-identical PATH ce"

# --- state 10: the mirror of state 8. There an UNVERIFIED PATH
# ce-core had to be refused; here a PATH ce-core that MATCHES the pin
# is accepted and bound, so the installer's core is reused too.
# Airgapped, and the data dir holds no core, so only the PATH one can
# answer — `ce doctor` names the core it actually reached.
mkdir -p "$work/pathcore10" "$work/codeeraser-10"
cp "$CE_BIN" "$work/pathcore10/ce-core$ext"
core10=$(sha_of "$work/pathcore10/ce-core$ext")
cp "$CE_BIN" "$work/codeeraser-10/$payload"
cat >"$work/manifest10.env" <<EOF
CE_MANIFEST_VERSION="0.0.0-test"
CE_BASE_URL="file:///nonexistent"
CE_SHA256_${envkey}_CE="$pin"
CE_SHA256_${envkey}_CECORE="$core10"
EOF
out=$(PATH="$work/pathcore10:$PATH" CE_MANIFEST_FILE="$work/manifest10.env" \
      CE_AIRGAPPED=1 CLAUDE_PLUGIN_DATA="$work/codeeraser-10" \
      sh "$STARTER" doctor . 2>&1) || true
# Read exactly like state 8, inverted: when the bind target is the
# absent data-dir core, doctor says "cannot start <data>/ce-core-<ver>-<key>".
# Its ABSENCE is the proof the PATH core was bound instead — the
# stand-in core is ce itself, so a successful spawn prints ce's own
# output and never names a path.
# ...and the doctor must have reached its core section at all: an
# empty or aborted run would pass the absence test vacuously
case "$out" in
    *"codeeraser-10/ce-core"*) fail 10 "pin-identical PATH ce-core was not reused: $out" ;;
    *handshake*) ;;
    *) fail 10 "doctor never reached the core section: $out" ;;
esac
[ ! -f "$work/codeeraser-10/$corepayload" ] || fail 10 "a core was placed despite a verified PATH one"

# --- state 11: the bound stamp's fast path hashes NOTHING. State 1's
# verified download left `bound-<version>.env` in codeeraser-1; with a
# poisoned hasher first on PATH (it screams on stderr and answers a
# bogus hash) the fast path must still exec the bound copy silently —
# the slow path would have refused on the bogus hash --------------
mkdir -p "$work/poison"
for h in sha256sum shasum; do
    printf '#!/bin/sh\necho POISONED_HASHER >&2\necho "ffff  $1"\n' >"$work/poison/$h"
    chmod +x "$work/poison/$h"
done
[ -f "$work/codeeraser-1/bound-0.0.0-test.env" ] || fail 11 "verified leg left no stamp"
: >"$work/out11"
err=$(PATH="$work/poison:$PATH" CE_MANIFEST_FILE="$work/manifest1.env" \
      CE_BOOTSTRAP_BASE_URL="file://$work/gone" \
      CLAUDE_PLUGIN_DATA="$work/codeeraser-1" sh "$STARTER" --version 2>&1 >"$work/out11") || true
out=$(cat "$work/out11")
[ "$out" = "$want" ] || fail 11 "stamped fast path answered '$out' not '$want'"
case "$err" in *POISONED_HASHER*) fail 11 "the fast path hashed: $err" ;; esac
[ -z "$err" ] || fail 11 "the fast path was not silent: $err"

# --- state 12: a binary NEWER than the stamp is not trusted by it —
# the full chain runs again and HEARS the poisoned hasher through
# its effect: the candidate loop hashes with 2>/dev/null, so the
# bogus hash disqualifies every copy, the download leg finds
# nothing, and R3 degrades to PATH ce with the `pin unverified`
# notice — the bound copy is never exec'd in silence -------------
touch -t 203001010000 "$work/codeeraser-1/$payload"
rc=0
out=$(PATH="$work/poison:$work/pathbin:$PATH" CE_MANIFEST_FILE="$work/manifest1.env" \
      CE_BOOTSTRAP_BASE_URL="file://$work/gone" \
      CLAUDE_PLUGIN_DATA="$work/codeeraser-1" sh "$STARTER" --version 2>"$work/err12") || rc=$?
err=$(cat "$work/err12")
case "$err" in *"pin unverified"*) ;; *) fail 12 "a newer binary was trusted by the stamp: $err" ;; esac
[ "$rc" -eq 0 ] && [ "$out" = "$want" ] || fail 12 "the R3 fallback did not run PATH ce (rc=$rc): '$out'"
# with the real hasher back, the same call re-verifies and REWRITES
# the stamp (a sentinel planted in the old one is gone afterwards)
echo "# sentinel12" >>"$work/codeeraser-1/bound-0.0.0-test.env"
out=$(CE_MANIFEST_FILE="$work/manifest1.env" CE_BOOTSTRAP_BASE_URL="file://$work/gone" \
      CLAUDE_PLUGIN_DATA="$work/codeeraser-1" sh "$STARTER" --version)
[ "$out" = "$want" ] || fail 12 "re-verification answered '$out' not '$want'"
! grep -q sentinel12 "$work/codeeraser-1/bound-0.0.0-test.env" || fail 12 "the stamp was not rewritten"
# and once the binary is no longer newer than the stamp, the fast
# path is live again: the poison goes unheard
touch -t 202001010000 "$work/codeeraser-1/$payload"
out=$(PATH="$work/poison:$work/pathbin:$PATH" CE_MANIFEST_FILE="$work/manifest1.env" \
      CE_BOOTSTRAP_BASE_URL="file://$work/gone" \
      CLAUDE_PLUGIN_DATA="$work/codeeraser-1" sh "$STARTER" --version 2>"$work/err12b") || fail 12 "the fast path exec failed"
err=$(cat "$work/err12b")
[ -z "$err" ] && [ "$out" = "$want" ] || fail 12 "the fast path did not come back: $err '$out'"

# --- state 13: `health` (the SessionStart hook) verifies every
# session regardless of the stamp: the poison is heard (state 12's
# same effect) even though the fast path would have been live -------
rc=0
err=$(PATH="$work/poison:$work/pathbin:$PATH" CE_MANIFEST_FILE="$work/manifest1.env" \
      CE_BOOTSTRAP_BASE_URL="file://$work/gone" \
      CLAUDE_PLUGIN_DATA="$work/codeeraser-1" sh "$STARTER" health --hook 2>&1 >/dev/null) || rc=$?
case "$err" in *"pin unverified"*) ;; *) fail 13 "health took the fast path: $err" ;; esac
[ "$rc" -eq 0 ] || fail 13 "health via the R3 fallback did not run (rc=$rc): $err"

# --- state 14: the unverified PATH leg (state 0, empty pins) never
# stamps — a stamp is a claim of verification -----------------------
for s in "$work"/codeeraser-0/bound-*; do
    [ -e "$s" ] && fail 14 "the unverified leg left a stamp: $s"
done

# --- state 15: a HAND-PLACED copy at the pinned path that MATCHES the
# pin is the verified leg — air-gapped, no source at all: it execs
# silently and stamps (O79; the placement road plugin/README describes)
mkdir -p "$work/codeeraser-15"
cp "$CE_BIN" "$work/codeeraser-15/$payload"
: >"$work/out15"
err=$(CE_MANIFEST_FILE="$work/manifest1.env" CE_AIRGAPPED=1 \
      CE_BOOTSTRAP_BASE_URL="file://$work/gone" \
      CLAUDE_PLUGIN_DATA="$work/codeeraser-15" sh "$STARTER" --version 2>&1 >"$work/out15") || true
out=$(cat "$work/out15")
[ "$out" = "$want" ] || fail 15 "hand-placed verified copy answered '$out' not '$want'"
[ -z "$err" ] || fail 15 "the hand-placed verified leg was not silent: $err"
[ -f "$work/codeeraser-15/bound-0.0.0-test.env" ] || fail 15 "the hand-placed verified leg left no stamp"

# --- state 16: a hand-placed copy that does NOT match the pin is
# refused BY NAME — never exec'd, never stamped — and, air-gapped, the
# starter degrades to PATH ce with its unverified notice. The file is
# left where it is: unlike a stray ce-core (state 7) nothing execs an
# unverified ce out of the data dir, and a hasher answering wrong
# (state 12) must not cost the operator their placed binary ---------
mkdir -p "$work/codeeraser-16"
cp "$CE_BIN" "$work/codeeraser-16/$payload"
cat >"$work/manifest16.env" <<EOF
CE_MANIFEST_VERSION="0.0.0-test"
CE_BASE_URL="file:///nonexistent"
CE_SHA256_${envkey}_CE="0000000000000000000000000000000000000000000000000000000000000000"
EOF
rc=0
out=$(PATH="$work/pathbin:$PATH" CE_MANIFEST_FILE="$work/manifest16.env" CE_AIRGAPPED=1 \
      CLAUDE_PLUGIN_DATA="$work/codeeraser-16" sh "$STARTER" --version 2>"$work/err16") || rc=$?
err=$(cat "$work/err16")
case "$err" in *"REFUSING on-disk ce "*) ;; *) fail 16 "hand-placed mismatch was not refused by name: $err" ;; esac
case "$err" in *"pin unverified"*) ;; *) fail 16 "no unverified notice on the PATH fallback: $err" ;; esac
[ "$rc" -eq 0 ] && [ "$out" = "$want" ] || fail 16 "PATH fallback did not answer (rc=$rc): '$out'"
[ -f "$work/codeeraser-16/$payload" ] || fail 16 "the refused hand-placed copy was deleted"
for s in "$work"/codeeraser-16/bound-*; do
    [ -e "$s" ] && fail 16 "a refused hand-placed copy left a stamp: $s"
done

# --- state 17: state 10 with the REAL core. Every state above stands
# ce in for ce-core and proves the binding by `--version` / doctor,
# which spawn the core — and a spawn finds `<path>` as `<path>.exe` on
# Windows while the package loader did not, so 1.9.0-1.9.1 passed this
# suite with every measuring command refused (1.9.2 D1). Here the PATH
# core is the built ce-core, pin-identical, airgapped; one measuring
# command through the starter must answer, and the stamp must name a
# core file that exists ----------------------------------------------
mkdir -p "$work/pathcore17" "$work/codeeraser-17" "$work/fx17"
cp "$CORE_BIN" "$work/pathcore17/ce-core$ext"
cp "$CE_BIN" "$work/codeeraser-17/$payload"
printf 'def one():\n    return 1\n' >"$work/fx17/one.py"
cat >"$work/manifest17.env" <<EOF
CE_MANIFEST_VERSION="0.0.0-test"
CE_BASE_URL="file:///nonexistent"
CE_SHA256_${envkey}_CE="$pin"
CE_SHA256_${envkey}_CECORE="$(sha_of "$work/pathcore17/ce-core$ext")"
EOF
rc=0
PATH="$work/pathcore17:$PATH" CE_MANIFEST_FILE="$work/manifest17.env" \
    CE_AIRGAPPED=1 CLAUDE_PLUGIN_DATA="$work/codeeraser-17" \
    sh "$STARTER" scan --format json "$work/fx17" >"$work/out17" 2>"$work/err17" || rc=$?
err=$(cat "$work/err17")
[ "$rc" -eq 0 ] || fail 17 "a measuring command through the starter refused (rc=$rc): $err"
case "$err" in *unavailable*) fail 17 "the core was bound but not reachable: $err" ;; esac
grep -q '"files"' "$work/out17" || fail 17 "scan printed no report: $(cat "$work/out17")"
bound=$(sed -n 's/^BOUND_CORE=//p' "$work/codeeraser-17/bound-0.0.0-test.env")
[ -f "$bound" ] || fail 17 "the stamp names a core file that does not exist: '$bound'"

# --- state 18: CLAUDE_PLUGIN_DATA names ANOTHER plugin's folder (a Bash
# tool shell carries what an earlier hook exported); this plugin's own
# folder beside it holds a verified copy. The starter uses its own: the
# copy answers, one stderr line names the swap, and the other folder is
# byte-identical afterwards (nothing read, nothing written) ----------
mkdir -p "$work/pd18/other-plugin" "$work/pd18/codeeraser-mkt"
echo keep >"$work/pd18/other-plugin/theirs.txt"
cp "$CE_BIN" "$work/pd18/codeeraser-mkt/$payload"
before18=$(ls -la "$work/pd18/other-plugin"; sha_of "$work/pd18/other-plugin/theirs.txt")
out=$(CE_MANIFEST_FILE="$work/manifest1.env" CE_AIRGAPPED=1 \
      CLAUDE_PLUGIN_DATA="$work/pd18/other-plugin" sh "$STARTER" --version 2>"$work/err18")
[ "$out" = "$want" ] || fail 18 "the own folder's copy did not answer: '$out' $(cat "$work/err18")"
grep -q "another plugin's folder" "$work/err18" || fail 18 "the swap was not said: $(cat "$work/err18")"
after18=$(ls -la "$work/pd18/other-plugin"; sha_of "$work/pd18/other-plugin/theirs.txt")
[ "$before18" = "$after18" ] || fail 18 "the other plugin's folder changed"
[ -f "$work/pd18/codeeraser-mkt/bound-0.0.0-test.env" ] || fail 18 "the own folder was not stamped"

# --- state 19: another plugin's folder ALONE: no folder of ours beside
# it, so there is no data folder — nothing written into theirs, and the
# PATH copy (it matches the pin) answers as it does with none ---------
mkdir -p "$work/pd19/other-plugin"
out=$(PATH="$work/pathbin:$PATH" CE_MANIFEST_FILE="$work/manifest1.env" CE_AIRGAPPED=1 \
      CLAUDE_PLUGIN_DATA="$work/pd19/other-plugin" sh "$STARTER" --version 2>"$work/err19")
[ "$out" = "$want" ] || fail 19 "the PATH leg did not answer: '$out' $(cat "$work/err19")"
[ -z "$(ls -A "$work/pd19/other-plugin")" ] || fail 19 "something was written into another plugin's folder"

# --- state 20: this plugin's folder under another marketplace suffix is
# its own: a pinned download lands there, as in state 1 --------------
cp "$CE_BIN" "$work/src/$payload"
out=$(CE_MANIFEST_FILE="$work/manifest1.env" CE_BOOTSTRAP_BASE_URL="file://$work/src" \
      CLAUDE_PLUGIN_DATA="$work/codeeraser-other-market" sh "$STARTER" --version)
[ "$out" = "$want" ] || fail 20 "the own-suffixed folder did not serve: '$out'"
[ -f "$work/codeeraser-other-market/$payload" ] || fail 20 "the download did not land in the own folder"

# --- state 21: state 18 with the value in native Windows form
# (`cygpath -w`, backslashes; native paths reach the starter on Windows).
# Pathname expansion splits a pattern on `/` only, so the search beside
# the folder must see the separators as `/`. Runs wherever cygpath
# exists; on MINGW / MSYS / CYGWIN without it the state FAILS (a skip on
# the one platform it is about would prove nothing); elsewhere it says
# it does not apply ---------------------------------------------------
if command -v cygpath >/dev/null 2>&1; then
    mkdir -p "$work/pd21/other-plugin" "$work/pd21/codeeraser-mkt"
    echo keep >"$work/pd21/other-plugin/theirs.txt"
    cp "$CE_BIN" "$work/pd21/codeeraser-mkt/$payload"
    native21=$(cygpath -w "$work/pd21/other-plugin")
    before21=$(ls -la "$work/pd21/other-plugin"; sha_of "$work/pd21/other-plugin/theirs.txt")
    out=$(CE_MANIFEST_FILE="$work/manifest1.env" CE_AIRGAPPED=1 \
          CLAUDE_PLUGIN_DATA="$native21" sh "$STARTER" --version 2>"$work/err21") || true
    [ "$out" = "$want" ] || fail 21 "the own folder's copy did not answer for '$native21': '$out' $(cat "$work/err21")"
    grep -q "another plugin's folder" "$work/err21" || fail 21 "the swap was not said: $(cat "$work/err21")"
    after21=$(ls -la "$work/pd21/other-plugin"; sha_of "$work/pd21/other-plugin/theirs.txt")
    [ "$before21" = "$after21" ] || fail 21 "the other plugin's folder changed"
    [ -f "$work/pd21/codeeraser-mkt/bound-0.0.0-test.env" ] || fail 21 "the own folder was not stamped"
else
    case "$(uname -s)" in
        MINGW* | MSYS* | CYGWIN*) fail 21 "cygpath is missing on $(uname -s): the native-path state cannot run" ;;
        *) echo "bootstrap_e2e: state 21 (native Windows path) does not apply on $(uname -s)" ;;
    esac
fi

# --- states 22 and 23: what the starter execs reads the folder the
# starter decided on, not the raw variable. A stub `ce` on PATH prints
# the CLAUDE_PLUGIN_DATA it was given (empty pins: the PATH leg) ------
mkdir -p "$work/stub"
cat >"$work/stub/ce" <<'EOF'
#!/bin/sh
printf '%s\n' "${CLAUDE_PLUGIN_DATA-(unset)}"
EOF
chmod +x "$work/stub/ce"
# 22: another plugin's folder with exactly one folder of ours beside it
mkdir -p "$work/pd22/other-plugin" "$work/pd22/codeeraser-mkt"
out=$(PATH="$work/stub:$PATH" CE_MANIFEST_FILE="$work/manifest0.env" \
      CLAUDE_PLUGIN_DATA="$work/pd22/other-plugin" sh "$STARTER" --version 2>/dev/null)
[ "$out" = "$work/pd22/codeeraser-mkt" ] || fail 22 "the exec'd ce was handed '$out', not this plugin's own folder"
# 23: another plugin's folder alone: the exec'd ce sees no folder at all
mkdir -p "$work/pd23/other-plugin"
out=$(PATH="$work/stub:$PATH" CE_MANIFEST_FILE="$work/manifest0.env" \
      CLAUDE_PLUGIN_DATA="$work/pd23/other-plugin" sh "$STARTER" --version 2>/dev/null)
[ "$out" = "(unset)" ] || fail 23 "the exec'd ce was handed '$out', not an unset variable"
# 24: two trailing separators leave an empty leaf after the one dropped:
# the value names no folder, as if unset. A folder of ours beside it is
# not picked, and a stamp planted in the named folder (pointing at a
# second stub) is not read: the PATH stub answers, the folder unchanged
mkdir -p "$work/pd24/other-plugin" "$work/pd24/codeeraser-mkt" "$work/stub24"
cat >"$work/stub24/ce" <<'EOF'
#!/bin/sh
echo "stamp followed"
EOF
chmod +x "$work/stub24/ce"
printf 'BOUND_CE=%s\nBOUND_CORE=\n' "$work/stub24/ce" >"$work/pd24/other-plugin/bound-0.0.0-test.env"
before24=$(ls -la "$work/pd24/other-plugin" "$work/pd24/codeeraser-mkt")
out=$(PATH="$work/stub:$PATH" CE_MANIFEST_FILE="$work/manifest0.env" \
      CLAUDE_PLUGIN_DATA="$work/pd24/other-plugin//" sh "$STARTER" --version 2>"$work/err24")
[ "$out" = "(unset)" ] || fail 24 "the exec'd ce was handed '$out', not an unset variable"
if [ -s "$work/err24" ]; then fail 24 "a value naming no folder was spoken of: $(cat "$work/err24")"; fi
after24=$(ls -la "$work/pd24/other-plugin" "$work/pd24/codeeraser-mkt")
[ "$before24" = "$after24" ] || fail 24 "a folder beside or under the value changed"

# --- state 25: two plugin versions open at once share the data folder
# (every update with an older session still running). Version X
# (0.0.0-test, manifest4) places its pair and stamp; a run of version Y
# (0.0.1-test, another core pin) places Y's core at Y's own name and
# leaves X's core and stamp as they were; X then re-verifies with the
# transport gone: its own core answers, nothing is fetched. Until 1.10.0
# both versions wrote one `ce-core`, and each removed the other's -------
mkdir -p "$work/src25" "$work/codeeraser-25"
cp "$CE_BIN" "$work/src25/$payload"
cp "$CE_BIN" "$work/src25/$corepayload"
cp "$CE_BIN" "$work/src25/ce-0.0.1-test-$key$ext"
echo "the core of 0.0.1-test" >"$work/src25/ce-core-0.0.1-test-$key$ext"
corepinY=$(sha_of "$work/src25/ce-core-0.0.1-test-$key$ext")
cat >"$work/manifest25.env" <<EOF
CE_MANIFEST_VERSION="0.0.1-test"
CE_BASE_URL="file:///nonexistent"
CE_SHA256_${envkey}_CE="$pin"
CE_SHA256_${envkey}_CECORE="$corepinY"
EOF
d25="$work/codeeraser-25"
out=$(CE_MANIFEST_FILE="$work/manifest4.env" CE_BOOTSTRAP_BASE_URL="file://$work/src25" \
      CLAUDE_PLUGIN_DATA="$d25" sh "$STARTER" --version)
[ "$out" = "$want" ] && [ -f "$d25/$corepayload" ] && [ -f "$d25/bound-0.0.0-test.env" ] \
    || fail 25 "version X did not place its core and stamp: '$out' $(ls "$d25")"
stampX=$(cat "$d25/bound-0.0.0-test.env")
out=$(CE_MANIFEST_FILE="$work/manifest25.env" CE_BOOTSTRAP_BASE_URL="file://$work/src25" \
      CLAUDE_PLUGIN_DATA="$d25" sh "$STARTER" --version 2>"$work/err25")
[ "$out" = "$want" ] || fail 25 "version Y did not answer: '$out' $(cat "$work/err25")"
[ "$(sha_of "$d25/ce-core-0.0.1-test-$key$ext")" = "$corepinY" ] || fail 25 "Y's core is not at Y's name"
[ "$(sha_of "$d25/$corepayload")" = "$corepin" ] || fail 25 "Y's run changed or removed X's core"
[ "$(cat "$d25/bound-0.0.0-test.env")" = "$stampX" ] || fail 25 "Y's run changed X's stamp"
[ ! -s "$work/err25" ] || fail 25 "Y's run refused something: $(cat "$work/err25")"
# the stamp is removed so X walks the slow path; a stamp comes back only
# when every pinned leg verified, and with no transport only X's own
# core on disk can have verified
rm "$d25/bound-0.0.0-test.env"
out=$(CE_MANIFEST_FILE="$work/manifest4.env" CE_BOOTSTRAP_BASE_URL="file://$work/gone" \
      CLAUDE_PLUGIN_DATA="$d25" sh "$STARTER" --version 2>"$work/err25b")
[ "$out" = "$want" ] || fail 25 "X did not answer again: '$out' $(cat "$work/err25b")"
bound=$(sed -n 's/^BOUND_CORE=//p' "$d25/bound-0.0.0-test.env" 2>/dev/null || true)
[ "$bound" = "$d25/$corepayload" ] || fail 25 "X re-verified to '$bound', not its own core: $(cat "$work/err25b")"

# --- state 26: the plain `ce-core` a starter up to 1.10.0 placed does not
# match this version's pin: it is that version's, so it stays where it is
# (no refusal, no removal), and ce is bound to this version's own path,
# never to it - air-gapped with no versioned core, doctor names the
# versioned path as the core it could not start ----------------------
d26="$work/codeeraser-26"
mkdir -p "$d26"
cp "$CE_BIN" "$d26/$payload"
echo "an older version's core" >"$d26/ce-core$ext"
chmod +x "$d26/ce-core$ext"
plain26=$(sha_of "$d26/ce-core$ext")
out=$(CE_MANIFEST_FILE="$work/manifest7.env" CE_AIRGAPPED=1 CLAUDE_PLUGIN_DATA="$d26" \
      sh "$STARTER" doctor . 2>&1) || true
case "$out" in
    *"codeeraser-26/ce-core-0.0.0-test-$key"*) ;;
    *) fail 26 "ce was not bound to the versioned core path: $out" ;;
esac
case "$out" in *"REFUSING on-disk ce-core"*) fail 26 "the plain core was refused: $out" ;; esac
[ "$(sha_of "$d26/ce-core$ext" 2>/dev/null)" = "$plain26" ] || fail 26 "the plain core was touched"

# --- state 27: the sweep, on `health` only. Another version (0.0.9-old)
# whose stamp and pair nobody touched since 2020 goes; a version with a
# fresh stamp (0.0.8-live) keeps its old pair; the plain core stays while
# a stamp binds it and goes once none does. Two rules hold it back: an
# air-gapped run removes nothing (nothing removed could be fetched back),
# and with no core pin the plain core stays (ce's resolver still reads it).
# The transport cannot serve: the verified pair on disk answers, and a
# sweep run fetches nothing --------------------------------------------
d27="$work/codeeraser-27"
mkdir -p "$d27"
cp "$CE_BIN" "$d27/$payload"
cp "$CE_BIN" "$d27/$corepayload"
for v in 0.0.9-old 0.0.8-live; do
    : >"$d27/ce-$v-$key$ext"
    : >"$d27/ce-core-$v-$key$ext"
    touch -t 202001010000 "$d27/ce-$v-$key$ext" "$d27/ce-core-$v-$key$ext"
done
printf 'BOUND_CE=%s\nBOUND_CORE=%s\n' "$d27/ce-0.0.9-old-$key$ext" "$d27/ce-core-0.0.9-old-$key$ext" \
    >"$d27/bound-0.0.9-old.env"
touch -t 202001010000 "$d27/bound-0.0.9-old.env"
: >"$d27/ce-core$ext"
printf 'BOUND_CE=%s\nBOUND_CORE=%s\n' "$d27/ce-0.0.8-live-$key$ext" "$d27/ce-core$ext" >"$d27/bound-0.0.8-live.env"
left27() { ls "$d27" | tr '\n' ' '; }
stale27="bound-0.0.9-old.env ce-0.0.9-old-$key$ext ce-core-0.0.9-old-$key$ext"
sweep27() { # $1 = manifest, $2 = CE_AIRGAPPED (empty = off), the rest = the command
    m27=$1; a27=$2; shift 2
    CE_MANIFEST_FILE="$m27" CE_AIRGAPPED="$a27" CE_BOOTSTRAP_BASE_URL="file://$work/gone" \
        CLAUDE_PLUGIN_DATA="$d27" sh "$STARTER" "$@" >/dev/null 2>&1 || true
}
sweep27 "$work/manifest4.env" "" --version
for f in $stale27 "ce-core$ext"; do
    [ -e "$d27/$f" ] || fail 27 "a non-health command swept $f: $(left27)"
done
sweep27 "$work/manifest4.env" 1 health --hook
for f in $stale27 "ce-core$ext"; do
    [ -e "$d27/$f" ] || fail 27 "an air-gapped health run removed $f: $(left27)"
done
sweep27 "$work/manifest4.env" "" health --hook
for f in $stale27; do
    [ ! -e "$d27/$f" ] || fail 27 "health kept the stale $f: $(left27)"
done
for f in bound-0.0.8-live.env "ce-0.0.8-live-$key$ext" "ce-core-0.0.8-live-$key$ext" "ce-core$ext" \
    bound-0.0.0-test.env "$payload" "$corepayload"; do
    [ -e "$d27/$f" ] || fail 27 "health removed $f: $(left27)"
done
grep -q "^BOUND_CORE=$d27/$corepayload\$" "$d27/bound-0.0.0-test.env" \
    || fail 27 "the sweep run did not re-verify its own pair: $(cat "$d27/bound-0.0.0-test.env")"
# no stamp binds the plain core now; under a manifest pinning ce and no
# core (manifest1) it stays, under one pinning a core (manifest4) it goes
printf 'BOUND_CE=%s\nBOUND_CORE=\n' "$d27/ce-0.0.8-live-$key$ext" >"$d27/bound-0.0.8-live.env"
sweep27 "$work/manifest1.env" "" health --hook
[ -e "$d27/ce-core$ext" ] || fail 27 "with no core pin the sweep removed the plain core: $(left27)"
sweep27 "$work/manifest4.env" "" health --hook
[ ! -e "$d27/ce-core$ext" ] || fail 27 "the plain core no stamp binds was kept: $(left27)"
[ -e "$d27/ce-0.0.8-live-$key$ext" ] || fail 27 "the live version's ce went with the plain core"

echo "bootstrap_e2e: PASS (28 states, key=$key)"
