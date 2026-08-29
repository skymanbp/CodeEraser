"""`python -m invoicer.cli order.json` — price an order and print it."""

import json
import sys

from .invoice import price
from .report import render


def main(argv: list[str]) -> int:
    if len(argv) != 2:
        print("usage: python -m invoicer.cli <order.json>", file=sys.stderr)
        return 2
    with open(argv[1], encoding="utf-8") as handle:
        order = json.load(handle)
    print(render(price(order)))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
