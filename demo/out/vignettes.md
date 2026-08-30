**A copied helper, refused before the file exists.** Move 1 of the run above, on its own. The reason names the region the content duplicates and the ordering that would pass, so the refusal is actionable rather than a veto.

```console
$ Write invoicer/discount.py
✗ ce: content for <work>/invoicer/discount.py duplicates 1 indexed region(s): invoicer/money.py:1-18 (89 tokens). Reuse the existing implementation instead of re-writing it. Moving it? Trim the source region first: the probe verifies against the current tree, and the same write then passes.
```

**One line, two mouths.** `ce.toml` puts `invoicer/**` on `file_lines_fail = 40`. The write-time guard refuses the write that would cross it, and `ce scan` grades the same tree against the same number — one declaration, read by the hook and by CI.

```console
$ Write invoicer/invoice.py
✗ ce: this write leaves <work>/invoicer/invoice.py at 93 lines, past the hard budget of 40 (plan §4.1). Split the file instead of growing it.
$ ce scan .
FAIL invoicer/invoice.py:1 file-lines = 51 (limit 40) [invoicer/invoice.py]
warn invoicer/report.py:1 file-lines = 35 (limit 30) [invoicer/report.py]
scanned 9 files / 19 functions — 1 warn, 1 fail -> FAIL (failed: hard_line)
```
