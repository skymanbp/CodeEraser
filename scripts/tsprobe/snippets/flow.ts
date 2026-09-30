// Flow sample (analysis-track §5.3, round 3): every statement,
// declaration and access shape the flow/1 lowering table names.

interface Point {
  x: number;
  y: number;
}

declare const process: { exit(code: number): never };

let counter = 0;

function branches(a: number, b = 1, { x, y: why }: Point = { x: 0, y: 0 }, [first, , third]: number[] = [], ...rest: number[]): string {
  let label: string;
  if (a > 0) {
    label = "pos";
  } else if (a < 0) {
    label = "neg";
  } else label = "zero";
  for (let i = 0; i < rest.length; i++) {
    if (rest[i] === 0) continue;
    if (rest[i] < 0) break;
    counter += rest[i];
  }
  for (const value of rest) {
    counter -= value;
  }
  for (const key in { a, b }) {
    label += key;
  }
  let n = 3;
  while (n) n--;
  do {
    n++;
  } while (n < 2);
  while (true) {
    if (n > 10) break;
    n = n + 1;
  }
  for (;;) {
    break;
  }
  outer: for (const p of rest) {
    for (const q of rest) {
      if (p === q) continue outer;
      if (p > q) break outer;
    }
  }
  found: {
    if (n > 5) break found;
    n = 0;
  }
  return label + x + why + first + third;
}

function switching(kind: string): number {
  let score = 0;
  switch (kind) {
    case "a":
      score = 1;
    case "b":
      score += 2;
      break;
    case "c": {
      score = 3;
      break;
    }
    default:
      score = -1;
  }
  return score;
}

function guarded(path: string): string {
  let data = "";
  try {
    data = path.trim();
    if (!data) throw new Error("empty");
  } catch (err) {
    data = String(err);
  } finally {
    counter++;
  }
  try {
    JSON.parse(data);
  } catch {
    return "";
  }
  return data;
}

function accesses(obj: { p: number; list: number[] }, flag: boolean): number {
  const { p, list: [head] } = obj;
  let x = 0;
  let y = 0;
  x = x + 1;
  x += p;
  ++x;
  x--;
  obj.p = x;
  obj.list[0] = head;
  flag && (y = 1);
  flag || (y = 2);
  y ??= 3;
  y ||= 4;
  y &&= 5;
  const pick = flag ? (x = 6) : (y = 7);
  let shadow = 1;
  {
    let shadow = 2;
    shadow++;
  }
  var hoisted = shadow;
  const shorthand = { x, y };
  return pick + hoisted + shorthand.x;
}

function closures(values: number[]): () => number {
  const base = 10;
  let scale = 2;
  const add = (d: number) => d + base;
  function helper(v: number): number {
    scale = scale + 1;
    return v * base;
  }
  const f = function named(k: number) {
    return k + scale;
  };
  class Local {
    tag = base;
    read(): number {
      return this.tag + scale;
    }
  }
  return () => add(helper(values[0])) + f(1) + new Local().read();
}

function dynamic(code: string): unknown {
  const local = 1;
  const result = eval(code);
  const make = new Function("a", "return a + 1");
  return [result, make, local];
}

// `with` is sloppy-mode JavaScript that tsc rejects in a module; it is
// here because the lowering reads it as a dynamic statement (§5.1 rule 7)
function scoped(obj: { a: number }): number {
  let out = 0;
  with (obj) {
    out = a;
  }
  return out;
}

function fatal(code: number): void {
  if (code) {
    process.exit(code);
  }
  process.exit(1);
  counter = 0;
}

function* generate(limit: number): Generator<number> {
  for (let i = 0; i < limit; i++) yield i;
}

class Shape {
  private area = 0;
  constructor(private readonly side: number) {}
  grow(this: Shape, by?: number): number {
    this.area = this.side * (by ?? 1);
    return this.area;
  }
}
