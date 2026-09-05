//! Porter's stemmer — M. F. Porter, "An algorithm for suffix
//! stripping", Program 14(3):130–137, 1980 — the 1980 algorithm as
//! published, over ASCII lowercase only: a word carrying any other
//! byte returns unchanged, so a non-English identifier piece is still
//! a deterministic term. In-repo and pure (spec §三: no external
//! model, no dictionary file): a measure m over the [C](VC)^m[V]
//! form and five tables of suffix rules, each step taking the FIRST
//! matching suffix whether or not its condition then fires (the
//! paper's rule, which is why `rational` survives step 2).

/// The stem of one word.
pub fn stem(word: &str) -> String {
    if word.len() <= 2 || !word.bytes().all(|b| b.is_ascii_lowercase()) {
        return word.to_string();
    }
    let mut w = word.as_bytes().to_vec();
    step1a(&mut w);
    step1b(&mut w);
    step1c(&mut w);
    apply(&mut w, STEP2, 0);
    apply(&mut w, STEP3, 0);
    step4(&mut w);
    step5(&mut w);
    String::from_utf8(w).expect("ascii in, ascii out")
}

/// A consonant is a letter other than a e i o u, and other than y
/// when a consonant precedes it (so `y` in `sky` is a vowel and in
/// `yes` a consonant).
fn is_consonant(w: &[u8], i: usize) -> bool {
    match w[i] {
        b'a' | b'e' | b'i' | b'o' | b'u' => false,
        b'y' => i == 0 || !is_consonant(w, i - 1),
        _ => true,
    }
}

/// m of the stem `w[..end]`: the number of VC sequences.
fn measure(w: &[u8], end: usize) -> usize {
    let mut m = 0;
    let mut i = 0;
    while i < end && is_consonant(w, i) {
        i += 1;
    }
    loop {
        while i < end && !is_consonant(w, i) {
            i += 1;
        }
        if i >= end {
            return m;
        }
        while i < end && is_consonant(w, i) {
            i += 1;
        }
        m += 1;
        if i >= end {
            return m;
        }
    }
}

/// *v*: the stem `w[..end]` contains a vowel.
fn has_vowel(w: &[u8], end: usize) -> bool {
    (0..end).any(|i| !is_consonant(w, i))
}

/// *d: the word ends with a double consonant.
fn double_consonant(w: &[u8]) -> bool {
    let n = w.len();
    n >= 2 && w[n - 1] == w[n - 2] && is_consonant(w, n - 1)
}

/// *o: the stem `w[..end]` ends cvc where the second c is not w, x or y.
fn cvc(w: &[u8], end: usize) -> bool {
    end >= 3
        && is_consonant(w, end - 1)
        && !is_consonant(w, end - 2)
        && is_consonant(w, end - 3)
        && !matches!(w[end - 1], b'w' | b'x' | b'y')
}

fn ends(w: &[u8], s: &[u8]) -> bool {
    w.len() >= s.len() && &w[w.len() - s.len()..] == s
}

/// Rule `(cond) S1 -> S2`: when the word ends with S1, replace it by
/// S2 if the stem satisfies `cond`; returns whether S1 matched at
/// all, because a step stops at its first matching suffix.
fn rule(w: &mut Vec<u8>, s1: &[u8], s2: &[u8], cond: impl Fn(&[u8], usize) -> bool) -> bool {
    if !ends(w, s1) {
        return false;
    }
    let stem_len = w.len() - s1.len();
    if cond(w, stem_len) {
        w.truncate(stem_len);
        w.extend_from_slice(s2);
    }
    true
}

fn step1a(w: &mut Vec<u8>) {
    let always = |_: &[u8], _: usize| true;
    let _ = rule(w, b"sses", b"ss", always)
        || rule(w, b"ies", b"i", always)
        || rule(w, b"ss", b"ss", always)
        || rule(w, b"s", b"", always);
}

fn step1b(w: &mut Vec<u8>) {
    if rule(w, b"eed", b"ee", |w, n| measure(w, n) > 0) {
        return;
    }
    let fired = rule(w, b"ed", b"", has_vowel) || rule(w, b"ing", b"", has_vowel);
    if !fired {
        return;
    }
    let n = w.len();
    if ends(w, b"at") || ends(w, b"bl") || ends(w, b"iz") {
        w.push(b'e');
    } else if double_consonant(w) && !matches!(w[n - 1], b'l' | b's' | b'z') {
        w.pop();
    } else if measure(w, n) == 1 && cvc(w, n) {
        w.push(b'e');
    }
}

fn step1c(w: &mut [u8]) {
    let n = w.len();
    if n >= 1 && w[n - 1] == b'y' && has_vowel(w, n - 1) {
        w[n - 1] = b'i';
    }
}

/// Steps 2 and 3: `(m > 0) S1 -> S2` rules in the paper's order. One
/// text per step rather than a tuple table: two tuple tables of byte
/// strings are the same token stream to the clone ratchet, and the
/// ratchet was right that they were one shape.
fn apply(w: &mut Vec<u8>, rules: &str, min_m: usize) {
    for rule_text in rules.split(',') {
        let (s1, s2) = rule_text.split_once('>').expect("s1>s2");
        if rule(w, s1.as_bytes(), s2.as_bytes(), |w, n| {
            measure(w, n) > min_m
        }) {
            return;
        }
    }
}

const STEP2: &str = concat!(
    "ational>ate,tional>tion,enci>ence,anci>ance,izer>ize,abli>able,alli>al,",
    "entli>ent,eli>e,ousli>ous,ization>ize,ation>ate,ator>ate,alism>al,",
    "iveness>ive,fulness>ful,ousness>ous,aliti>al,iviti>ive,biliti>ble"
);

const STEP3: &str = "icate>ic,ative>,alize>al,iciti>ic,ical>ic,ful>,ness>";

/// Step 4 suffixes, `(m > 1) S -> ""`, longer-before-shorter where
/// one is the other's tail (ement / ment / ent); `ion` additionally
/// needs the stem to end in s or t.
const STEP4: &[&[u8]] = &[
    b"al", b"ance", b"ence", b"er", b"ic", b"able", b"ible", b"ant", b"ement", b"ment", b"ent",
    b"ion", b"ou", b"ism", b"ate", b"iti", b"ous", b"ive", b"ize",
];

fn step4(w: &mut Vec<u8>) {
    for s in STEP4 {
        let cond = |w: &[u8], n: usize| {
            measure(w, n) > 1 && (*s != b"ion" || (n >= 1 && matches!(w[n - 1], b's' | b't')))
        };
        if rule(w, s, b"", cond) {
            return;
        }
    }
}

/// Step 5a drops a final e when m > 1, or when m = 1 and the stem is
/// not *o; step 5b singles a final double l when m > 1.
fn step5(w: &mut Vec<u8>) {
    let n = w.len();
    if n >= 1 && w[n - 1] == b'e' {
        let m = measure(w, n - 1);
        if m > 1 || (m == 1 && !cvc(w, n - 1)) {
            w.pop();
        }
    }
    let n = w.len();
    if n >= 2 && w[n - 1] == b'l' && double_consonant(w) && measure(w, n) > 1 {
        w.pop();
    }
}

#[cfg(test)]
#[path = "../../tests/unit/similar/stem.rs"]
mod tests;
