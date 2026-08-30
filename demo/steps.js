"use strict";
// The scripted agent's moves — the SAME seven writes in the SAME order in
// both runs, each built from the seed alone so no write depends on an
// earlier one having landed. They are the moves a reactive coding agent
// makes on a long-lived tree: copy a helper instead of importing it,
// re-implement a renderer beside the old one, paste the paragraph that
// already exists, grow the busiest file, leave the old module behind.

/** Text of the seed file between two markers (both exclusive). */
function between(text, from, to) {
  const start = text.indexOf(from);
  const end = text.indexOf(to, start);
  if (start < 0 || end < 0) throw new Error(`markers not found: ${from} .. ${to}`);
  return text.slice(start, end);
}

/** The seed's `to_cents` and `scale_cents`, verbatim (money.py). */
function moneyHelpers(seed) {
  return between(seed["invoicer/money.py"], "def to_cents", "\n\n\ndef format_cents");
}

/** Step 1 — a discount module that copies its rounding instead of importing it. */
function discountModule(seed) {
  return `"""Discounts: percentage and fixed, applied in cents."""

from decimal import ROUND_HALF_UP, Decimal

# copied from money.py so this module has no imports to keep in sync
${moneyHelpers(seed)}


def apply_percent(cents: int, percent: int) -> int:
    """Take a percentage off the cents, rounding half up once."""
    return cents - scale_cents(cents, percent, 100)


def apply_fixed(cents: int, off: str) -> int:
    """Take a fixed decimal amount off the cents, never below zero."""
    return max(0, cents - to_cents(off))
`;
}

/** Step 2 — a "compact" renderer: the old rows and footer, renamed and reordered. */
function compactRenderer(seed) {
  return `${seed["invoicer/report.py"]}

def compact_rows(inv: Invoice) -> list[str]:
    out = []
    for item in inv.lines:
        money = format_cents(item.total_cents)
        text = f"{item.quantity} x {item.description}"
        out.append(f"{text:<{WIDTH - len(money)}}{money}")
    return out


def compact_footer(inv: Invoice) -> list[str]:
    due = format_cents(inv.total_cents)
    vat = format_cents(inv.tax_cents)
    sub = format_cents(inv.subtotal_cents)
    rows = [
        f"{'Subtotal':<{WIDTH - len(sub)}}{sub}",
        f"{'Tax':<{WIDTH - len(vat)}}{vat}",
        f"{'Total':<{WIDTH - len(due)}}{due}",
    ]
    rows.insert(0, "=" * WIDTH)
    return rows


def render_compact(inv: Invoice) -> str:
    return "\\n".join(header(inv) + compact_rows(inv) + compact_footer(inv))
`;
}

/** Step 3 — a discounts page that opens by pasting the pricing rules. */
function discountsDoc(seed) {
  const rules = between(seed["docs/PRICING.md"], "Every line item", "\n\nTax rates");
  return `# Discounts

${rules}

A percentage discount is taken off the subtotal before tax; a fixed
discount is taken off the grand total after tax. Both are recorded on the
invoice as their own line so the printed total still adds up.
`;
}

/** Step 4 — CSV export appended to the busiest module. */
function csvExport(seed) {
  return `${seed["invoicer/invoice.py"]}

CSV_COLUMNS = ("sku", "description", "quantity", "unit_cents", "total_cents")


def csv_rows(invoice: Invoice) -> list[tuple[str, ...]]:
    rows = [CSV_COLUMNS]
    for line in invoice.lines:
        rows.append(
            (
                line.sku,
                line.description,
                str(line.quantity),
                str(line.unit_cents),
                str(line.total_cents),
            )
        )
    return rows


def csv_summary(invoice: Invoice) -> list[tuple[str, ...]]:
    return [
        ("subtotal_cents", str(invoice.subtotal_cents)),
        ("tax_cents", str(invoice.tax_cents)),
        ("total_cents", str(invoice.total_cents)),
    ]


def quote(cell: str) -> str:
    if any(ch in cell for ch in ',"\\n'):
        return '"' + cell.replace('"', '""') + '"'
    return cell


def to_csv(invoice: Invoice) -> str:
    table = csv_rows(invoice) + [()] + csv_summary(invoice)
    return "\\n".join(",".join(quote(cell) for cell in row) for row in table) + "\\n"


def write_csv(invoice: Invoice, path: str) -> None:
    with open(path, "w", encoding="utf-8", newline="") as handle:
        handle.write(to_csv(invoice))
`;
}

/** Step 5 — a JSON renderer, written fresh. */
function jsonRenderer() {
  return `"""JSON rendering of a priced invoice."""

import json

from .invoice import Invoice


def as_dict(invoice: Invoice) -> dict:
    return {
        "number": invoice.number,
        "region": invoice.region,
        "lines": [
            {"sku": line.sku, "quantity": line.quantity, "total_cents": line.total_cents}
            for line in invoice.lines
        ],
        "subtotal_cents": invoice.subtotal_cents,
        "tax_cents": invoice.tax_cents,
        "total_cents": invoice.total_cents,
    }


def render(invoice: Invoice) -> str:
    return json.dumps(as_dict(invoice), indent=2)
`;
}

/** Step 6 — the CLI switches to the JSON renderer; report.py stays behind. */
function cliOnJson(seed) {
  return seed["invoicer/cli.py"].replace("from .report import render", "from .report_json import render");
}

/** Step 7 — the API formats money with a local copy of format.ts's function. */
function apiWithLocalFormat(seed) {
  const fn = between(seed["web/format.ts"], "export function formatCents", "\n\nexport function formatQuantity");
  return seed["web/api.ts"]
    .replace('import { formatCents, formatQuantity } from "./format";', 'import { formatQuantity } from "./format";')
    .replace("export function toResponse", `${fn.replace("export function", "function")}\n\nexport function toResponse`);
}

/** The compact variant that reuses the renderer instead of copying it: the
 *  seed's report.py, plus a `render_compact` that differs from `render`
 *  only where the two really differ (the rule above the totals). Built
 *  from the seed like every other write, so it cannot drift from what
 *  step 2 duplicated. */
function dedupedRenderer(seed) {
  return `${seed["invoicer/report.py"]}

def render_compact(invoice: Invoice) -> str:
    rows = footer(invoice)
    rows[0] = "=" * WIDTH
    return "\\n".join(header(invoice) + body(invoice) + rows)
`;
}

/** The one write the agent makes because a gate asked for it rather than
 *  because the task did: the Stop audit refuses to end the turn while the
 *  two duplicate blocks it names are still there. Only the run with the
 *  hooks in the loop ever gets here — nothing in the other run asks. */
function repair(seed) {
  return {
    id: 8,
    file: "invoicer/report.py",
    say: "The Stop audit named two duplicate blocks — reuse the rows and footer already there.",
    content: dedupedRenderer(seed),
  };
}

/** The seven writes, in order. `say` is the agent's own one-line narration. */
function steps(seed) {
  return [
    { id: 1, file: "invoicer/discount.py", say: "Add discounts. I'll keep the rounding helpers local so the module is self-contained.", content: discountModule(seed) },
    { id: 2, file: "invoicer/report.py", say: "Add a compact report variant next to the existing renderer.", content: compactRenderer(seed) },
    { id: 3, file: "docs/DISCOUNTS.md", say: "Document discounts; open with the pricing rules so the page stands alone.", content: discountsDoc(seed) },
    { id: 4, file: "invoicer/invoice.py", say: "Add CSV export where the invoice already lives.", content: csvExport(seed) },
    { id: 5, file: "invoicer/report_json.py", say: "Add a JSON renderer.", content: jsonRenderer() },
    { id: 6, file: "invoicer/cli.py", say: "Switch the CLI to JSON output.", content: cliOnJson(seed) },
    { id: 7, file: "web/api.ts", say: "Format money in the API handler without reaching into format.ts.", content: apiWithLocalFormat(seed) },
  ];
}

module.exports = { steps, repair };
