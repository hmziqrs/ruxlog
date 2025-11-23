use serde::{Deserialize, Serialize};

/// Seed mode for controlling data generation randomness
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum SeedMode {
    /// Random seed based on current timestamp - unique data each run
    Random,
    /// Static seed with specific value - reproducible data
    Static { value: u64 },
}

impl Default for SeedMode {
    fn default() -> Self {
        Self::Random
    }
}

impl SeedMode {
    /// Get the seed value as u64
    pub fn to_seed(&self) -> u64 {
        match self {
            Self::Random => chrono::Utc::now().timestamp_millis() as u64,
            Self::Static { value } => *value,
        }
    }
}

/// Named seed preset for reproducible data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedPreset {
    pub name: String,
    pub seed: u64,
    pub description: String,
}

/// Target categories for individual seeders
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CustomSeedTarget {
    Posts,
    PostComments,
    CommentFlags,
    PostViews,
}

impl CustomSeedTarget {
    pub fn label(&self) -> &'static str {
        match self {
            CustomSeedTarget::Posts => "Posts",
            CustomSeedTarget::PostComments => "Post comments",
            CustomSeedTarget::CommentFlags => "Comment flags",
            CustomSeedTarget::PostViews => "Post views",
        }
    }
}

/// Size presets for individual seeders
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SeedSizePreset {
    Low,
    Default,
    Medium,
    Large,
    VeryLarge,
    Massive,
}

impl SeedSizePreset {
    pub fn label(&self) -> &'static str {
        match self {
            SeedSizePreset::Low => "Low",
            SeedSizePreset::Default => "Default",
            SeedSizePreset::Medium => "Medium",
            SeedSizePreset::Large => "Large",
            SeedSizePreset::VeryLarge => "Very large",
            SeedSizePreset::Massive => "Massive",
        }
    }

    /// Get a count for a given target based on the preset.
    pub fn count_for_target(&self, target: CustomSeedTarget) -> u32 {
        match target {
            CustomSeedTarget::Posts => match self {
                SeedSizePreset::Low => 10,
                SeedSizePreset::Default => 25,
                SeedSizePreset::Medium => 50,
                SeedSizePreset::Large => 100,
                SeedSizePreset::VeryLarge => 250,
                SeedSizePreset::Massive => 500,
            },
            CustomSeedTarget::PostComments => match self {
                SeedSizePreset::Low => 25,
                SeedSizePreset::Default => 60,
                SeedSizePreset::Medium => 120,
                SeedSizePreset::Large => 240,
                SeedSizePreset::VeryLarge => 500,
                SeedSizePreset::Massive => 1000,
            },
            CustomSeedTarget::CommentFlags => match self {
                SeedSizePreset::Low => 10,
                SeedSizePreset::Default => 25,
                SeedSizePreset::Medium => 60,
                SeedSizePreset::Large => 120,
                SeedSizePreset::VeryLarge => 240,
                SeedSizePreset::Massive => 480,
            },
            CustomSeedTarget::PostViews => match self {
                SeedSizePreset::Low => 200,
                SeedSizePreset::Default => 500,
                SeedSizePreset::Medium => 1200,
                SeedSizePreset::Large => 2500,
                SeedSizePreset::VeryLarge => 5000,
                SeedSizePreset::Massive => 10000,
            },
        }
    }
}

/// Get all available seed presets
pub fn list_presets() -> Vec<SeedPreset> {
    vec![
        SeedPreset {
            name: "demo".to_string(),
            seed: 1000,
            description: "Consistent seed for demos and screenshots".to_string(),
        },
        SeedPreset {
            name: "test".to_string(),
            seed: 2000,
            description: "Consistent seed for testing and QA".to_string(),
        },
        SeedPreset {
            name: "showcase".to_string(),
            seed: 3000,
            description: "Consistent seed for presentations and showcases".to_string(),
        },
        SeedPreset {
            name: "development".to_string(),
            seed: 4000,
            description: "Consistent seed for development and debugging".to_string(),
        },
    ]
}

/// Convert preset name to seed value
pub fn preset_to_seed(name: &str) -> Option<u64> {
    match name.to_lowercase().as_str() {
        "demo" => Some(1000),
        "test" => Some(2000),
        "showcase" => Some(3000),
        "development" => Some(4000),
        _ => None,
    }
}

/// Get preset by name
pub fn get_preset(name: &str) -> Option<SeedPreset> {
    list_presets()
        .into_iter()
        .find(|p| p.name.to_lowercase() == name.to_lowercase())
}
