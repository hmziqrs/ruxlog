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
