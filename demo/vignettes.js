"use strict";
// The small scenes the READMEs embed beside the two-column table.
//
// The table answers "does it change the outcome"; these answer "what
// does it look like" — one question per scene, played on its own copy
// of demo/seed, with every line the verbatim output of a `ce`
// subprocess. Each scene is asked once per language and both answers
// come from the SAME tree, so the pair is a translation of one run and
// not two runs that happened to agree.
//
// Two rules this family lives by:
//
//   * Every act declares what it must answer (`expect`), and a scene
//     that answers otherwise throws. Without it the failure mode is
//     silent and total: a probe against a tree whose index never got
//     built degrades to `allow`, and a refusal exhibit becomes a
//     picture of the guard doing nothing, byte-gated in that state.
//   * No prose from the scripted agent is shown. steps.js writes its
//     narration in English only, and a Chinese README carrying an
//     English `agent>` line would be a translation gap dressed as a
//     transcript. The scene's own title carries the context instead.

const fs = require("fs");
const path = require("path");
const { CE, run, seedTree, probe, normalize, ejectTree } = require("./tree");
const { renderSvg, typed } = require("./render");
const { steps } = require("./steps");

const LANGS = ["en", "zh"];

/** The declaration scene 2 adds to the seed's ce.toml. The ladder must
 *  climb (config::Thresholds::ladder_fault), so a class that tightens
 *  the hard line states its warn line too. */
const CLASS = `
[[rules.class]]
name = "invoicer"
globs = ["invoicer/**"]
knobs = { file_lines_warn = 30, file_lines_fail = 40 }
`;

const EXHIBITS = [
  {
    id: "copied-helper",
    title: {
      en: "**A copied helper, refused before the file exists.** Move 1 of the run above, on its own. The reason names the region the content duplicates and the ordering that would pass, so the refusal is actionable rather than a veto.",
      zh: "**抄来的辅助函数，在文件存在之前就被拒。** 上表第 1 步单独拿出来。理由点名这段内容重复了哪块区域、以及怎样排序才能通过——所以这是一条可执行的拒绝，不是一张否决票。",
    },
    seed: (seed) => seed,
    acts: [{ move: 1, expect: "deny" }],
  },
  {
    id: "one-line-two-mouths",
    title: {
      en: "**One line, two mouths.** `ce.toml` puts `invoicer/**` on `file_lines_fail = 40`. The write-time guard refuses the write that would cross it, and `ce scan` grades the same tree against the same number — one declaration, read by the hook and by CI.",
      zh: "**一条线，两张嘴。** `ce.toml` 给 `invoicer/**` 定下 `file_lines_fail = 40`。写入时守卫拒绝会越线的那次写入，`ce scan` 用同一个数给同一棵树评级——一处声明，钩子与 CI 同读。",
    },
    seed: (seed) => ({ ...seed, "ce.toml": seed["ce.toml"] + CLASS }),
    acts: [
      { move: 4, expect: "deny" },
      { cmd: ["scan", "."], expect: 1, tail: 3 },
    ],
  },
];

/** A scene that stops refusing must stop the build, not the story. */
function held(id, act, got) {
  const want = act.expect;
  if (got !== want) throw new Error(`vignette ${id}: expected ${JSON.stringify(want)}, got ${JSON.stringify(got)}`);
}

/** One write, put to the guard: the console lines a reader sees. */
function askGuard(dir, seed, act, lang) {
  const step = steps(seed).find((s) => s.id === act.move);
  const verdict = probe(dir, step.file, step.content, lang);
  return { got: verdict ? verdict.decision : "allow", lines: [`$ Write ${step.file}`, verdict ? `✗ ${normalize(verdict.reason, dir)}` : "✓ landed"] };
}

/** One `ce` family over the tree, clipped to the rows the scene is about. */
function askGate(dir, act, lang) {
  const r = run(CE, act.cmd, dir, undefined, { CE_LANG: lang });
  const shown = normalize(r.out, dir).split("\n").slice(-act.tail);
  return { got: r.rc, lines: [`$ ce ${act.cmd.join(" ")}`, ...shown] };
}

/** One scene, played in one language against an already-built tree. */
function play(dir, seed, exhibit, lang) {
  const lines = [];
  for (const act of exhibit.acts) {
    const { got, lines: said } = act.move === undefined ? askGate(dir, act, lang) : askGuard(dir, seed, act, lang);
    held(exhibit.id, act, got);
    lines.push(...said);
  }
  return lines;
}

/** Title, then the console block — the shape both READMEs embed. */
function render(exhibit, lines, lang) {
  return [exhibit.title[lang], "", "```console", ...lines, "```"].join("\n");
}

/** The scene the READMEs open with. A reader's first glance should
 *  be the product doing its one visible thing, not a diagram of the
 *  machine that does it; this is the same scene as the exhibit below,
 *  drawn instead of quoted, so there is no second capture to drift. */
const HERO = "copied-helper";
const HERO_TITLE = {
  en: "a copied helper, refused before the file exists",
  zh: "抄来的辅助函数，在文件存在之前就被拒",
};

/** Every scene, built once and played in every language: the artefacts
 *  demo/out gains, keyed the way run.js's EMBEDS names them. */
function vignetteFiles(seed, work) {
  const played = EXHIBITS.map((exhibit) => {
    const dir = path.join(work, `vignette-${exhibit.id}`);
    fs.mkdirSync(dir, { recursive: true });
    seedTree(dir, exhibit.seed(seed));
    const said = Object.fromEntries(LANGS.map((lang) => [lang, play(dir, seed, exhibit, lang)]));
    ejectTree(dir);
    return { exhibit, said };
  });
  const files = {};
  const hero = played.find(({ exhibit }) => exhibit.id === HERO);
  if (!hero) throw new Error(`vignettes: the hero scene ${HERO} is gone`);
  for (const lang of LANGS) {
    const suffix = lang === "en" ? "" : `.${lang}`;
    files[`vignettes${suffix}.md`] = played.map(({ exhibit, said }) => render(exhibit, said[lang], lang)).join("\n\n") + "\n";
    files[`hero${suffix}.svg`] = renderSvg(HERO_TITLE[lang], hero.said[lang].map(typed));
  }
  return files;
}

module.exports = { vignetteFiles, EXHIBITS };
