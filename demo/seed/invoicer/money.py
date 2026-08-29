"""Cent arithmetic. The only module allowed to round."""

from decimal import ROUND_HALF_UP, Decimal


def to_cents(amount: str) -> int:
    """Parse a decimal string into whole cents, rounding half up once."""
    value = Decimal(amount).quantize(Decimal("0.01"), rounding=ROUND_HALF_UP)
    return int(value * 100)


def scale_cents(cents: int, numerator: int, denominator: int) -> int:
    """Multiply cents by a ratio and round half up, staying in integers."""
    scaled = cents * numerator * 2 + denominator
    return scaled // (denominator * 2)


def format_cents(cents: int, currency: str = "USD") -> str:
    """Render cents as `USD 12.34` with a sign in front of the amount."""
    sign = "-" if cents < 0 else ""
    whole, frac = divmod(abs(cents), 100)
    return f"{currency} {sign}{whole}.{frac:02d}"
