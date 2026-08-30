---
description: "Check for a newer CodeEraser release and say how this install updates"
---

Run the update check through the plugin's own starter and relay its answer:

```sh
sh "${CLAUDE_PLUGIN_ROOT}/bin/ce.sh" update
```

The exit code is the verdict — 0 current, 1 newer release available, 2 unknown
(no network, or the release's manifest could not be read). On 0 the last line is
`latest: <version> — up to date` and on 2 `latest: unknown — <reason>`, and there
is nothing to run. An action line follows only on 1, and it names the ONE action
for this install; follow it exactly and do not substitute another:

- **plugin's bound copy** — reported when the starter's own pinned copy in
  `CLAUDE_PLUGIN_DATA` ran (an installer-placed `ce` on PATH reports the
  installer-sidecar case below instead, even under this command): tell the user
  to run `/plugin update codeeraser` — the plugin's next manifest carries the
  new pins, and the starter re-verifies and re-downloads on the next session.
  Never run `ce update --yes` here: the starter would refuse a binary its
  manifest does not pin.
- **cargo install**: `cargo install codeeraser`.
- **placed by hand / installer sidecar**: `ce update --yes` replaces `ce` and
  `ce-core` in place, verified against the release commit's pins; add
  `--installer` to also save the verified GUI installer and print its path.

Report the current and latest versions, the verdict, and the action in one short
message. Do not download anything yourself.
