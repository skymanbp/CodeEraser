#!/bin/sh
# CodeEraser plugin starter (ADR-007 unique distribution path). The
# plugin repo ships only this script; the real `ce` binary resolves in
# order: (1) a copy in CLAUDE_PLUGIN_DATA whose SHA256 matches the pin
# in bin/manifest.env, (2) a fresh download verified against that pin
# and placed atomically, (3) `ce` on PATH (air-gapped preview stance —
# also the only path while pins are empty). A pin mismatch REFUSES the
# artifact loudly and never executes or keeps it; it does not silently
# fall through to PATH, so a tampered download cannot hide behind a
# locally installed binary. Hooks stay fail-open (R3): "no binary
# anywhere" reports one line and exits 0 instead of blocking a session.
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
        *) curl -fsSL -o "$2" "$1" ;;
    esac
}

run_path_ce() {
    if command -v ce >/dev/null 2>&1; then
        exec ce "$@"
    fi
    echo "codeeraser: no ce binary (no verified copy, no PATH ce) — install with 'cargo install --path cli' or place a pinned binary in CLAUDE_PLUGIN_DATA"
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

# No pin or no data dir: the download path is not configured — PATH ce.
if [ -z "$pin" ] || [ -z "$data" ]; then
    run_path_ce "$@"
fi

target="$data/ce-$CE_MANIFEST_VERSION-$key$ext"
if [ -x "$target" ] && [ "$(sha_of "$target")" = "$pin" ]; then
    exec "$target" "$@"
fi

if [ -n "${CE_AIRGAPPED:-}" ]; then
    # Manual-placement mode: never download; a stale/absent copy just
    # falls back to PATH (the placement instructions live in README).
    run_path_ce "$@"
fi

mkdir -p "$data"
url="${CE_BOOTSTRAP_BASE_URL:-$CE_BASE_URL}/ce-$CE_MANIFEST_VERSION-$key$ext"
tmp="$target.download"
if ! fetch "$url" "$tmp"; then
    rm -f "$tmp"
    echo "codeeraser: download failed ($url) — falling back to PATH ce"
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
exec "$target" "$@"
