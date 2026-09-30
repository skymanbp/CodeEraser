//! Flow sample (analysis-track §5.3, round 3): every statement,
//! declaration and access shape the flow/1 lowering table names.

use std::collections::HashMap;

pub struct Point {
    pub x: i32,
    pub y: i32,
}

pub fn branches(a: i32, (b, c): (i32, i32), Point { x, y: why }: Point, mut acc: Vec<i32>, _unused: u8) -> i32 {
    let label = if a > 0 {
        1
    } else if a < 0 {
        -1
    } else {
        0
    };
    if let Some(&first) = acc.first() {
        acc.push(first);
    } else {
        acc.push(0);
    }
    let mut n = 3;
    while n > 0 {
        n -= 1;
    }
    while let Some(top) = acc.pop() {
        if top == 0 {
            continue;
        }
        if top < 0 {
            break;
        }
        n += top;
    }
    for i in 0..b {
        n = n + i;
    }
    'outer: loop {
        for j in 0..c {
            if j == x {
                continue 'outer;
            }
            if j == why {
                break 'outer;
            }
        }
        break;
    }
    let found = 'search: {
        if n > 10 {
            break 'search true;
        }
        false
    };
    let value = loop {
        break n;
    };
    'check: {
        if n > 100 {
            break 'check;
        }
        n = 0;
    }
    while true {
        n += 1;
        break;
    }
    label + value + i32::from(found)
}

pub fn matching(opt: Option<i32>, pair: (i32, i32)) -> i32 {
    let Some(present) = opt else {
        return -2;
    };
    let score = match opt {
        Some(n) if n > 0 => n,
        Some(0) => 0,
        Some(small @ -5..=-1) => small,
        None => return -1,
        _ => 1,
    };
    match pair {
        (0, _) => {}
        (first, second) => {
            let _ = first + second;
        }
    }
    score + present
}

pub fn accesses(point: &mut Point, items: &mut Vec<i32>, flag: bool) -> i32 {
    let mut x = 0;
    let x2;
    x = x + 1;
    x += point.x;
    point.y = x;
    items[0] = x;
    let r = &mut x;
    *r = 5;
    let s = &x;
    let shadow = 1;
    let shadow = shadow + 1;
    let mut y = 0;
    let _ = flag && {
        y = 1;
        true
    };
    let _ = flag || {
        y = 4;
        false
    };
    let pick = if flag {
        y = 2;
        y
    } else {
        3
    };
    x2 = pick;
    let text = format!("{}", y);
    println!("{x2} {text} {s}");
    let (p, q) = (shadow, x2);
    let Point { x: px, y: py } = Point { x: p, y: q };
    let built = Point { x: px, y };
    px + py + built.x
}

pub fn closures(values: &[i32]) -> impl Fn(i32) -> i32 {
    let base = 10;
    let mut scale = 2;
    let mut bump = |d: i32| {
        scale += d;
        scale
    };
    bump(1);
    fn helper(v: i32) -> i32 {
        v * 2
    }
    let total: i32 = values.iter().map(|v| v + base).sum();
    let pending = async move { base + 1 };
    drop(pending);
    move |k| k + base + helper(total)
}

pub fn fatal(code: i32) -> i32 {
    if code > 0 {
        std::process::exit(code);
    }
    if code < 0 {
        panic!("negative {code}");
    }
    unreachable!();
}

pub fn more_fatal(flag: bool) -> ! {
    if flag {
        todo!()
    }
    unimplemented!("never {}", flag)
}

pub fn lookup(map: &HashMap<String, i32>, key: &str) -> Result<i32, String> {
    let found = map.get(key).ok_or_else(|| key.to_string())?;
    unsafe {
        let raw = found as *const i32;
        let _ = *raw;
    }
    Ok(*found)
}

impl Point {
    pub fn shift(&mut self, by: i32) -> i32 {
        self.x += by;
        self.x
    }
}
