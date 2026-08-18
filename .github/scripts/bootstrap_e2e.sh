#!/bin/sh
# Bootstrap chain e2e (M7-P1 acceptance): drives plugin/bin/ce.sh
# through its four states with the just-built REAL ce binary as the
# payload and file:// as the transport (hermetic — the https leg is
# curl's contract, not ours; stated in the CI step name). States:
#   0 regression : empty pins -> PATH ce answers (air-gapped stance)
#   1 fresh      : pin + source -> download, verify, place, exec
#   2 warm       : source deleted -> cached verified copy answers
#   3 tampered   : wrong pin -> exit 1, loud refusal, nothing placed
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
[ ! -f "$work/data3/$payload.download" ] || fail 3 "tmp download not cleaned"

echo "bootstrap_e2e: PASS (4 states, key=$key)"
