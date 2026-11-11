//! The names a format string reads (plan v2.31 step 4 E; design booklet
//! docs/reference/analysis-track.md §5.1 rule 9, RS-1): std's
//! placeholder grammar, applied to every string literal inside a macro
//! call whatever the macro — a user macro forwarding to format_args
//! writes the same placeholders, and a name read once too often only
//! loses a finding, the safe side.

/// The names a string's placeholders read: `{name}` and `{name:…}`
/// capture a variable, and so does every `name$` in a format spec
/// (`{:width$}`, `{:.prec$}`, `{:>w$.p$}`); `{}`, `{0}` and `{:1$}`
/// read nothing, and `{{` / `}}` are literal braces. A raw string reads
/// the same: its prefix and quotes hold no brace.
pub(super) fn format_names(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    each_field(text, |rest| {
        let len = rest.find('}')?;
        let inner = &rest[..len];
        let (arg, spec) = inner.split_once(':').unwrap_or((inner, ""));
        out.extend(ident(arg.trim()));
        out.extend(counts(spec));
        Some(len + 1)
    });
    out
}

/// A string's `{` openings, left to right, paired the way the
/// formatters pair braces — `{{` is one literal brace: `field` is
/// called at each `{` that opens a field with the text after it, and
/// answers how many bytes past that `{` the walk resumes, or `None` to
/// stop. The one walk both `format_names` and `access_write::braced`
/// read (v2.35 V9).
pub(super) fn each_field(text: &str, mut field: impl FnMut(&str) -> Option<usize>) {
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'{' {
            i += 1;
        } else if bytes.get(i + 1) == Some(&b'{') {
            i += 2;
        } else {
            match field(&text[i + 1..]) {
                Some(step) => i += 1 + step,
                None => break,
            }
        }
    }
}

/// The spec's `name$` counts: the identifier ending right before
/// each `$`.
fn counts(spec: &str) -> Vec<String> {
    spec.match_indices('$')
        .filter_map(|(at, _)| {
            let head = &spec[..at];
            let start = head
                .rfind(|c: char| !(c.is_alphanumeric() || c == '_'))
                .map_or(0, |p| p + 1);
            ident(&head[start..])
        })
        .collect()
}

/// The text when it is an identifier (a letter or `_` first), not an
/// index.
fn ident(text: &str) -> Option<String> {
    let first = text.chars().next()?;
    let ok = (first.is_alphabetic() || first == '_')
        && text.chars().all(|c| c.is_alphanumeric() || c == '_');
    ok.then(|| text.to_owned())
}
