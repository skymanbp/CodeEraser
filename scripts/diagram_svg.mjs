// The viewer's "Download SVG" rules (archify assets/template.html,
// serializeSvg with autoTheme), applied to the static markup of a
// delivered HTML instead of a live DOM — the SVG half of
// scripts/diagram.mjs, which documents the whole pipeline. Only the
// semantic-class, theme-variable and archify-* keyframe rules of the
// page stylesheet are carried; both variable sets are resolved from
// those rules the way getComputedStyle would resolve them on a probe
// element (dark is the default, light swaps in under
// prefers-color-scheme, svg[data-theme] forces either); the runtime
// state serializeSvg strips is stripped; the result must be XML.

// ---- the viewer's serializeSvg rules, applied to the static markup ----

const KEEP = /(^|,)\s*(svg|:root|\[data-theme|\[data-preset|\.c-|\.t-|\.a-|\.m-)/;

// Top-level rules of one stylesheet: plain style rules and @keyframes
// (CSSOM types 1 and 7); every other at-rule is skipped whole.
function rules(css) {
  const text = css.replace(/\/\*[\s\S]*?\*\//g, "");
  const out = [];
  let i = 0;
  while (i < text.length) {
    const open = text.indexOf("{", i);
    if (open < 0) break;
    const prelude = text.slice(i, open).trim();
    const semi = text.indexOf(";", i);
    if (prelude.startsWith("@") && semi >= 0 && semi < open && !prelude.includes("\n")) {
      i = semi + 1; // a statement at-rule (@import, @charset)
      continue;
    }
    const close = balanced(text, open);
    const body = text.slice(open + 1, close);
    if (prelude.startsWith("@")) {
      if (/^@keyframes\s+archify-/.test(prelude)) out.push({ prelude, body, keyframes: true });
    } else {
      out.push({ prelude, body });
    }
    i = close + 1;
  }
  return out;
}

function balanced(text, open) {
  let depth = 0;
  for (let j = open; j < text.length; j += 1) {
    if (text[j] === "{") depth += 1;
    else if (text[j] === "}" && (depth -= 1) === 0) return j;
  }
  throw new Error("unbalanced stylesheet");
}

const selector = (s) => s.replace(/\s+/g, " ").replace(/\s*,\s*/g, ", ").trim();

const declarations = (body) =>
  body
    .split(";")
    .map((d) => d.trim())
    .filter(Boolean)
    .map((d) => d.replace(/\s*:\s*/, ": "));

function cssText(rule) {
  if (rule.keyframes) return `${rule.prelude} {${rule.body.replace(/\s+/g, " ").trim() ? " " + rule.body.replace(/\s+/g, " ").trim() + " " : ""}}`;
  return `${selector(rule.prelude)} { ${declarations(rule.body).join("; ")}; }`;
}

// A compound selector of :root / html / svg / [data-*="…"] parts matched
// against a probe element; anything with a combinator or a class never
// matches (the variable blocks are simple compounds).
function matches(compound, element) {
  const s = compound.trim();
  if (/[\s>+~]/.test(s.replace(/\[[^\]]*\]/g, ""))) return null;
  let specificity = 0;
  for (const part of s.match(/:root|\[[^\]]+\]|\.[\w-]+|[a-z]+|\*/g) ?? []) {
    if (part === ":root") {
      if (!element.root) return null;
      specificity += 10;
    } else if (part.startsWith("[")) {
      const m = /^\[([\w-]+)="?([^"\]]*)"?\]$/.exec(part);
      if (!m || element.attrs[m[1]] !== m[2]) return null;
      specificity += 10;
    } else if (part.startsWith(".")) {
      return null;
    } else if (part !== "*") {
      if (part !== element.tag) return null;
      specificity += 1;
    }
  }
  return specificity;
}

function computedVars(kept, element) {
  const wins = new Map(); // name -> [specificity, order, value]
  kept.forEach((rule, order) => {
    if (rule.keyframes) return;
    const best = Math.max(...rule.prelude.split(",").map((c) => matches(c, element) ?? -1));
    if (best < 0) return;
    for (const d of declarations(rule.body)) {
      const m = /^(--[\w-]+): (.*)$/.exec(d);
      if (!m) continue;
      const prev = wins.get(m[1]);
      if (!prev || best > prev[0] || (best === prev[0] && order >= prev[1])) {
        wins.set(m[1], [best, order, m[2]]);
      }
    }
  });
  return wins;
}

function resolveVars(kept, names, rootAttrs, theme) {
  const inherited = computedVars(kept, { tag: "html", root: true, attrs: rootAttrs });
  const probe = computedVars(kept, {
    tag: "div",
    root: false,
    attrs: { "data-theme": theme, "data-preset": rootAttrs["data-preset"] ?? "classic" },
  });
  return names.map((n) => `${n}: ${(probe.get(n) ?? inherited.get(n) ?? [0, 0, ""])[2]};`).join(" ");
}

// What serializeSvg strips from the clone: exploration/runtime state
// attributes, the detail and legend interaction wiring, and every
// overlay element (a static render must carry none — refuse if it does).
const DROP = /^data-(detail|detail-anchor|view-scale|(focus|reach|lens|story|route|share|chapter|intent-trace|legend-preview|relationship-preview|relationship-direct|relationship-pin|route-journey)(-[\w-]+)?)$/;
const LEGEND = ["data-legend-kind", "data-legend-label", "data-legend-count", "data-legend-zero", "data-legend-selected", "role", "tabindex", "aria-label", "aria-pressed", "aria-haspopup", "aria-controls", "aria-expanded"];
const OVERLAY = /data-([\w-]+-)?overlay|data-legend-bridge-runtime|data-source-evidence-beacon|data-story-carrier-token/;

function clean(svg) {
  if (OVERLAY.test(svg)) throw new Error("delivered svg carries runtime overlays");
  return svg.replace(/<([a-zA-Z][\w:-]*)((?:\s+[^\s=/>]+(?:="[^"]*")?)*)\s*(\/?)>/g, (_, tag, attrs, close) => {
    const list = [...attrs.matchAll(/([^\s=/>]+)(?:="([^"]*)")?/g)].map((m) => [m[1], m[2] ?? ""]);
    const has = (n) => list.some(([k]) => k === n);
    const drop = new Set(list.map(([k]) => k).filter((k) => DROP.test(k)));
    if (has("data-legend-kind")) LEGEND.forEach((k) => drop.add(k));
    if (has("data-legend-bridge")) ["data-legend-bridge", "role", "aria-label"].forEach((k) => drop.add(k));
    if (has("data-source-evidence-count")) ["data-source-evidence-count", "data-source-evidence-original-label"].forEach((k) => drop.add(k));
    const original = list.find(([k]) => k === "data-source-evidence-original-label")?.[1] ?? "";
    const kept = list
      .filter(([k]) => !drop.has(k))
      .map(([k, v]) => {
        if (k === "aria-pressed" && has("data-node-id")) return [k, "false"];
        if (k === "aria-label" && has("data-source-evidence-count")) return original ? [k, original] : null;
        return [k, v];
      })
      .filter(Boolean);
    return `<${tag}${kept.map(([k, v]) => ` ${k}="${v}"`).join("")}${close ? " /" : ""}>`;
  });
}

// The SVG must be XML, not HTML: every attribute valued, tags balanced,
// no bare `&` or `<` in text — GitHub's renderer stops at the first error.
function assertWellFormed(svg) {
  const stack = [];
  const re = /<!--[\s\S]*?-->|<\/([\w:-]+)\s*>|<([\w:-]+)((?:\s+[\w:-]+="[^"<]*")*)\s*(\/?)>|([^<]+)|(<)/g;
  for (const m of svg.matchAll(re)) {
    if (m[1]) {
      if (stack.pop() !== m[1]) throw new Error(`svg: stray </${m[1]}>`);
    } else if (m[2]) {
      if (!m[4]) stack.push(m[2]);
    } else if (m[5]) {
      if (/&(?![#\w]+;)/.test(m[5])) throw new Error("svg: bare & in text");
    } else if (m[6]) {
      throw new Error(`svg: malformed tag at byte ${m.index}`);
    }
  }
  if (stack.length) throw new Error(`svg: unclosed <${stack.join("><")}>`);
}

// Chrome archify writes itself, which no IR key reaches: the legend
// heading is a literal in the viewer template (assets/template.html).
// The zh twin is meant to be one language throughout, so the word is
// mapped here — the only place that sees both the rendered markup and
// the language that asked for it. A term the map does not carry stays
// in English and the zh-chrome leg names it.
const CHROME = { zh: { Legend: "图例" } };

const chrome = (svg, lang) =>
  Object.entries(CHROME[lang] ?? {}).reduce(
    (s, [from, to]) => s.replaceAll(`>${from}</text>`, `>${to}</text>`),
    svg,
  );

export function extractSvg(html, lang = "en") {
  const style = /<style>([\s\S]*?)<\/style>/.exec(html)?.[1];
  const htmlTag = /<html\b([^>]*)>/.exec(html)?.[1] ?? "";
  const start = html.indexOf("<svg", html.indexOf('class="diagram-container"'));
  if (!style || start < 0) throw new Error("delivered HTML lacks a stylesheet or the diagram svg");
  let depth = 0;
  let end = start;
  for (const m of html.slice(start).matchAll(/<svg\b|<\/svg>/g)) {
    depth += m[0] === "<svg" ? 1 : -1;
    if (depth === 0) {
      end = start + m.index + m[0].length;
      break;
    }
  }
  const svg = html.slice(start, end);
  const rootAttrs = Object.fromEntries([...htmlTag.matchAll(/([\w-]+)="([^"]*)"/g)].map((m) => [m[1], m[2]]));
  const built = assemble(clean(svg), rules(style).filter((r) => r.keyframes || KEEP.test(selector(r.prelude))), rootAttrs);
  const out = chrome(built, lang);
  assertWellFormed(out);
  return out;
}

function assemble(svg, kept, rootAttrs) {
  const host = kept.map(cssText).join("\n");
  const names = [...new Set(host.match(/--[a-zA-Z0-9-]+(?=\s*:)/g) ?? [])];
  const dark = resolveVars(kept, names, rootAttrs, "dark");
  const light = resolveVars(kept, names, rootAttrs, "light");
  const fontFallback = [400, 500, 600, 700]
    .map((w) => `@font-face { font-family: 'JetBrains Mono'; font-weight: ${w}; src: local('JetBrains Mono'), local('JetBrainsMono-Regular'); }`)
    .join("\n");
  const css =
    `${fontFallback}\n` +
    "svg { font-family: 'JetBrains Mono', ui-monospace, SFMono-Regular, Menlo, Consolas, 'DejaVu Sans Mono', 'Liberation Mono', 'Noto Sans Mono CJK SC', 'PingFang SC', 'Hiragino Sans GB', 'Microsoft YaHei', monospace; }\n" +
    `${host}\n` +
    `:root, svg { ${dark} }\n` +
    `@media (prefers-color-scheme: light) { :root, svg { ${light} } }\n` +
    `svg[data-theme="light"] { ${light} }\n` +
    `svg[data-theme="dark"] { ${dark} }\n` +
    "rect.c-bg-rect { fill: var(--bg); }\n";
  const tagEnd = svg.indexOf(">") + 1;
  const [, w, h] = /viewBox="[-\d.]+ [-\d.]+ ([\d.]+) ([\d.]+)"/.exec(svg.slice(0, tagEnd)) ?? [];
  if (!w || !h) throw new Error("svg has no viewBox");
  let tag = svg.slice(0, tagEnd).replace(/\sdata-theme="[^"]*"/, "");
  tag = tag.replace(/^<svg/, `<svg xmlns="http://www.w3.org/2000/svg" width="${w}" height="${h}"`);
  const rest = svg.slice(tagEnd);
  const afterDesc = rest.indexOf("</desc>") >= 0 ? rest.indexOf("</desc>") + "</desc>".length : rest.indexOf("</title>") + "</title>".length;
  const inject = `\n<style>${css}</style>\n<rect width="100%" height="100%" class="c-bg-rect"/>`;
  return `${tag}${rest.slice(0, afterDesc)}${inject}${rest.slice(afterDesc)}`.replace(/\r\n/g, "\n");
}
