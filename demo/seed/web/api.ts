// The HTTP shape of a priced invoice: the same numbers the CLI prints,
// formatted once through format.ts.

import { formatCents, formatQuantity } from "./format";

export interface PricedLine {
  sku: string;
  description: string;
  quantity: number;
  totalCents: number;
}

export interface PricedInvoice {
  number: string;
  region: string;
  lines: PricedLine[];
  subtotalCents: number;
  taxCents: number;
  totalCents: number;
}

export function toResponse(invoice: PricedInvoice) {
  return {
    number: invoice.number,
    region: invoice.region,
    lines: invoice.lines.map((line) => ({
      sku: line.sku,
      label: `${formatQuantity(line.quantity, "unit")} ${line.description}`,
      total: formatCents(line.totalCents),
    })),
    subtotal: formatCents(invoice.subtotalCents),
    tax: formatCents(invoice.taxCents),
    total: formatCents(invoice.totalCents),
  };
}
