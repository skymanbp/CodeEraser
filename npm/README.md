# CodeEraser

**This npm package is a name-reservation pointer, not a distribution.**
CodeEraser is a Rust + Haskell tool; nothing here installs it.

Get the real thing:

- **Prebuilt binaries & GUI installers** — <https://github.com/skymanbp/CodeEraser/releases>
  (verify against `SHA256SUMS`)
- **Cargo** — `cargo install codeeraser` (binary name: `ce`); every
  measuring command also needs the `ce-core` judgment core beside it
  (shipped in the release archives and installers, or built with GHC)
- **Claude Code plugin** — `/plugin marketplace add skymanbp/CodeEraser@release`,
  then `/plugin install codeeraser@codeeraser` (SHA256-pinned binary bootstrap)

CodeEraser erases LLM-induced code & document entropy: T1/T2/T3 clone
judgment, dead-code liveness, doc duplication, structural entropy and a
ratcheted repo gate — measurement in Rust, judgment in Haskell.

"CodeEraser"™ is a trademark of skymanbp. Code is Apache-2.0;
per §6, the license does not grant trademark rights.
