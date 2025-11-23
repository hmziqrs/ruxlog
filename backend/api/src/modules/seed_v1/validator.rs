use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::services::seed_config::{preset_to_seed, SeedMode};

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct V1SeedPayload {
    /// Seed mode: "random" or "static" or "preset"
    pub seed_mode: Option<String>,
    /// Seed value for static mode
    pub seed_value: Option<u64>,
    /// Preset name for preset mode (demo, test, showcase, development)
    pub preset_name: Option<String>,
}

impl V1SeedPayload {
    /// Convert payload to SeedMode
    pub fn to_seed_mode(&self) -> Result<SeedMode, String> {
        match self.seed_mode.as_deref() {
            None | Some("random") => Ok(SeedMode::Random),
            Some("static") => {
                let value = self
                    .seed_value
                    .ok_or_else(|| "seed_value required for static mode".to_string())?;
                Ok(SeedMode::Static { value })
            }
            Some("preset") => {
                let name = self
                    .preset_name
                    .as_deref()
                    .ok_or_else(|| "preset_name required for preset mode".to_string())?;
                let value = preset_to_seed(name).ok_or_else(|| {
                    format!(
                        "Unknown preset '{}'. Available: demo, test, showcase, development",
                        name
                    )
                })?;
                Ok(SeedMode::Static { value })
            }
            Some(mode) => Err(format!(
                "Invalid seed_mode '{}'. Use: random, static, or preset",
                mode
            )),
        }
    }
}
