# invoicer

A small invoicing service: line items in, a priced invoice and a printable
report out. It exists to be edited — the CodeEraser demo runs a scripted
coding agent against it twice.

- [invoicer/cli.py](invoicer/cli.py) — the entry point: reads a JSON order, prints the report it renders
- [invoicer/invoice.py](invoicer/invoice.py) — line totals, subtotal, tax, grand total
- [invoicer/tax.py](invoicer/tax.py) — the tax table and the tax line
- [invoicer/money.py](invoicer/money.py) — cent arithmetic; the one place rounding lives
- [web/api.ts](web/api.ts) — the HTTP shape of the same invoice
- [web/format.ts](web/format.ts) — money formatting for the web face
- [docs/PRICING.md](docs/PRICING.md) — the pricing rules the code implements

Run: `python -m invoicer.cli order.json`.
