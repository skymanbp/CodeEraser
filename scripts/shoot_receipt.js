// The receipt scripts/shoot_gui.js leaves beside the pictures, read by
// cli/tests/it/site_shots_receipt.rs: which report schemas these
// pictures show, which bytes were actually shot, and which gui/ui tree
// rendered them. Ancestry alone would not have caught the bug that
// started this — the candidates screen showed `ce.join-report/0.1.0`
// through two schema bumps without `gui/ui` changing once. The picture
// digests tie the receipt to the pixels; without them a schema bump
// could be answered by editing three strings here while the old picture
// stayed on the page.
//
// The `ui` digest exists because the freshness leg reads COMMITS: an
// edit to gui/ui that is not committed yet is invisible to it, so a
// local run stayed green while the CI run on the commit went red — twice
// (v2.29 steps 6 and 8). A digest of the tree itself answers on either
// side of the commit. CRLF folds to LF before hashing: the Windows
// checkout carries CRLF, the Linux one LF, and the digest must name the
// same tree on both.
"use strict";
const crypto = require("crypto");
const fs = require("fs");
const path = require("path");

/** Every file under `dir`, recursively, as repo-style relative paths in sorted order. */
function filesUnder(dir) {
  const out = [];
  const walk = (at) => {
    for (const entry of fs.readdirSync(at, { withFileTypes: true })) {
      const full = path.join(at, entry.name);
      if (entry.isDirectory()) walk(full);
      else out.push(path.relative(dir, full).split(path.sep).join("/"));
    }
  };
  walk(dir);
  return out.sort();
}

/** sha256 over `<rel>\0<bytes with CRLF folded to LF>\0` for every file under `dir`. */
function uiDigest(dir) {
  const hash = crypto.createHash("sha256");
  for (const rel of filesUnder(dir)) {
    hash.update(rel + "\0");
    hash.update(fs.readFileSync(path.join(dir, rel)).toString("latin1").replace(/\r\n/g, "\n"), "latin1");
    hash.update("\0");
  }
  return hash.digest("hex");
}

/**
 * Write the receipt — only when `out` is the directory the site serves;
 * a shoot aimed elsewhere (a preview, a diff) must not claim to be the
 * site's pictures. `names` are the shot basenames without extension.
 */
function writeReceipt({ repo, ui, site, names, view, docs, out }) {
  const file = path.join(repo, "contracts", "gui-shots.json");
  if (out !== site) {
    console.log(`  receipt untouched — ${path.relative(repo, out)} is not what the site serves`);
    return;
  }
  const shots = {};
  for (const name of names) {
    const png = fs.readFileSync(path.join(out, `${name}.png`));
    shots[`${name}.png`] = crypto.createHash("sha256").update(png).digest("hex");
  }
  const body = {
    note: "Written by scripts/shoot_gui.js; gated by cli/tests/it/site_screenshots.rs and site_shots_receipt.rs.",
    window: [view.width, view.height],
    shots,
    ui: uiDigest(ui),
    schemas: {
      structure: docs.structure.schema,
      join: docs.join.schema,
      dedup: docs.dedup.schema,
    },
  };
  fs.writeFileSync(file, JSON.stringify(body, null, 2) + "\n");
  console.log(`  ${path.relative(repo, file)}`);
}

module.exports = { uiDigest, writeReceipt };
