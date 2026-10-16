# Security Policy

## Supported versions

| Version | Supported |
|---|---|
| 1.9.x | yes |
| < 1.9 | no: upgrade first |

## Reporting a vulnerability

Please do not open a public issue. Use GitHub's private reporting:
[Security → Report a vulnerability](https://github.com/skymanbp/CodeEraser/security/advisories/new).

Include the `ce --version` line, the operating system and shell, and a
reproduction: the file or the tool call the guard saw and the verdict it
gave, or the `ce` command and its output.

## What counts as a vulnerability here

CodeEraser is a command-line tool, a per-project daemon and a Claude Code
plugin whose hooks deny a duplicate write. The following count:

- A write the guard's `deny` mode is set to refuse that gets through: a
  path spelling, an encoding or a tool the hook does not see.
- A file outside the project that `ce`, the daemon or the plugin reads or
  writes without being asked to.
- Content of a scanned file that makes the daemon or the core run code,
  read elsewhere or hang.
- A release binary the pinned download of `plugin/bin/manifest.env`
  accepts although its SHA256 does not match.

## What does not count

- A clone or a duplicate the index does not find: that is a detection
  gap, reported as an ordinary issue.
- The `observe` and `warn` modes not refusing a write: by design they
  report and do not gate.
- A user who disables the plugin, edits its hooks or answers a permission
  prompt: the hooks run with the user's rights and do not defend against
  the user.
