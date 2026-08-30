#!/usr/bin/env node
"use strict";
// The demo family's writer (plan v2.21 S5). run.js measures both trees
// and writes demo/out/*; its `--check` compares the three README demo
// blocks but never writes them, so a verdict whose wording moved left
// the READMEs stale until a human pasted the table. This splices
// demo/out/summary.md and summary.zh.md into the `<!-- demo:begin -->`
// … `<!-- demo:end -->` blocks of README.md, README.zh.md and
// demo/README.md — the same EMBEDS run.js checks. A second file rather
// than a flag: run.js sits at its ratchet ceiling, and writing is a
// second job.
//
//   node demo/run.js && node demo/bless.js

const fs = require("fs");
const path = require("path");
const { EMBEDS } = require("./run");

const HERE = __dirname;
const BLOCK = /(<!-- demo:begin -->\n)[\s\S]*?(<!-- demo:end -->)/;

let wrote = 0;
for (const [rel, table] of EMBEDS) {
  const file = path.join(HERE, rel);
  const text = fs.readFileSync(file, "utf8");
  const want = fs.readFileSync(path.join(HERE, "out", table), "utf8");
  if (!BLOCK.test(text)) {
    console.error(`demo: ${rel} has no demo block`);
    process.exit(2);
  }
  const next = text.replace(BLOCK, (_, begin, end) => begin + want + end);
  if (next !== text) {
    fs.writeFileSync(file, next);
    wrote += 1;
  }
}
console.log(`demo: ${wrote} block(s) rewritten`);
