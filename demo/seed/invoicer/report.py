"""Plain-text rendering of a priced invoice."""

from .invoice import Invoice
from .money import format_cents

WIDTH = 48


def header(invoice: Invoice) -> list[str]:
    return [f"Invoice {invoice.number}", f"Region  {invoice.region}", "-" * WIDTH]


def body(invoice: Invoice) -> list[str]:
    rows = []
    for line in invoice.lines:
        label = f"{line.quantity} x {line.description}"
        amount = format_cents(line.total_cents)
        rows.append(f"{label:<{WIDTH - len(amount)}}{amount}")
    return rows


def footer(invoice: Invoice) -> list[str]:
    subtotal = format_cents(invoice.subtotal_cents)
    tax = format_cents(invoice.tax_cents)
    total = format_cents(invoice.total_cents)
    return [
        "-" * WIDTH,
        f"{'Subtotal':<{WIDTH - len(subtotal)}}{subtotal}",
        f"{'Tax':<{WIDTH - len(tax)}}{tax}",
        f"{'Total':<{WIDTH - len(total)}}{total}",
    ]


def render(invoice: Invoice) -> str:
    return "\n".join(header(invoice) + body(invoice) + footer(invoice))
