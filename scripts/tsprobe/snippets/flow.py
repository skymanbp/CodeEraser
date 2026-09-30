"""Flow sample (analysis-track §5.3, round 3): every statement,
declaration and access shape the flow/1 lowering table names."""

import os
import sys

counter = 0


class Account:
    def __init__(self, owner, balance=0):
        self.owner = owner
        self.balance = balance

    @classmethod
    def empty(cls, owner: str, *, note: str = "") -> "Account":
        return cls(owner)

    def deposit(self, amount, /, *extra, **options):
        self.balance += amount
        self.history[0] = amount
        return self.balance


def branches(x, y=1, *args, z: int = 2, **kw):
    global counter
    counter += 1
    if x > 0:
        label = "pos"
    elif x < 0:
        label = "neg"
    elif x == 0 and y:
        label = "zero"
    else:
        label = None
    total: int = 0
    for item in args:
        if item is None:
            continue
        if item == -1:
            break
        total += item
    else:
        total -= 1
    n = 3
    while n:
        n -= 1
    else:
        n = -1
    while True:
        if total > 10:
            break
        total = total + 1
    assert total >= 0, "negative"
    return label, total, z, kw


def matching(command):
    match command:
        case ["go", direction]:
            result = direction
        case {"action": act, **rest}:
            result = (act, rest)
        case Point(x=0, y=yy) if yy > 0:
            result = yy
        case str() | bytes() as raw:
            result = raw
        case _:
            result = None
    return result


def guarded(path):
    handle = None
    try:
        handle = open(path, mode="r")
        data = handle.read()
    except (OSError, ValueError) as err:
        data = str(err)
    except KeyError:
        raise
    else:
        data = data.strip()
    finally:
        if handle is not None:
            handle.close()
    with open(path) as fh, open(path) as gh:
        first = fh.readline()
    del gh
    raise ValueError(first) from None


def closures(values):
    base = 10
    scale = 2

    def helper(v):
        nonlocal scale
        scale = scale + 1
        return v * base

    class Local:
        tag = base

    squares = [v * v for v in values if v > base]
    lookup = {k: v for k, v in enumerate(values)}
    unique = {v % base for v in values}
    lazy = (v + scale for v in values)
    adder = lambda d, e=base: d + e
    if (size := len(values)) > 3 and (flag := True):
        size = 0
    pick = base if values else (scale := 5)
    a, (b, *rest) = values[0], values[1:]
    squares[0] = a
    print(f"{base} {flag}")
    return helper, Local, squares, lookup, lazy, adder, size, pick, b, rest, unique


def dynamic(expr):
    value = eval(expr)
    exec("print(value)")
    return value


def fatal(code):
    if code:
        sys.exit(code)
    os._exit(1)
    unreachable = 1
    pass
