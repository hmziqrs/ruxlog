use std::collections::HashMap;

use rand::{rngs::StdRng, SeedableRng};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::services::seed_config::SeedMode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableRange {
    pub from: i32,
    pub to: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedOutcome {
    pub ranges: HashMap<String, TableRange>,
    pub seed_run_id: Option<i32>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl SeedOutcome {
    pub fn counts(&self) -> HashMap<String, i32> {
        self.ranges
            .iter()
            .map(|(k, v)| {
                let count = if v.to > 0 && v.to >= v.from {
                    v.to - v.from + 1
                } else {
                    0
                };
                (k.clone(), count)
            })
            .collect()
    }

    pub fn ranges_json(&self) -> Value {
        let mut map = serde_json::Map::new();
        for (k, v) in &self.ranges {
            map.insert(
                k.clone(),
                serde_json::json!({
                    "from": v.from,
                    "to": v.to,
                }),
            );
        }
        Value::Object(map)
    }
}

#[derive(Debug, Error)]
pub enum SeedError {
    #[error("database error: {0}")]
    Db(String),
}

impl From<sea_orm::DbErr> for SeedError {
    fn from(value: sea_orm::DbErr) -> Self {
        SeedError::Db(value.to_string())
    }
}

pub type SeedResult<T> = Result<T, SeedError>;

/// Progress callback function type for seed operations
pub type ProgressCallback = Box<dyn Fn(String) + Send + Sync>;

pub fn compute_range(before: i32, after: i32) -> TableRange {
    if after > before {
        TableRange {
            from: before + 1,
            to: after,
        }
    } else {
        TableRange { from: 0, to: 0 }
    }
}

pub fn seeded_rng(seed_mode: Option<SeedMode>) -> StdRng {
    let seed_value = seed_mode.unwrap_or_default().to_seed();
    StdRng::seed_from_u64(seed_value)
}

pub fn size_label(count: u32) -> &'static str {
    match count {
        0..=15 => "low",
        16..=60 => "default",
        61..=150 => "medium",
        151..=300 => "large",
        301..=700 => "very large",
        _ => "massive",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── compute_range ─────────────────────────────────────────────────────

    #[test]
    fn compute_range_normal_growth() {
        let range = compute_range(0, 10);
        assert_eq!(range.from, 1);
        assert_eq!(range.to, 10);
    }

    #[test]
    fn compute_range_no_growth() {
        let range = compute_range(5, 5);
        assert_eq!(range.from, 0);
        assert_eq!(range.to, 0);
    }

    #[test]
    fn compute_range_negative_growth() {
        let range = compute_range(10, 3);
        assert_eq!(range.from, 0);
        assert_eq!(range.to, 0);
    }

    #[test]
    fn compute_range_from_zero() {
        let range = compute_range(0, 0);
        assert_eq!(range.from, 0);
        assert_eq!(range.to, 0);
    }

    #[test]
    fn compute_range_single_insert() {
        let range = compute_range(0, 1);
        assert_eq!(range.from, 1);
        assert_eq!(range.to, 1);
    }

    #[test]
    fn compute_range_large_numbers() {
        let range = compute_range(999_990, 1_000_000);
        assert_eq!(range.from, 999_991);
        assert_eq!(range.to, 1_000_000);
    }

    #[test]
    fn compute_range_negative_before() {
        let range = compute_range(-5, 3);
        assert_eq!(range.from, -4);
        assert_eq!(range.to, 3);
    }

    // ── size_label ────────────────────────────────────────────────────────

    #[test]
    fn size_label_boundaries() {
        assert_eq!(size_label(0), "low");
        assert_eq!(size_label(15), "low");
        assert_eq!(size_label(16), "default");
        assert_eq!(size_label(60), "default");
        assert_eq!(size_label(61), "medium");
        assert_eq!(size_label(150), "medium");
        assert_eq!(size_label(151), "large");
        assert_eq!(size_label(300), "large");
        assert_eq!(size_label(301), "very large");
        assert_eq!(size_label(700), "very large");
        assert_eq!(size_label(701), "massive");
    }

    #[test]
    fn size_label_midpoints() {
        assert_eq!(size_label(8), "low");
        assert_eq!(size_label(38), "default");
        assert_eq!(size_label(100), "medium");
        assert_eq!(size_label(225), "large");
        assert_eq!(size_label(500), "very large");
        assert_eq!(size_label(10_000), "massive");
    }

    // ── SeedOutcome::counts ───────────────────────────────────────────────

    #[test]
    fn counts_normal_ranges() {
        let mut outcome = SeedOutcome {
            ranges: HashMap::new(),
            seed_run_id: None,
            errors: vec![],
            warnings: vec![],
        };
        outcome.ranges.insert("users".to_string(), TableRange { from: 1, to: 10 });
        outcome.ranges.insert("posts".to_string(), TableRange { from: 1, to: 50 });

        let counts = outcome.counts();
        assert_eq!(counts.get("users"), Some(&10));
        assert_eq!(counts.get("posts"), Some(&50));
    }

    #[test]
    fn counts_zero_range() {
        let mut outcome = SeedOutcome {
            ranges: HashMap::new(),
            seed_run_id: None,
            errors: vec![],
            warnings: vec![],
        };
        outcome.ranges.insert("tags".to_string(), TableRange { from: 0, to: 0 });

        let counts = outcome.counts();
        assert_eq!(counts.get("tags"), Some(&0));
    }

    #[test]
    fn counts_inverted_range() {
        let mut outcome = SeedOutcome {
            ranges: HashMap::new(),
            seed_run_id: None,
            errors: vec![],
            warnings: vec![],
        };
        // from > to: should yield 0
        outcome.ranges.insert("bad".to_string(), TableRange { from: 10, to: 5 });

        let counts = outcome.counts();
        assert_eq!(counts.get("bad"), Some(&0));
    }

    #[test]
    fn counts_empty_outcome() {
        let outcome = SeedOutcome {
            ranges: HashMap::new(),
            seed_run_id: None,
            errors: vec![],
            warnings: vec![],
        };
        let counts = outcome.counts();
        assert!(counts.is_empty());
    }

    // ── SeedOutcome::ranges_json ──────────────────────────────────────────

    #[test]
    fn ranges_json_valid_output() {
        let mut outcome = SeedOutcome {
            ranges: HashMap::new(),
            seed_run_id: None,
            errors: vec![],
            warnings: vec![],
        };
        outcome.ranges.insert("users".to_string(), TableRange { from: 1, to: 25 });

        let json = outcome.ranges_json();
        let obj = json.as_object().expect("should be a JSON object");
        let users = obj.get("users").expect("should have users key");
        assert_eq!(users.get("from").unwrap(), 1);
        assert_eq!(users.get("to").unwrap(), 25);
    }

    #[test]
    fn ranges_json_empty_outcome() {
        let outcome = SeedOutcome {
            ranges: HashMap::new(),
            seed_run_id: None,
            errors: vec![],
            warnings: vec![],
        };
        let json = outcome.ranges_json();
        assert!(json.as_object().unwrap().is_empty());
    }

    #[test]
    fn ranges_json_serializes_correctly() {
        let mut outcome = SeedOutcome {
            ranges: HashMap::new(),
            seed_run_id: None,
            errors: vec![],
            warnings: vec![],
        };
        outcome
            .ranges
            .insert("posts".to_string(), TableRange { from: 5, to: 15 });
        outcome
            .ranges
            .insert("tags".to_string(), TableRange { from: 1, to: 3 });

        let json = outcome.ranges_json();
        let serialized = serde_json::to_string(&json).unwrap();
        // Verify it is valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&serialized).unwrap();
        assert_eq!(parsed["posts"]["from"], 5);
        assert_eq!(parsed["posts"]["to"], 15);
        assert_eq!(parsed["tags"]["from"], 1);
        assert_eq!(parsed["tags"]["to"], 3);
    }
}
