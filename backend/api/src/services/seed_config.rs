use serde::{Deserialize, Serialize};

/// Seed mode for controlling data generation randomness
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum SeedMode {
    /// Random seed based on current timestamp - unique data each run
    #[default]
    Random,
    /// Static seed with specific value - reproducible data
    Static { value: u64 },
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

#[cfg(test)]
mod tests {
    use super::*;

    // ── SeedMode::to_seed ──────────────────────────────────────────────

    #[test]
    fn seed_mode_random_returns_non_zero() {
        let mode = SeedMode::Random;
        let seed = mode.to_seed();
        // A timestamp-based seed must be non-zero in any realistic scenario.
        assert_ne!(seed, 0);
    }

    #[test]
    fn seed_mode_static_returns_exact_value() {
        let mode = SeedMode::Static { value: 42 };
        assert_eq!(mode.to_seed(), 42);
    }

    #[test]
    fn seed_mode_static_zero() {
        let mode = SeedMode::Static { value: 0 };
        assert_eq!(mode.to_seed(), 0);
    }

    // ── CustomSeedTarget::label ────────────────────────────────────────

    #[test]
    fn custom_seed_target_labels() {
        assert_eq!(CustomSeedTarget::Users.label(), "Users");
        assert_eq!(CustomSeedTarget::Categories.label(), "Categories");
        assert_eq!(CustomSeedTarget::Tags.label(), "Tags");
        assert_eq!(CustomSeedTarget::Posts.label(), "Posts");
        assert_eq!(CustomSeedTarget::PostComments.label(), "Post comments");
        assert_eq!(CustomSeedTarget::CommentFlags.label(), "Comment flags");
        assert_eq!(CustomSeedTarget::PostViews.label(), "Post views");
        assert_eq!(CustomSeedTarget::UserSessions.label(), "User sessions");
        assert_eq!(
            CustomSeedTarget::EmailVerifications.label(),
            "Email verifications"
        );
        assert_eq!(
            CustomSeedTarget::ForgotPasswords.label(),
            "Forgot passwords"
        );
        assert_eq!(CustomSeedTarget::PostRevisions.label(), "Post revisions");
        assert_eq!(CustomSeedTarget::PostSeries.label(), "Post series");
        assert_eq!(
            CustomSeedTarget::ScheduledPosts.label(),
            "Scheduled posts"
        );
        assert_eq!(CustomSeedTarget::Media.label(), "Media");
        assert_eq!(CustomSeedTarget::MediaVariants.label(), "Media variants");
        assert_eq!(CustomSeedTarget::MediaUsage.label(), "Media usage");
        assert_eq!(
            CustomSeedTarget::NewsletterSubscribers.label(),
            "Newsletter subscribers"
        );
        assert_eq!(CustomSeedTarget::RouteStatus.label(), "Route status");
    }

    // ── SeedSizePreset::label ──────────────────────────────────────────

    #[test]
    fn seed_size_preset_labels() {
        assert_eq!(SeedSizePreset::Low.label(), "Low");
        assert_eq!(SeedSizePreset::Default.label(), "Default");
        assert_eq!(SeedSizePreset::Medium.label(), "Medium");
        assert_eq!(SeedSizePreset::Large.label(), "Large");
        assert_eq!(SeedSizePreset::VeryLarge.label(), "Very large");
        assert_eq!(SeedSizePreset::Massive.label(), "Massive");
    }

    // ── SeedSizePreset::count_for_target ───────────────────────────────

    #[test]
    fn count_for_target_users() {
        assert_eq!(SeedSizePreset::Low.count_for_target(CustomSeedTarget::Users), 10);
        assert_eq!(SeedSizePreset::Default.count_for_target(CustomSeedTarget::Users), 25);
        assert_eq!(SeedSizePreset::Massive.count_for_target(CustomSeedTarget::Users), 400);
    }

    #[test]
    fn count_for_target_posts() {
        assert_eq!(SeedSizePreset::Low.count_for_target(CustomSeedTarget::Posts), 10);
        assert_eq!(SeedSizePreset::Massive.count_for_target(CustomSeedTarget::Posts), 500);
    }

    #[test]
    fn count_for_target_post_views() {
        assert_eq!(SeedSizePreset::Low.count_for_target(CustomSeedTarget::PostViews), 200);
        assert_eq!(SeedSizePreset::Massive.count_for_target(CustomSeedTarget::PostViews), 10000);
    }

    #[test]
    fn count_for_target_categories() {
        assert_eq!(SeedSizePreset::Low.count_for_target(CustomSeedTarget::Categories), 5);
        assert_eq!(SeedSizePreset::Massive.count_for_target(CustomSeedTarget::Categories), 120);
    }

    #[test]
    fn count_for_target_newsletter_subscribers() {
        assert_eq!(
            SeedSizePreset::Low.count_for_target(CustomSeedTarget::NewsletterSubscribers),
            50
        );
        assert_eq!(
            SeedSizePreset::Massive.count_for_target(CustomSeedTarget::NewsletterSubscribers),
            1800
        );
    }

    // ── list_presets ───────────────────────────────────────────────────

    #[test]
    fn list_presets_returns_four() {
        let presets = list_presets();
        assert_eq!(presets.len(), 4);
    }

    #[test]
    fn list_presets_names_and_seeds() {
        let presets = list_presets();
        let names: Vec<&str> = presets.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"demo"));
        assert!(names.contains(&"test"));
        assert!(names.contains(&"showcase"));
        assert!(names.contains(&"development"));

        for p in &presets {
            assert!(!p.description.is_empty());
            assert_ne!(p.seed, 0);
        }

        // Spot-check specific seeds
        let demo = presets.iter().find(|p| p.name == "demo").unwrap();
        assert_eq!(demo.seed, 1000);
        let dev = presets.iter().find(|p| p.name == "development").unwrap();
        assert_eq!(dev.seed, 4000);
    }

    // ── preset_to_seed ─────────────────────────────────────────────────

    #[test]
    fn preset_to_seed_known_names() {
        assert_eq!(preset_to_seed("demo"), Some(1000));
        assert_eq!(preset_to_seed("test"), Some(2000));
        assert_eq!(preset_to_seed("showcase"), Some(3000));
        assert_eq!(preset_to_seed("development"), Some(4000));
    }

    #[test]
    fn preset_to_seed_case_insensitive() {
        assert_eq!(preset_to_seed("DEMO"), Some(1000));
        assert_eq!(preset_to_seed("Test"), Some(2000));
    }

    #[test]
    fn preset_to_seed_unknown_returns_none() {
        assert_eq!(preset_to_seed("nonexistent"), None);
        assert_eq!(preset_to_seed(""), None);
    }

    // ── get_preset ─────────────────────────────────────────────────────

    #[test]
    fn get_preset_known_returns_correct_struct() {
        let preset = get_preset("demo").expect("demo preset should exist");
        assert_eq!(preset.name, "demo");
        assert_eq!(preset.seed, 1000);
        assert_eq!(preset.description, "Consistent seed for demos and screenshots");
    }

    #[test]
    fn get_preset_case_insensitive() {
        let preset = get_preset("SHOWCASE").expect("showcase preset should exist");
        assert_eq!(preset.name, "showcase");
        assert_eq!(preset.seed, 3000);
    }

    #[test]
    fn get_preset_unknown_returns_none() {
        assert!(get_preset("nope").is_none());
        assert!(get_preset("").is_none());
    }
}
