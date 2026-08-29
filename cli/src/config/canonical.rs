//! The knob fingerprint's CANONICAL FORM (plan v2.18 step #14, O39):
//! the set of knobs whose effective value differs from the shipped
//! default, computed generically over the serialized `Config` against
//! the serialized effective default — never field by field.
//!
//! 6.0.0 hashed the whole serialized config, and that fingerprinted
//! the DOCUMENT rather than the judgment: every `Option` knob wrote
//! `null`, so adding an optional knob to the schema moved the digest
//! of every repo that never declared it, and `viol_cost = 10` (the
//! shipped value, spelled out) fingerprinted differently from silence
//! although both judge identically. The digest exists to stop a
//! config edit from moving the lines in silence; an edit that changes
//! no effective knob moved no line, and a knob nobody declared must
//! not move it the day it is declared. Four rules, applied to the
//! serde_json tree of the parsed config against the tree of the
//! effective default (`Config::default()` with the core's own
//! constants standing in for every absent score / trend / graph
//! knob, and every axis weighed at the effective default weight):
//!
//! 1. a `null` leaf is undeclared — dropped;
//! 2. a leaf equal to the default tree's leaf at the same path is a
//!    declaration of the shipped value — dropped (so a threshold
//!    spelled at its default is the same fingerprint as silence, and
//!    a later SHIPPED change of that default moves the digest of the
//!    repos that spelled it, by name, which is the ledger's job);
//! 3. an array is one value: compared whole against the default's
//!    (the exclude list, the classes, the entry globs), with rules 1
//!    and 4 applied INSIDE its objects — a class knob left absent is
//!    dropped there, so a new `ClassKnobs` option never moves a
//!    classed repo's digest, while a class knob spelled at the global
//!    line is kept: its default is inheritance, which is dynamic, not
//!    a constant, and the declaration shadows every later global edit;
//! 4. an object left empty says nothing — dropped;
//! 5. a class's `name` is a label — the uniqueness key at load and
//!    the tag in error text, reaching no threshold, no wire field
//!    and no rendered row — dropped, so a rename is silence (step
//!    #16, O42's open half); declaration ORDER, which is
//!    precedence, still counts through rule 3.
//!
//! What survives is hashed as JSON (serde_json's map is key-ordered,
//! so the bytes are canonical); an empty survivor is no fingerprint
//! at all — the shipped-default repo keeps the bytes it had before
//! the fence existed (K11).

use super::{Config, GraphCfg, TrendCfg};
use serde_json::{Map, Value};

/// The shipped defaults as the core JUDGES them: every optional
/// score / trend / graph knob carries the constant the core uses
/// when the row is absent (`score::knobs::core_defaults`, pinned
/// live against the core's echo), so "declared at the default" and
/// "undeclared" serialize to the same leaf.
pub(crate) fn effective_default() -> Config {
    Config {
        score: crate::score::knobs::core_defaults(),
        trend: TrendCfg::core(),
        graph: GraphCfg {
            scc_floor: Some(crate::score::knobs::CORE_SCC_FLOOR),
            ..GraphCfg::default()
        },
        ..Config::default()
    }
}

/// The canonical tree of `cfg`: an object holding exactly the knobs
/// whose effective value differs from the shipped default, empty when
/// the config judges as the shipped default does.
pub fn canonical(cfg: &Config) -> Value {
    let mut declared = serde_json::to_value(cfg).expect("Config serializes");
    // rule 5: the class name is a label, never a knob
    if let Some(classes) = declared
        .get_mut("rules")
        .and_then(|r| r.get_mut("class"))
        .and_then(Value::as_array_mut)
    {
        for class in classes {
            if let Some(fields) = class.as_object_mut() {
                fields.remove("name");
            }
        }
    }
    let mut shipped = serde_json::to_value(effective_default()).expect("Config serializes");
    // every axis weighs the effective default weight unless named
    let weight = cfg
        .score
        .default_weight
        .or(effective_default().score.default_weight)
        .expect("core default weight");
    shipped["score"]["weights"] = Value::Object(
        crate::score::knobs::AXES
            .iter()
            .map(|axis| ((*axis).to_string(), Value::from(weight)))
            .collect(),
    );
    prune(declared, Some(&shipped)).unwrap_or(Value::Object(Map::new()))
}

/// The four rules over one node: `shipped` is the default tree's node
/// at the same path, `None` inside arrays (rule 3).
pub(super) fn prune(node: Value, shipped: Option<&Value>) -> Option<Value> {
    match node {
        Value::Null => None,
        Value::Object(fields) => {
            let kept: Map<String, Value> = fields
                .into_iter()
                .filter_map(|(key, child)| {
                    let at = shipped.and_then(|s| s.get(&key));
                    prune(child, at).map(|v| (key, v))
                })
                .collect();
            (!kept.is_empty()).then_some(Value::Object(kept))
        }
        Value::Array(items) => {
            let inside: Vec<Value> = items
                .into_iter()
                .filter_map(|item| prune(item, None))
                .collect();
            let whole = Value::Array(inside);
            (shipped != Some(&whole)).then_some(whole)
        }
        leaf => (shipped != Some(&leaf)).then_some(leaf),
    }
}
