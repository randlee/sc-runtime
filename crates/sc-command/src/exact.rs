//! Shared object-key checks for the envelope and [`OpError`](crate::OpError) decoders.

use serde_json::{Map, Value};

use crate::DISCRIMINANT_CODE;

pub(crate) fn required<'a>(map: &'a Map<String, Value>, key: &str) -> Result<&'a Value, String> {
    map.get(key)
        .ok_or_else(|| format!("{DISCRIMINANT_CODE}: missing {key}"))
}

pub(crate) fn require_exact_keys(map: &Map<String, Value>, keys: &[&str]) -> Result<(), String> {
    for key in map.keys() {
        if !keys.contains(&key.as_str()) {
            return Err(format!("{DISCRIMINANT_CODE}: unknown key {key}"));
        }
    }
    for key in keys {
        if !map.contains_key(*key) {
            return Err(format!("{DISCRIMINANT_CODE}: missing {key}"));
        }
    }
    Ok(())
}
