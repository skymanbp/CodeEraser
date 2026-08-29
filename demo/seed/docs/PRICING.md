# Pricing rules

Every line item is priced in whole cents. A unit price arrives as a
decimal string, is converted once with round-half-up to cents, and is
never rounded again: the line total is the cent price multiplied by the
quantity, the subtotal is the plain sum of line totals, and the tax line
is computed on that subtotal with a single round-half-up at the end.
Rounding twice — once per line and once on the sum — is the classic
off-by-one-cent bug, and the code avoids it by keeping every intermediate
value as an integer number of cents.

Tax rates are looked up by region code. An unknown region is an error,
never a zero rate: a silently untaxed invoice is worse than a refused one.

The grand total is subtotal plus tax. Nothing else is added.
