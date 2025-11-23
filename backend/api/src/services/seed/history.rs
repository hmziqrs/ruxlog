use std::collections::HashMap;

use sea_orm::{DatabaseConnection, EntityTrait, QueryOrder};

use crate::db::sea_models::seed_run;

use super::types::{SeedResult, TableRange};

#[derive(Debug, Clone)]
pub struct SeedOutcomeRow {
    pub id: i32,
    pub key: String,
    pub ran_at: chrono::DateTime<chrono::FixedOffset>,
    pub ranges: HashMap<String, TableRange>,
    pub counts: HashMap<String, i32>,
}

pub async fn list_seed_runs(db: &DatabaseConnection) -> SeedResult<Vec<SeedOutcomeRow>> {
    let runs = seed_run::Entity::find()
        .order_by_desc(seed_run::Column::RanAt)
        .all(db)
        .await?;

    let mut rows = Vec::new();
    for r in runs {
        let ranges_map: HashMap<String, TableRange> =
            serde_json::from_value(r.ranges.clone()).unwrap_or_default();
        let counts = ranges_map
            .iter()
            .map(|(k, v)| {
                let count = if v.to > 0 && v.to >= v.from {
                    v.to - v.from + 1
                } else {
                    0
                };
                (k.clone(), count)
            })
            .collect();
        rows.push(SeedOutcomeRow {
            id: r.id,
            key: r.key,
            ran_at: r.ran_at,
            ranges: ranges_map,
            counts,
        });
    }
    Ok(rows)
}
