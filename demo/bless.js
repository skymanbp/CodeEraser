#!/usr/bin/env node
"use strict";
// The demo family's writer (plan v2.21 S5). run.js measures the trees
// and writes demo/out/*; its `--check` compares every marked README
// block but never writes one, so a verdict whose wording moved left
// the READMEs stale until a human pasted the new text. This splices
// each artefact into the block that carries it — the same EMBEDS table
// run.js checks, marker column included, so a new family of blocks
// needs no second writer. A second file rather than a flag: run.js
// sits at its ratchet ceiling, and writing is a second job.
//
//   node demo/run.js && node demo/bless.js

const fs = require("fs");
const path = require("path");
const { EMBEDS, blockOf } = require("./run");

const HERE = __dirname;

let wrote = 0;
for (const [rel, artefact, mark] of EMBEDS) {
  const file = path.join(HERE, rel);
  const text = fs.readFileSync(file, "utf8");
  const want = fs.readFileSync(path.join(HERE, "out", artefact), "utf8");
  if (!blockOf(mark).test(text)) {
    console.error(`demo: ${rel} has no ${mark} block`);
    process.exit(2);
  }
  // a function replacer, not a string: the console blocks are full of
  // `$`, and String.replace reads `$&` / `$'` / `$1` out of one
  const next = text.replace(blockOf(mark), () => `<!-- ${mark}:begin -->\n${want}<!-- ${mark}:end -->`);
  if (next !== text) {
    fs.writeFileSync(file, next);
    wrote += 1;
  }
}
console.log(`demo: ${wrote} block(s) rewritten`);
