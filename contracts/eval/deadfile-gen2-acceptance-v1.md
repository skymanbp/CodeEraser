# The coordinator's acceptance of the generation-2 replay (BARS-GEN2 section 6, item 3)

Written by the main session on 2026-11-13, after the report was printed and before any of its tables was accepted.
The replay: product `5a3f959380b339031f06b7cd01c452b3e1244729` (release pair), instrument
`244a0dd3ce82e3493e2f21c2b118ac866c669ab0`, the ten docs `deadfile-gen2-replay-v1.<corpus>.json` beside this file.

## 1. Before any count

- Tips: nine first choices judged. C's first choice jqlang/jq was refused at its tip (`exit 2: ce deadcode:
  mentions: submodule vendor/oniguruma is not checked out`) and replaced by the pre-registered redis/hiredis (788
  first-parent commits, tip judged), the second replacement reason of section 1.
- The unjudged side, read before the report ran: zero unjudged and zero retried trees in all ten corpora (801 trees
  each; Penlight 713, hiredis 788). Checked on the registered clones, read-only: no commit of any replayed chain
  touches `.gitmodules` (`git log --first-parent <chain base>..<tip> -- .gitmodules` is empty ten times), so the
  zero is not the refusal that removed jq going unseen.
- No warm / cold mismatch was written; both workers ended without a failed corpus.

## 2. The outcome sentences against the printed integers (section 4)

| state | events | re-referred | kept up focused | kept up sweep | removed untouched | undetermined | A | B |
|---|---|---|---|---|---|---|---|---|
| orphaned | 21 | 3 | 7 | 4 | 7 | 0 | 14/21 = 66.7 % | 10/17 = 58.8 % |
| born | 90 | 9 | 53 | 9 | 11 | 8 | 79/90 = 87.8 % | 62/73 = 84.9 % |

Summed by hand from the per-corpus rows: orphaned 4+0+0+6+0+5+3+2+0+1 = 21, born 20+2+0+8+1+27+10+6+2+14 = 90
(Penlight, fd, flask, hiredis, hlint, junit4, purrr, spdlog, viper, zustand), each fate column likewise. A =
(re-referred + kept up + undetermined) / events; B = (re-referred + kept up focused) / (those + removed
untouched). Both match the printed shares.

Condition (1), at least 100 events, fails for both states. The sentence printed for each is the one section 4
gives: fewer than 100 events, not judged, it does not block.

## 3. Twenty fates by hand

The rule, fixed before any event was looked at: order the 111 events by
`sha256("recheck|" + corpus + "|" + commit + "|" + path)`; take the two lowest of every fate word that occurs; fill
to 20 with the lowest not yet taken. For each event the coordinator wrote down, from git alone on the registered
clone (the file at the event's first parent, `git diff -M -l0 --name-status`, the first later first-parent commit
that touches the file, a pure rename followed), its state, fate, deciding commit and the file's name there, and
only then opened the docs' rows.

| n | corpus | commit | path | state | fate | decided at | name there | by hand |
|---|---|---|---|---|---|---|---|---|
| 1 | Penlight | `764acddef0` | `docs/manual/05-dates.md` | born | kept up focused | `ddb5e78461` | `docs/manual/05-dates.md` | same |
| 2 | zustand | `37112e585c` | `docs/persisting-store-data.md` | born | kept up focused | `fa581ddd9b` | `docs/persisting-store-data.md` | same |
| 3 | Penlight | `764acddef0` | `docs/manual/09-discussion.md` | born | kept up focused | `dfaa12c028` | `docs/manual/09-discussion.md` | same |
| 4 | Penlight | `82d0e4e666` | `CHANGES.md` | orphaned | kept up focused | `2f882c2ff0` | `CHANGES.md` | same |
| 5 | hiredis | `e245ab48ec` | `extra/hiredis/libev.h` | born | kept up focused | `c8d72f7c7a` | `extra/hiredis/libev.h` | same |
| 6 | zustand | `8e2445cc2a` | `docs/auto-generating-selectors.md` | born | kept up focused | `15ff6623c0` | `docs/auto-generating-selectors.md` | same |
| 7 | junit4 | `192920c166` | `doc/ReleaseNotes4.5.html` | born | kept up focused | `fd691746a7` | `doc/ReleaseNotes4.5.html` | same |
| 8 | fd | `5b46867507` | `doc/sponsors.md` | born | kept up focused | `29936f0fba` | `doc/sponsors.md` | same |
| 9 | junit4 | `7aac4b19d3` | `doc/ReleaseNotes4.8.html` | born | kept up focused | `5a3a326096` | `doc/ReleaseNotes4.8.html` | same |
| 10 | spdlog | `9ebc4b24d9` | `include/spdlog/fmt/bundled/args.h` | born | kept up focused | `d8199b607d` | `include/spdlog/fmt/bundled/args.h` | same |
| 11 | zustand | `da15804ae8` | `docs/maps-and-sets-usage.md` | born | kept up focused | `2122a0c5e7` | `docs/maps-and-sets-usage.md` | same |
| 12 | junit4 | `2f879b9eea` | `doc/ReleaseNotes4.6.md` | born | kept up focused | `1077e63243` | `doc/ReleaseNotes4.6.md` | same |
| 13 | zustand | `5b938fbbff` | `src/types.d.ts` | born | kept up sweep | `9062ca634f` | `src/types.d.ts` | same |
| 14 | junit4 | `751f75986b` | `doc/ReleaseNotes4.9b1.html` | born | removed untouched | `d6ccf5b0fa` | `doc/ReleaseNotes4.9b1.html` | same |
| 15 | purrr | `e979a33abc` | `docs/reference/when.html` | orphaned | kept up sweep | `011e690be5` | `docs/reference/when.html` | same |
| 16 | Penlight | `c5196ecaeb` | `lua/pl/globchk.lua` | born | removed untouched | `dcab67d666` | `lua/pl/globchk.lua` | same |
| 17 | junit4 | `34a0ecb4ff` | `doc/ReleaseNotes4.8.1.html` | born | undetermined | `-` | `doc/ReleaseNotes4.8.1.html` | same |
| 18 | Penlight | `cecfaee1fe` | `lua/pl/OrderedMap.lua` | born | re-referred | `1c99f854e2` | `lua/pl/OrderedMap.lua` | read in the commit |
| 19 | Penlight | `1ef7baa8fc` | `change-0.9.5.md` | born | re-referred | `9398825c46` | `CHANGES.txt` | read in the commit |
| 20 | junit4 | `4dd14be2eb` | `epl-v10.html` | born | undetermined | `-` | `epl-v10.html` | same |

- 18 of the 18 events git alone can decide agree in all four fields: 12 kept up focused, 2 kept up sweep, 2
  removed untouched, 2 undetermined.
- A re-reference is the product's word (git cannot see deadness), so rows 18 and 19 were read in their deciding
  commits:
  - row 18: commit `1c99f854e2` adds `OrderedMap = require 'pl.OrderedMap'` in another file. A real new reference;
    the same commit also edits the file among more than 20 others, and section 2 lets the first fate in its list
    decide.
  - row 19: commit `9398825c46` is `R100 change-0.9.5.md -> CHANGES.txt`, and no file of that tree names
    `CHANGES.txt`. The file "exists and is not dead" there because `.txt` is outside the judged set, not because a
    reference appeared. By section 2's wording it is re-referred; had it not been, the next commit that edits it
    (`e311315c43`, 2 files) makes it kept up focused. Both words are false under A and under B and stand in B's
    denominator: neither reading moves. Row 4 is the mirror case: `82d0e4e666` is `R097 CHANGES.txt -> CHANGES.md`,
    an orphaned event (the file existed at the parent and was not dead there).
- Disagreements: none. The tables are accepted.

## 4. Noted, not changed

- A row with an empty denominator prints `0/0 (0.0 %, upper 100.0 %)`: the renderer's writing of an undefined
  share. The instrument is frozen from its first printed count; the integers are right.
- What the 20 events are: mostly documents nothing links to (manual pages, release notes, a licence text, a
  sponsors page) and headers a library offers its users.
- The secondary table (the thirteen audited corpora, section 1) was not run: the instrument reads one registry, the
  primary one, and no line of it changes after the first printed count. Section 1 gives that table no say.
