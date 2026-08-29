"""Tax lookup by region and the tax line on a subtotal."""

from .money import scale_cents

# rate = numerator / denominator, kept as integers so the tax line is
# computed without a float in sight
RATES: dict[str, tuple[int, int]] = {
    "US-CA": (725, 10000),
    "US-NY": (4, 100),
    "EU-DE": (19, 100),
    "EU-FR": (20, 100),
}


class UnknownRegion(ValueError):
    """Raised for a region without a tax rate; never a silent zero."""


def rate_for(region: str) -> tuple[int, int]:
    try:
        return RATES[region]
    except KeyError as missing:
        raise UnknownRegion(region) from missing


def tax_line(subtotal_cents: int, region: str) -> int:
    numerator, denominator = rate_for(region)
    return scale_cents(subtotal_cents, numerator, denominator)
