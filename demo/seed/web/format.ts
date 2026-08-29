// Money formatting for the web face. Mirrors invoicer/money.py's
// format_cents on purpose: one rule, two languages, each written once.

export function formatCents(cents: number, currency = "USD"): string {
  if (!/^[A-Z]{3}$/.test(currency)) {
    throw new Error(`not an ISO 4217 code: ${currency}`);
  }
  const sign = cents < 0 ? "-" : "";
  const abs = Math.abs(cents);
  const whole = Math.floor(abs / 100);
  const frac = String(abs % 100).padStart(2, "0");
  return `${currency} ${sign}${whole}.${frac}`;
}

export function formatQuantity(quantity: number, unit: string): string {
  return quantity === 1 ? `1 ${unit}` : `${quantity} ${unit}s`;
}
