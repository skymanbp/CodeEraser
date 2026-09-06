// The release roster for the JavaScript hosts (pin_release.js,
// packaging.js): update::version::TARGETS spelled once more, with the
// same derivations from a key's `<os>` half — tests/it/release_roster.rs
// holds TARGETS here to the Rust const, so the two never drift apart.
"use strict";

/** `<arch>-<os>` keys, in SHA256SUMS order (plan v2.29 step 10, O70). */
const TARGETS = ["x86_64-windows", "x86_64-linux", "aarch64-macos", "x86_64-macos", "aarch64-linux"];

/** The GUI bundle per os: [asset suffix after the key, manifest key tail]. */
const BUNDLE = { windows: ["-setup.exe", "SETUP"], linux: [".AppImage", "APPIMAGE"], macos: [".dmg", "DMG"] };

/** The three artifacts every target ships, in pin order. */
const WHAT = ["ce", "ce-core", "CodeEraser"];

function os(key) {
  return key.slice(key.lastIndexOf("-") + 1);
}

function arch(key) {
  return key.slice(0, key.lastIndexOf("-"));
}

/** Binary suffix: `.exe` on Windows, nothing elsewhere. */
function ext(key) {
  return os(key) === "windows" ? ".exe" : "";
}

/** Asset name of `what` ("ce" | "ce-core" | "CodeEraser") for one target at `ver`. */
function asset(what, ver, key) {
  if (what === "CodeEraser") return `CodeEraser-${ver}-${key}${BUNDLE[os(key)][0]}`;
  return `${what}-${ver}-${key}${ext(key)}`;
}

/** Manifest key of `what` for one target: CE_SHA256_<PLATFORM>_<TAIL>, [a-z-] uppercased to [A-Z_]. */
function pinKey(what, key) {
  const tail = what === "ce" ? "CE" : what === "ce-core" ? "CECORE" : BUNDLE[os(key)][1];
  return `CE_SHA256_${key.toUpperCase().replace(/-/g, "_")}_${tail}`;
}

/** plugin/bin/manifest.env as sh reads it: KEY="value" lines, comments and blanks dropped. */
function parseManifest(text) {
  const out = {};
  for (const line of text.split("\n")) {
    const l = line.trim();
    if (!l || l.startsWith("#")) continue;
    const i = l.indexOf("=");
    if (i < 0) continue;
    out[l.slice(0, i).trim()] = l
      .slice(i + 1)
      .trim()
      .replace(/^"(.*)"$/, "$1");
  }
  return out;
}

module.exports = { TARGETS, BUNDLE, WHAT, os, arch, ext, asset, pinKey, parseManifest };
