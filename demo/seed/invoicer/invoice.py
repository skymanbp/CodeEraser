"""Price an order: line totals, subtotal, tax, grand total."""

from dataclasses import dataclass

from .money import to_cents
from .tax import tax_line


@dataclass(frozen=True)
class Line:
    sku: str
    description: str
    quantity: int
    unit_cents: int

    @property
    def total_cents(self) -> int:
        return self.quantity * self.unit_cents


@dataclass(frozen=True)
class Invoice:
    number: str
    region: str
    lines: tuple[Line, ...]

    @property
    def subtotal_cents(self) -> int:
        return sum(line.total_cents for line in self.lines)

    @property
    def tax_cents(self) -> int:
        return tax_line(self.subtotal_cents, self.region)

    @property
    def total_cents(self) -> int:
        return self.subtotal_cents + self.tax_cents


def price(order: dict) -> Invoice:
    """Turn a raw order dict into a priced, immutable Invoice."""
    lines = tuple(
        Line(
            sku=item["sku"],
            description=item["description"],
            quantity=int(item["quantity"]),
            unit_cents=to_cents(item["unit_price"]),
        )
        for item in order["items"]
    )
    return Invoice(number=order["number"], region=order["region"], lines=lines)
