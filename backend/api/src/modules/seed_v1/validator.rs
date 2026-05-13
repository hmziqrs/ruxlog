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

#[cfg(test)]
mod tests {
    use super::*;

    // ── V1SeedPayload::to_seed_mode ───────────────────────────────────────

    #[test]
    fn to_seed_mode_none_is_random() {
        let payload = V1SeedPayload {
            seed_mode: None,
            seed_value: None,
            preset_name: None,
        };
        let result = payload.to_seed_mode().unwrap();
        assert!(matches!(result, SeedMode::Random));
    }

    #[test]
    fn to_seed_mode_random_explicit() {
        let payload = V1SeedPayload {
            seed_mode: Some("random".to_string()),
            seed_value: None,
            preset_name: None,
        };
        let result = payload.to_seed_mode().unwrap();
        assert!(matches!(result, SeedMode::Random));
    }

    #[test]
    fn to_seed_mode_static_with_value() {
        let payload = V1SeedPayload {
            seed_mode: Some("static".to_string()),
            seed_value: Some(42),
            preset_name: None,
        };
        let result = payload.to_seed_mode().unwrap();
        assert_eq!(result, SeedMode::Static { value: 42 });
    }

    #[test]
    fn to_seed_mode_static_missing_value() {
        let payload = V1SeedPayload {
            seed_mode: Some("static".to_string()),
            seed_value: None,
            preset_name: None,
        };
        let err = payload.to_seed_mode().unwrap_err();
        assert!(err.contains("seed_value required"));
    }

    #[test]
    fn to_seed_mode_preset_demo() {
        let payload = V1SeedPayload {
            seed_mode: Some("preset".to_string()),
            seed_value: None,
            preset_name: Some("demo".to_string()),
        };
        let result = payload.to_seed_mode().unwrap();
        assert_eq!(result, SeedMode::Static { value: 1000 });
    }

    #[test]
    fn to_seed_mode_preset_test() {
        let payload = V1SeedPayload {
            seed_mode: Some("preset".to_string()),
            seed_value: None,
            preset_name: Some("test".to_string()),
        };
        let result = payload.to_seed_mode().unwrap();
        assert_eq!(result, SeedMode::Static { value: 2000 });
    }

    #[test]
    fn to_seed_mode_preset_showcase() {
        let payload = V1SeedPayload {
            seed_mode: Some("preset".to_string()),
            seed_value: None,
            preset_name: Some("showcase".to_string()),
        };
        let result = payload.to_seed_mode().unwrap();
        assert_eq!(result, SeedMode::Static { value: 3000 });
    }

    #[test]
    fn to_seed_mode_preset_development() {
        let payload = V1SeedPayload {
            seed_mode: Some("preset".to_string()),
            seed_value: None,
            preset_name: Some("development".to_string()),
        };
        let result = payload.to_seed_mode().unwrap();
        assert_eq!(result, SeedMode::Static { value: 4000 });
    }

    #[test]
    fn to_seed_mode_preset_missing_name() {
        let payload = V1SeedPayload {
            seed_mode: Some("preset".to_string()),
            seed_value: None,
            preset_name: None,
        };
        let err = payload.to_seed_mode().unwrap_err();
        assert!(err.contains("preset_name required"));
    }

    #[test]
    fn to_seed_mode_preset_unknown_name() {
        let payload = V1SeedPayload {
            seed_mode: Some("preset".to_string()),
            seed_value: None,
            preset_name: Some("nonexistent".to_string()),
        };
        let err = payload.to_seed_mode().unwrap_err();
        assert!(err.contains("Unknown preset"));
        assert!(err.contains("nonexistent"));
    }

    #[test]
    fn to_seed_mode_invalid_mode() {
        let payload = V1SeedPayload {
            seed_mode: Some("invalid_mode".to_string()),
            seed_value: None,
            preset_name: None,
        };
        let err = payload.to_seed_mode().unwrap_err();
        assert!(err.contains("Invalid seed_mode"));
        assert!(err.contains("invalid_mode"));
    }
}
