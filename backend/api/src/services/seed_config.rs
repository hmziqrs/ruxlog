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
    Users,
    Categories,
    Tags,
    Posts,
    PostComments,
    CommentFlags,
    PostViews,
    UserSessions,
    EmailVerifications,
    ForgotPasswords,
    PostRevisions,
    PostSeries,
    ScheduledPosts,
    Media,
    MediaVariants,
    MediaUsage,
    NewsletterSubscribers,
    RouteStatus,
}

impl CustomSeedTarget {
    pub fn label(&self) -> &'static str {
        match self {
            CustomSeedTarget::Users => "Users",
            CustomSeedTarget::Categories => "Categories",
            CustomSeedTarget::Tags => "Tags",
            CustomSeedTarget::Posts => "Posts",
            CustomSeedTarget::PostComments => "Post comments",
            CustomSeedTarget::CommentFlags => "Comment flags",
            CustomSeedTarget::PostViews => "Post views",
            CustomSeedTarget::UserSessions => "User sessions",
            CustomSeedTarget::EmailVerifications => "Email verifications",
            CustomSeedTarget::ForgotPasswords => "Forgot passwords",
            CustomSeedTarget::PostRevisions => "Post revisions",
            CustomSeedTarget::PostSeries => "Post series",
            CustomSeedTarget::ScheduledPosts => "Scheduled posts",
            CustomSeedTarget::Media => "Media",
            CustomSeedTarget::MediaVariants => "Media variants",
            CustomSeedTarget::MediaUsage => "Media usage",
            CustomSeedTarget::NewsletterSubscribers => "Newsletter subscribers",
            CustomSeedTarget::RouteStatus => "Route status",
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
            CustomSeedTarget::Users => match self {
                SeedSizePreset::Low => 10,
                SeedSizePreset::Default => 25,
                SeedSizePreset::Medium => 50,
                SeedSizePreset::Large => 100,
                SeedSizePreset::VeryLarge => 200,
                SeedSizePreset::Massive => 400,
            },
            CustomSeedTarget::Categories => match self {
                SeedSizePreset::Low => 5,
                SeedSizePreset::Default => 10,
                SeedSizePreset::Medium => 20,
                SeedSizePreset::Large => 40,
                SeedSizePreset::VeryLarge => 80,
                SeedSizePreset::Massive => 120,
            },
            CustomSeedTarget::Tags => match self {
                SeedSizePreset::Low => 10,
                SeedSizePreset::Default => 25,
                SeedSizePreset::Medium => 50,
                SeedSizePreset::Large => 100,
                SeedSizePreset::VeryLarge => 200,
                SeedSizePreset::Massive => 400,
            },
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
            CustomSeedTarget::UserSessions => match self {
                SeedSizePreset::Low => 20,
                SeedSizePreset::Default => 50,
                SeedSizePreset::Medium => 100,
                SeedSizePreset::Large => 200,
                SeedSizePreset::VeryLarge => 400,
                SeedSizePreset::Massive => 800,
            },
            CustomSeedTarget::EmailVerifications => match self {
                SeedSizePreset::Low => 20,
                SeedSizePreset::Default => 50,
                SeedSizePreset::Medium => 100,
                SeedSizePreset::Large => 200,
                SeedSizePreset::VeryLarge => 400,
                SeedSizePreset::Massive => 800,
            },
            CustomSeedTarget::ForgotPasswords => match self {
                SeedSizePreset::Low => 20,
                SeedSizePreset::Default => 50,
                SeedSizePreset::Medium => 100,
                SeedSizePreset::Large => 200,
                SeedSizePreset::VeryLarge => 400,
                SeedSizePreset::Massive => 800,
            },
            CustomSeedTarget::PostRevisions => match self {
                SeedSizePreset::Low => 20,
                SeedSizePreset::Default => 50,
                SeedSizePreset::Medium => 100,
                SeedSizePreset::Large => 200,
                SeedSizePreset::VeryLarge => 400,
                SeedSizePreset::Massive => 800,
            },
            CustomSeedTarget::PostSeries => match self {
                SeedSizePreset::Low => 5,
                SeedSizePreset::Default => 10,
                SeedSizePreset::Medium => 20,
                SeedSizePreset::Large => 40,
                SeedSizePreset::VeryLarge => 80,
                SeedSizePreset::Massive => 120,
            },
            CustomSeedTarget::ScheduledPosts => match self {
                SeedSizePreset::Low => 10,
                SeedSizePreset::Default => 25,
                SeedSizePreset::Medium => 50,
                SeedSizePreset::Large => 100,
                SeedSizePreset::VeryLarge => 200,
                SeedSizePreset::Massive => 400,
            },
            CustomSeedTarget::Media => match self {
                SeedSizePreset::Low => 20,
                SeedSizePreset::Default => 50,
                SeedSizePreset::Medium => 100,
                SeedSizePreset::Large => 200,
                SeedSizePreset::VeryLarge => 400,
                SeedSizePreset::Massive => 800,
            },
            CustomSeedTarget::MediaVariants => match self {
                SeedSizePreset::Low => 50,
                SeedSizePreset::Default => 120,
                SeedSizePreset::Medium => 240,
                SeedSizePreset::Large => 480,
                SeedSizePreset::VeryLarge => 960,
                SeedSizePreset::Massive => 1800,
            },
            CustomSeedTarget::MediaUsage => match self {
                SeedSizePreset::Low => 20,
                SeedSizePreset::Default => 50,
                SeedSizePreset::Medium => 100,
                SeedSizePreset::Large => 200,
                SeedSizePreset::VeryLarge => 400,
                SeedSizePreset::Massive => 800,
            },
            CustomSeedTarget::NewsletterSubscribers => match self {
                SeedSizePreset::Low => 50,
                SeedSizePreset::Default => 120,
                SeedSizePreset::Medium => 240,
                SeedSizePreset::Large => 480,
                SeedSizePreset::VeryLarge => 960,
                SeedSizePreset::Massive => 1800,
            },
            CustomSeedTarget::RouteStatus => match self {
                SeedSizePreset::Low => 5,
                SeedSizePreset::Default => 10,
                SeedSizePreset::Medium => 20,
                SeedSizePreset::Large => 40,
                SeedSizePreset::VeryLarge => 80,
                SeedSizePreset::Massive => 120,
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
