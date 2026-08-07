"""PreToolUse(Edit|Write) payload collector (M0, ADR-007 open item).

Claude Code pipes the hook event JSON via stdin; official docs carry no
verbatim Edit/Write example, so we capture real ones. Appends one NDJSON
line per event next to this script; never blocks the tool call (exit 0).
"""

import json
import os
import sys


def main() -> int:
    raw = sys.stdin.read()
    try:
        event = json.loads(raw)
    except json.JSONDecodeError:
        event = {"_unparsed": raw}
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)), "payloads.ndjson")
    with open(out, "a", encoding="utf-8") as fh:
        fh.write(json.dumps(event, ensure_ascii=False) + "\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
