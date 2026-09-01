"use strict";
// A captured transcript rendered as a terminal-style SVG, assembled as
// text with no dependency. A screenshot is a picture of a claim; this is
// generated from the transcript run.js just captured, so re-running the
// demo re-renders the image and the replay test fails when the committed
// image no longer matches what the hooks and gates produce.

const FONT = "ui-monospace, SFMono-Regular, Menlo, Consolas, 'DejaVu Sans Mono', monospace";
const FONT_SIZE = 13;
const LINE_H = 19;
const CHAR_W = 7.82;
const PAD_X = 18;
const PAD_TOP = 46;
const PAD_BOT = 18;
const COLS = 104;

const COLOR = {
  bg: "#0d1117",
  chrome: "#161b22",
  title: "#8b949e",
  cmd: "#58a6ff",
  agent: "#d2a8ff",
  deny: "#ff7b72",
  block: "#ff7b72",
  allow: "#3fb950",
  out: "#c9d1d9",
  red: "#ff7b72",
  green: "#3fb950",
  note: "#8b949e",
};

function esc(text) {
  return text.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

/** Glyphs drawn two columns wide — CJK and the fullwidth forms. The
 *  canvas is fixed at COLS * CHAR_W, so a line measured in characters
 *  would run past the right edge with nothing to notice: a Chinese
 *  sentence is half the characters and more than half the width. */
const WIDE = /[\u1100-\u115f\u2e80-\ua4cf\uac00-\ud7a3\uf900-\ufaff\ufe30-\ufe6f\uff00-\uff60\uffe0-\uffe6]/;

/** Display columns of a string. */
function cols(text) {
  let n = 0;
  for (const ch of text) n += WIDE.test(ch) ? 2 : 1;
  return n;
}

/** How many characters of `text` fit in `width` columns. */
function fit(text, width) {
  let taken = 0;
  let used = 0;
  for (const ch of text) {
    const w = WIDE.test(ch) ? 2 : 1;
    if (used + w > width) break;
    used += w;
    taken += ch.length;
  }
  return taken;
}

/** Hard-wrap one transcript line at COLS, continuation lines indented.
 *  An all-ASCII line wraps exactly where it always did: there `fit`
 *  answers the width itself, and the rest of the rule is untouched. */
function wrap(text) {
  const out = [];
  let rest = text;
  let first = true;
  while (cols(rest) > (first ? COLS : COLS - 4)) {
    const width = first ? COLS : COLS - 4;
    let cut = fit(rest, width);
    const space = rest.lastIndexOf(" ", cut);
    if (space >= cut / 2) cut = space;
    out.push((first ? "" : "    ") + rest.slice(0, cut));
    rest = rest.slice(cut).replace(/^ /, "");
    first = false;
  }
  out.push((first ? "" : "    ") + rest);
  return out;
}

function prefix(kind) {
  switch (kind) {
    case "cmd":
      return "$ ";
    case "agent":
      return "agent> ";
    case "deny":
    case "block":
      return "✗ ";
    case "allow":
      return "✓ ";
    default:
      return "";
  }
}

/** The inverse of prefix(): a console line read back as the typed
 *  row that produced it, so a scene captured as text can be drawn.
 *  Ordered so `deny` claims the mark it shares with `block` — the two
 *  paint the same colour, and only the prefix is being recovered. */
function typed(line) {
  for (const kind of ["cmd", "agent", "deny", "allow"]) {
    const mark = prefix(kind);
    if (line.startsWith(mark)) return { kind, text: line.slice(mark.length) };
  }
  return { kind: "out", text: line };
}

/** Every transcript line as one or more <text> rows. */
function rows(lines) {
  const out = [];
  for (const line of lines) {
    const color = COLOR[line.kind] || COLOR.out;
    for (const piece of wrap(prefix(line.kind) + line.text)) {
      out.push({ color, text: piece });
    }
  }
  return out;
}

/** The SVG document for one transcript. */
function renderSvg(title, lines) {
  const body = rows(lines);
  const width = Math.round(PAD_X * 2 + COLS * CHAR_W);
  const height = PAD_TOP + body.length * LINE_H + PAD_BOT;
  const text = body
    .map((row, i) => {
      const y = PAD_TOP + (i + 1) * LINE_H - 5;
      return `<text x="${PAD_X}" y="${y}" fill="${row.color}" xml:space="preserve">${esc(row.text)}</text>`;
    })
    .join("\n");
  return `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}" viewBox="0 0 ${width} ${height}" font-family="${FONT}" font-size="${FONT_SIZE}">
<title>${esc(title)}</title>
<rect width="${width}" height="${height}" rx="8" fill="${COLOR.bg}"/>
<rect width="${width}" height="32" rx="8" fill="${COLOR.chrome}"/>
<rect y="16" width="${width}" height="16" fill="${COLOR.chrome}"/>
<circle cx="18" cy="16" r="5" fill="#ff5f57"/><circle cx="36" cy="16" r="5" fill="#febc2e"/><circle cx="54" cy="16" r="5" fill="#28c840"/>
<text x="${width / 2}" y="21" fill="${COLOR.title}" text-anchor="middle">${esc(title)}</text>
${text}
</svg>
`;
}

module.exports = { renderSvg, typed, wrap };
