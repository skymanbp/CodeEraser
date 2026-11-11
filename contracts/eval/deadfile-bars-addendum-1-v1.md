# Addendum 1 to BARS.md — written 2026-11-11 07:12 local, AFTER answers began to arrive

BARS.md (sha256 2934ab1c7f33f3373065a8fbc37b349a35b816481a882a4e94b7c3145a29fa65) is not edited. This file states
what the main session knew when it wrote the addendum, and adds readings; it relaxes no bar.

## What was known at this moment

17 of 21 first-reading answer files existed. The main session had seen: every auditor's three-line final message
(counts per packet and the points each found unclear in the brief), the format check's output, and the evidence
text of seven rows printed to debug the format check's citation pattern. No table by class existed; the K1 class
of no row was known to anyone.

## A. The population of BARS §4 after a convention passes §3 (a gap in BARS, found from one evidence line)

One answer's evidence showed a row (`zod packages/docs/components/tabs.tsx`) that another file imports: the
importer is a Next.js `page.tsx`, itself dead only because nothing names the framework's entry file. So the
universe holds rows that are unreferenced-by-a-live-file, not unreferenced. BARS §3 removes only the convention's
own rows from §4's population; the product, once such a name is an entry, would also stop reporting everything
that entry reaches.

Both readings are computed and printed:
- reading A (as registered): population = audited rows minus the rows carrying a passing convention;
- reading B: population = audited rows the product still reports dead when every passing convention is declared
  an entry — computed by the product itself (`[graph] entry_globs` in a scratch `ce.toml` of the corpus clone
  copy, `ce deadcode`), never by hand.

Decision: a §4 outcome ships only when readings A and B give the same outcome for that class. If they differ,
nothing ships for that class from this audit and the numbers go to the owner.

## B. A second, mechanical reading of `in_use` (a reading, not a bar)

Several auditors reported that the brief does not say whether these count as use, and counted them `in_use`:
a generic runner that takes the file name on the command line (15 zod bench rows in one packet), a catch-all
install or copy rule (a Makefile wildcard, a Docker `COPY *.r`), a packaging script nothing invokes, a generator
named only in the header of its output, a notebook the README calls broken, a deprecated shim kept for code
outside the repository, an instruction in a code comment.

For every `in_use` answer the review table also carries a mechanical tag, computed from the cited evidence only:
- `named` — some cited `path:line` exists at the pinned tip and that line, or one of the two lines on either
  side, contains the row's basename or its stem as a word;
- `generic` — every existing citation fails that test (a glob, a directory rule, a generic runner), or the
  answer names a convention with no citation.

Every share of BARS §4 is printed twice: with all `in_use` answers, and with only `named` ones (`generic`
counted as not in use). The bars of §4 are evaluated on the first, as registered. If a class is ADVISORY on the
first and would not be on the second, that is stated next to the outcome and reported to the owner with it.

## C. Second reading — unchanged rule, one clarification of procedure

Rows: the fixed 93-row subset, every `unsure`, every `unused` carrying a §3 candidate convention (first seen:
`jsoup src/test/resources/htmltests/gzip.html`, under Maven's test-resources directory, answered `unused`).
Second readers get the same brief and the same hard rules, are different agents, and write to `answers2/`.
Because the brief's open points above are where two readers are most likely to differ, the agreement figure is
printed overall and separately for the rows whose first answer is tagged `generic`.
