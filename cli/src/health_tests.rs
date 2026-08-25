//! health.rs unit battery, split out per the E01 file cap (the
//! store.rs / store_tests.rs precedent): the observe feed's degraded
//! counter, whose contract is that it sees a producer wherever that
//! producer stamps itself.

use super::line_degraded;
use serde_json::json;

/// The nested case is the whole point: a Stop audit whose dedup leg
/// was fine but whose ce-core was missing carries `degraded: false`
/// at the top level and `true` one level down, and the old counter
/// read only the top. The last two rows keep the widening honest —
/// the SHAPE is the contract, so a `degraded` that is not a boolean
/// is not a stamp, and one buried two levels deep is not a producer.
#[test]
fn a_nested_producer_counts_and_a_healthy_line_does_not() {
    assert!(line_degraded(&json!({"event": "guard", "degraded": true})));
    assert!(line_degraded(&json!({
        "event": "stop_audit", "degraded": false,
        "fourclass": {"pairs": 3, "degraded": true}
    })));
    assert!(!line_degraded(&json!({
        "event": "stop_audit", "degraded": false,
        "fourclass": {"pairs": 3, "degraded": false}
    })));
    assert!(!line_degraded(&json!({"degraded": "maybe"})));
    assert!(!line_degraded(&json!({"a": {"b": {"degraded": true}}})));
}
