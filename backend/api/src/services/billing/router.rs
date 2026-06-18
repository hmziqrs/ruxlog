//! Multi-provider billing router with geo-based routing.
//!
//! `BillingRouter` holds all initialized providers and delegates to the
//! correct one based on client geography (for checkouts) or provider name
//! (for webhooks, subscription management). It implements `BillingProvider`
//! so existing controller code works transparently.

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;

use async_trait::async_trait;
use maxminddb::Reader as MaxMindReader;
use serde::Deserialize;

use super::provider::{
    BillingError, BillingProvider, CheckoutSession, ParsedWebhook, SubscriptionInfo, WebhookEvent,
};

// ── Config types ──────────────────────────────────────────────────────────

/// A single routing rule: provider + geographic filters.
///
/// Rules are evaluated in order. First match wins.
/// Include/exclude lists use ISO 3166-1 alpha-2 country codes (IN, BR, US)
/// and two-letter continent codes (AF, AS, EU, NA, OC, SA, AN).
///
/// If `include_countries` or `include_continents` is non-empty, only those
/// match. `exclude_*` removes entries from the included set.
#[derive(Debug, Clone, Deserialize)]
pub struct RoutingRule {
    pub provider: String,
    #[serde(default)]
    pub include_countries: Vec<String>,
    #[serde(default)]
    pub exclude_countries: Vec<String>,
    #[serde(default)]
    pub include_continents: Vec<String>,
    #[serde(default)]
    pub exclude_continents: Vec<String>,
}

/// Top-level JSON config for `BILLING_GEO_RULES` env var.
#[derive(Debug, Clone, Deserialize)]
pub struct GeoRulesConfig {
    pub default_provider: String,
    #[serde(default)]
    pub rules: Vec<RoutingRule>,
}

impl GeoRulesConfig {
    /// Load from `BILLING_GEO_RULES` env var (inline JSON) or `BILLING_GEO_RULES_FILE` (file path).
    /// Returns a minimal passthrough config if neither is set.
    pub fn from_env() -> Self {
        if let Ok(json) = std::env::var("BILLING_GEO_RULES") {
            return serde_json::from_str(&json).expect("Failed to parse BILLING_GEO_RULES JSON");
        }

        if let Ok(path) = std::env::var("BILLING_GEO_RULES_FILE") {
            let contents = std::fs::read_to_string(&path).unwrap_or_else(|e| {
                panic!("Failed to read BILLING_GEO_RULES_FILE '{}': {}", path, e)
            });
            return serde_json::from_str(&contents)
                .expect("Failed to parse BILLING_GEO_RULES_FILE JSON");
        }

        // No geo rules configured — use BILLING_PROVIDER or "stripe" as default
        let default = std::env::var("BILLING_PROVIDER").unwrap_or_else(|_| "stripe".to_string());
        Self {
            default_provider: default,
            rules: vec![],
        }
    }
}

// ── Geo lookup ────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct GeoInfo {
    country_code: Option<String>,
    continent_code: Option<String>,
}

/// Resolves an IP to a provider name using MaxMind + routing rules.
pub struct GeoRouter {
    reader: Option<MaxMindReader<Vec<u8>>>,
    rules: Vec<RoutingRule>,
    default_provider: String,
}

impl GeoRouter {
    pub fn new(config: GeoRulesConfig) -> Self {
        let reader = match std::env::var("GEOLITE2_DB_PATH") {
            Ok(path) if !path.is_empty() => match MaxMindReader::open_readfile(&path) {
                Ok(r) => {
                    tracing::info!(path = %path, "GeoLite2 database loaded");
                    Some(r)
                }
                Err(e) => {
                    tracing::warn!(path = %path, error = %e, "Failed to load GeoLite2 database, geo-routing disabled");
                    None
                }
            },
            _ => {
                tracing::info!(
                    "GEOLITE2_DB_PATH not set, geo-routing disabled (using default provider)"
                );
                None
            }
        };

        Self {
            reader,
            rules: config.rules,
            default_provider: config.default_provider,
        }
    }

    /// Resolve an IP to a provider name. Returns the default if no rule matches
    /// or if the matched provider isn't available.
    pub fn resolve(
        &self,
        ip: IpAddr,
        available_providers: &HashMap<String, Arc<dyn BillingProvider>>,
    ) -> String {
        let geo = self.lookup_geo(ip);

        for rule in &self.rules {
            if !available_providers.contains_key(&rule.provider) {
                continue;
            }
            if self.rule_matches(rule, &geo) {
                tracing::debug!(
                    ip = %ip,
                    provider = %rule.provider,
                    country = ?geo.as_ref().and_then(|g| g.country_code.as_deref()),
                    continent = ?geo.as_ref().and_then(|g| g.continent_code.as_deref()),
                    "Geo rule matched"
                );
                return rule.provider.clone();
            }
        }

        self.default_provider.clone()
    }

    fn lookup_geo(&self, ip: IpAddr) -> Option<GeoInfo> {
        let reader = self.reader.as_ref()?;
        // maxminddb 0.27 split lookup into `lookup()` (returns a LookupResult
        // handle) + `decode::<T>()` (materialises the typed record). We resolve
        // both errors (corrupt/truncated DB, decode failure) and missing-data
        // (None) to a plain `None` — geo routing always falls back gracefully.
        let result = reader.lookup(ip).ok()?;
        let geoip: maxminddb::geoip2::Country<'_> = result.decode().ok()??;

        let country_code = geoip.country.iso_code.map(String::from);
        let continent_code = geoip.continent.code.map(String::from);

        Some(GeoInfo {
            country_code,
            continent_code,
        })
    }

    fn rule_matches(&self, rule: &RoutingRule, geo: &Option<GeoInfo>) -> bool {
        let geo = match geo {
            Some(g) => g,
            None => return false,
        };

        // Continent include/exclude
        if !rule.include_continents.is_empty() {
            match &geo.continent_code {
                Some(cc) if rule.include_continents.iter().any(|c| c == cc) => {}
                _ => return false,
            }
        }
        if !rule.exclude_continents.is_empty() {
            if let Some(ref cc) = geo.continent_code {
                if rule.exclude_continents.iter().any(|c| c == cc) {
                    return false;
                }
            }
        }

        // Country include/exclude
        if !rule.include_countries.is_empty() {
            match &geo.country_code {
                Some(cc) if rule.include_countries.iter().any(|c| c == cc) => {}
                _ => return false,
            }
        }
        if !rule.exclude_countries.is_empty() {
            if let Some(ref cc) = geo.country_code {
                if rule.exclude_countries.iter().any(|c| c == cc) {
                    return false;
                }
            }
        }

        true
    }
}

// ── BillingRouter ─────────────────────────────────────────────────────────

/// Holds all initialized providers and routes requests to the correct one.
pub struct BillingRouter {
    providers: HashMap<String, Arc<dyn BillingProvider>>,
    geo_router: GeoRouter,
    default_provider: String,
}

impl BillingRouter {
    pub fn new(
        providers: HashMap<String, Arc<dyn BillingProvider>>,
        geo_router: GeoRouter,
    ) -> Self {
        let default_provider = geo_router.default_provider.clone();
        Self {
            providers,
            geo_router,
            default_provider,
        }
    }

    /// Geo-routed checkout: selects provider based on client IP.
    pub async fn create_checkout_for_ip(
        &self,
        client_ip: IpAddr,
        plan_slug: &str,
        customer_email: &str,
        user_id: i32,
        success_url: &str,
        cancel_url: &str,
    ) -> Result<CheckoutSession, BillingError> {
        let provider_name = self.geo_router.resolve(client_ip, &self.providers);
        let provider = self.get_provider(&provider_name)?;
        tracing::info!(
            ip = %client_ip,
            provider = %provider_name,
            user_id,
            "Geo-routed checkout"
        );
        provider
            .create_checkout(plan_slug, customer_email, user_id, success_url, cancel_url)
            .await
    }

    /// Geo-routed one-time checkout for a per-post purchase. Like
    /// [`create_checkout_for_ip`] but for single payments; providers that don't
    /// support one-time checkouts return `BillingError::Config` (per-post
    /// purchases are simply unavailable for those regions/providers).
    pub async fn create_post_checkout_for_ip(
        &self,
        client_ip: IpAddr,
        post_id: i32,
        amount_cents: i32,
        currency: &str,
        customer_email: &str,
        user_id: i32,
        success_url: &str,
        cancel_url: &str,
    ) -> Result<CheckoutSession, BillingError> {
        let provider_name = self.geo_router.resolve(client_ip, &self.providers);
        let provider = self.get_provider(&provider_name)?;
        tracing::info!(
            ip = %client_ip,
            provider = %provider_name,
            user_id,
            post_id,
            "Geo-routed per-post checkout"
        );
        provider
            .create_post_checkout(
                post_id,
                amount_cents,
                currency,
                customer_email,
                user_id,
                success_url,
                cancel_url,
            )
            .await
    }

    /// Cancel a subscription on a specific provider.
    pub async fn cancel_subscription_for_provider(
        &self,
        provider_name: &str,
        provider_subscription_id: &str,
        immediately: bool,
    ) -> Result<(), BillingError> {
        let provider = self.get_provider(provider_name)?;
        provider
            .cancel_subscription(provider_subscription_id, immediately)
            .await
    }

    /// Get subscription from a specific provider.
    pub async fn get_subscription_for_provider(
        &self,
        provider_name: &str,
        provider_subscription_id: &str,
    ) -> Result<SubscriptionInfo, BillingError> {
        let provider = self.get_provider(provider_name)?;
        provider.get_subscription(provider_subscription_id).await
    }

    /// Create portal session for a specific provider.
    pub async fn create_portal_session_for_provider(
        &self,
        provider_name: &str,
        provider_customer_id: &str,
        return_url: &str,
    ) -> Result<String, BillingError> {
        let provider = self.get_provider(provider_name)?;
        provider
            .create_portal_session(provider_customer_id, return_url)
            .await
    }

    fn get_provider(&self, name: &str) -> Result<&Arc<dyn BillingProvider>, BillingError> {
        self.providers
            .get(name)
            .ok_or_else(|| BillingError::Config(format!("Provider '{}' not initialized", name)))
    }

    pub fn provider_names(&self) -> Vec<&str> {
        self.providers.keys().map(|s| s.as_str()).collect()
    }

    pub fn has_provider(&self, name: &str) -> bool {
        self.providers.contains_key(name)
    }
}

#[async_trait]
impl BillingProvider for BillingRouter {
    fn provider_name(&self) -> &'static str {
        "router"
    }

    async fn create_checkout(
        &self,
        plan_slug: &str,
        customer_email: &str,
        user_id: i32,
        success_url: &str,
        cancel_url: &str,
    ) -> Result<CheckoutSession, BillingError> {
        // Fallback when no IP is available — use default provider
        let provider = self.get_provider(&self.default_provider)?;
        provider
            .create_checkout(plan_slug, customer_email, user_id, success_url, cancel_url)
            .await
    }

    async fn cancel_subscription(
        &self,
        provider_subscription_id: &str,
        immediately: bool,
    ) -> Result<(), BillingError> {
        let provider = self.get_provider(&self.default_provider)?;
        provider
            .cancel_subscription(provider_subscription_id, immediately)
            .await
    }

    async fn get_subscription(
        &self,
        provider_subscription_id: &str,
    ) -> Result<SubscriptionInfo, BillingError> {
        let provider = self.get_provider(&self.default_provider)?;
        provider.get_subscription(provider_subscription_id).await
    }

    async fn verify_webhook(&self, event: WebhookEvent) -> Result<ParsedWebhook, BillingError> {
        // Route to the provider named in the webhook event
        let provider = self.get_provider(&event.provider).map_err(|_| {
            BillingError::WebhookVerification(format!(
                "Unknown provider '{}' for webhook",
                event.provider
            ))
        })?;
        provider.verify_webhook(event).await
    }

    async fn create_portal_session(
        &self,
        provider_customer_id: &str,
        return_url: &str,
    ) -> Result<String, BillingError> {
        let provider = self.get_provider(&self.default_provider)?;
        provider
            .create_portal_session(provider_customer_id, return_url)
            .await
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::super::revolut::RevolutProvider;
    use super::super::stripe::StripeProvider;
    use super::*;

    fn make_geo_info(country: &str, continent: &str) -> Option<GeoInfo> {
        Some(GeoInfo {
            country_code: Some(country.to_string()),
            continent_code: Some(continent.to_string()),
        })
    }

    #[test]
    fn test_rule_include_country_match() {
        let rule = RoutingRule {
            provider: "razorpay".into(),
            include_countries: vec!["IN".into()],
            exclude_countries: vec![],
            include_continents: vec![],
            exclude_continents: vec![],
        };
        let router = GeoRouter::new_for_test(vec![rule], "stripe".into());
        assert!(router.rule_matches(&router.rules[0], &make_geo_info("IN", "AS")));
        assert!(!router.rule_matches(&router.rules[0], &make_geo_info("US", "NA")));
    }

    #[test]
    fn test_rule_exclude_country() {
        let rule = RoutingRule {
            provider: "razorpay".into(),
            include_countries: vec![],
            exclude_countries: vec!["CN".into()],
            include_continents: vec!["AS".into()],
            exclude_continents: vec![],
        };
        let router = GeoRouter::new_for_test(vec![rule], "stripe".into());
        assert!(router.rule_matches(&router.rules[0], &make_geo_info("IN", "AS")));
        assert!(!router.rule_matches(&router.rules[0], &make_geo_info("CN", "AS")));
    }

    #[test]
    fn test_rule_continent_only() {
        let rule = RoutingRule {
            provider: "revolut".into(),
            include_countries: vec![],
            exclude_countries: vec![],
            include_continents: vec!["EU".into()],
            exclude_continents: vec![],
        };
        let router = GeoRouter::new_for_test(vec![rule], "stripe".into());
        assert!(router.rule_matches(&router.rules[0], &make_geo_info("DE", "EU")));
        assert!(router.rule_matches(&router.rules[0], &make_geo_info("FR", "EU")));
        assert!(!router.rule_matches(&router.rules[0], &make_geo_info("US", "NA")));
    }

    #[test]
    fn test_rule_no_geo_data() {
        let rule = RoutingRule {
            provider: "stripe".into(),
            include_countries: vec!["US".into()],
            exclude_countries: vec![],
            include_continents: vec![],
            exclude_continents: vec![],
        };
        let router = GeoRouter::new_for_test(vec![rule], "stripe".into());
        assert!(!router.rule_matches(&router.rules[0], &None));
    }

    #[test]
    fn test_rule_empty_includes_match_all() {
        let rule = RoutingRule {
            provider: "stripe".into(),
            include_countries: vec![],
            exclude_countries: vec![],
            include_continents: vec![],
            exclude_continents: vec![],
        };
        let router = GeoRouter::new_for_test(vec![rule], "stripe".into());
        assert!(router.rule_matches(&router.rules[0], &make_geo_info("US", "NA")));
        assert!(router.rule_matches(&router.rules[0], &make_geo_info("IN", "AS")));
    }

    #[test]
    fn test_resolve_first_match_wins() {
        let rules = vec![
            RoutingRule {
                provider: "razorpay".into(),
                include_countries: vec!["IN".into()],
                exclude_countries: vec![],
                include_continents: vec![],
                exclude_continents: vec![],
            },
            RoutingRule {
                provider: "revolut".into(),
                include_continents: vec!["AS".into()],
                exclude_countries: vec![],
                include_countries: vec![],
                exclude_continents: vec![],
            },
        ];
        let router = GeoRouter::new_for_test(rules, "stripe".into());
        let mut providers: HashMap<String, Arc<dyn BillingProvider>> = HashMap::new();
        providers.insert(
            "razorpay".into(),
            Arc::new(StripeProvider::new("k".into(), "s".into())),
        );
        providers.insert(
            "revolut".into(),
            Arc::new(RevolutProvider::new("k".into(), "s".into())),
        );

        assert!(router.rule_matches(&router.rules[0], &make_geo_info("IN", "AS")));
    }

    #[test]
    fn test_resolve_skips_uninitialized_provider() {
        let rules = vec![RoutingRule {
            provider: "razorpay".into(),
            include_countries: vec!["IN".into()],
            exclude_countries: vec![],
            include_continents: vec![],
            exclude_continents: vec![],
        }];
        let router = GeoRouter::new_for_test(rules, "stripe".into());
        let providers: HashMap<String, Arc<dyn BillingProvider>> = HashMap::new();

        let result = router.resolve("1.2.3.4".parse().unwrap(), &providers);
        assert_eq!(result, "stripe");
    }

    #[test]
    fn test_geo_rules_config_from_json() {
        let json = r#"{"default_provider":"stripe","rules":[{"provider":"razorpay","include_countries":["IN"],"include_continents":["AS"],"exclude_countries":["CN"]}]}"#;
        let config: GeoRulesConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.default_provider, "stripe");
        assert_eq!(config.rules.len(), 1);
        assert_eq!(config.rules[0].provider, "razorpay");
        assert_eq!(config.rules[0].include_countries, vec!["IN"]);
        assert_eq!(config.rules[0].exclude_countries, vec!["CN"]);
        assert_eq!(config.rules[0].include_continents, vec!["AS"]);
    }

    impl GeoRouter {
        fn new_for_test(rules: Vec<RoutingRule>, default_provider: String) -> Self {
            let reader: Option<MaxMindReader<Vec<u8>>> = None;
            Self {
                reader,
                rules,
                default_provider,
            }
        }
    }

    #[test]
    fn test_billing_router_provider_names() {
        let mut providers: HashMap<String, Arc<dyn BillingProvider>> = HashMap::new();
        providers.insert(
            "stripe".into(),
            Arc::new(StripeProvider::new("k".into(), "s".into())),
        );
        providers.insert(
            "revolut".into(),
            Arc::new(RevolutProvider::new("k".into(), "s".into())),
        );

        let geo = GeoRouter::new_for_test(vec![], "stripe".into());
        let router = BillingRouter::new(providers, geo);

        let mut names = router.provider_names();
        names.sort();
        assert_eq!(names, vec!["revolut", "stripe"]);
    }

    #[test]
    fn test_billing_router_has_provider() {
        let mut providers: HashMap<String, Arc<dyn BillingProvider>> = HashMap::new();
        providers.insert(
            "stripe".into(),
            Arc::new(StripeProvider::new("k".into(), "s".into())),
        );

        let geo = GeoRouter::new_for_test(vec![], "stripe".into());
        let router = BillingRouter::new(providers, geo);

        assert!(router.has_provider("stripe"));
        assert!(!router.has_provider("razorpay"));
    }

    #[test]
    fn test_billing_router_get_missing_provider() {
        let providers: HashMap<String, Arc<dyn BillingProvider>> = HashMap::new();
        let geo = GeoRouter::new_for_test(vec![], "stripe".into());
        let router = BillingRouter::new(providers, geo);

        let result = router.get_provider("nonexistent");
        assert!(result.is_err());
    }

    // ── Edge case tests ──────────────────────────────────────────────────

    #[test]
    fn test_rule_include_continent_exclude_country_combo() {
        // Include all of Asia, but exclude China
        let rule = RoutingRule {
            provider: "razorpay".into(),
            include_countries: vec![],
            exclude_countries: vec!["CN".into()],
            include_continents: vec!["AS".into()],
            exclude_continents: vec![],
        };
        let router = GeoRouter::new_for_test(vec![rule], "stripe".into());
        assert!(router.rule_matches(&router.rules[0], &make_geo_info("IN", "AS")));
        assert!(router.rule_matches(&router.rules[0], &make_geo_info("JP", "AS")));
        assert!(!router.rule_matches(&router.rules[0], &make_geo_info("CN", "AS")));
        assert!(!router.rule_matches(&router.rules[0], &make_geo_info("US", "NA")));
    }

    #[test]
    fn test_rule_include_country_override_continent() {
        // Only IN is included, continent filter is empty — other AS countries excluded
        let rule = RoutingRule {
            provider: "razorpay".into(),
            include_countries: vec!["IN".into()],
            exclude_countries: vec![],
            include_continents: vec![],
            exclude_continents: vec![],
        };
        let router = GeoRouter::new_for_test(vec![rule], "stripe".into());
        assert!(router.rule_matches(&router.rules[0], &make_geo_info("IN", "AS")));
        assert!(!router.rule_matches(&router.rules[0], &make_geo_info("JP", "AS")));
    }

    #[test]
    fn test_rule_exclude_only() {
        // No includes, but exclude EU — matches everything except EU
        let rule = RoutingRule {
            provider: "global".into(),
            include_countries: vec![],
            exclude_countries: vec![],
            include_continents: vec![],
            exclude_continents: vec!["EU".into()],
        };
        let router = GeoRouter::new_for_test(vec![rule], "stripe".into());
        assert!(router.rule_matches(&router.rules[0], &make_geo_info("US", "NA")));
        assert!(router.rule_matches(&router.rules[0], &make_geo_info("IN", "AS")));
        assert!(!router.rule_matches(&router.rules[0], &make_geo_info("DE", "EU")));
    }

    #[test]
    fn test_resolve_no_rules_returns_default() {
        let router = GeoRouter::new_for_test(vec![], "stripe".into());
        let providers: HashMap<String, Arc<dyn BillingProvider>> = HashMap::new();
        let result = router.resolve("1.2.3.4".parse().unwrap(), &providers);
        assert_eq!(result, "stripe");
    }

    #[test]
    fn test_resolve_multiple_rules_fallback() {
        // Two rules for AS and SA, IP is from NA — falls back to default
        let rules = vec![
            RoutingRule {
                provider: "razorpay".into(),
                include_countries: vec![],
                exclude_countries: vec![],
                include_continents: vec!["AS".into()],
                exclude_continents: vec![],
            },
            RoutingRule {
                provider: "mercado_pago".into(),
                include_countries: vec![],
                exclude_countries: vec![],
                include_continents: vec!["SA".into()],
                exclude_continents: vec![],
            },
        ];
        let router = GeoRouter::new_for_test(rules, "stripe".into());
        let providers: HashMap<String, Arc<dyn BillingProvider>> = HashMap::new();
        let result = router.resolve("8.8.8.8".parse().unwrap(), &providers);
        assert_eq!(result, "stripe");
    }

    #[test]
    fn test_geo_rules_config_empty_rules() {
        let json = r#"{"default_provider":"polar","rules":[]}"#;
        let config: GeoRulesConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.default_provider, "polar");
        assert!(config.rules.is_empty());
    }

    #[test]
    fn test_geo_rules_config_defaults_empty() {
        let json = r#"{"default_provider":"stripe"}"#;
        let config: GeoRulesConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.default_provider, "stripe");
        assert!(config.rules.is_empty());
    }

    #[test]
    fn test_geo_rules_config_multiple_rules() {
        let json = r#"{"default_provider":"stripe","rules":[
            {"provider":"razorpay","include_countries":["IN"]},
            {"provider":"revolut","include_continents":["EU"]},
            {"provider":"mercado_pago","include_continents":["SA"]},
            {"provider":"airwallex","include_countries":["AU","NZ","SG"]}
        ]}"#;
        let config: GeoRulesConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.rules.len(), 4);
        assert_eq!(config.rules[0].provider, "razorpay");
        assert_eq!(config.rules[3].include_countries, vec!["AU", "NZ", "SG"]);
    }

    #[test]
    fn test_routing_rule_all_fields_default_empty() {
        let json = r#"{"provider":"test"}"#;
        let rule: RoutingRule = serde_json::from_str(json).unwrap();
        assert_eq!(rule.provider, "test");
        assert!(rule.include_countries.is_empty());
        assert!(rule.exclude_countries.is_empty());
        assert!(rule.include_continents.is_empty());
        assert!(rule.exclude_continents.is_empty());
    }

    #[test]
    fn test_billing_router_empty_providers() {
        let providers: HashMap<String, Arc<dyn BillingProvider>> = HashMap::new();
        let geo = GeoRouter::new_for_test(vec![], "stripe".into());
        let router = BillingRouter::new(providers, geo);

        assert!(router.provider_names().is_empty());
        assert!(!router.has_provider("stripe"));
        assert!(!router.has_provider("anything"));
    }

    #[tokio::test]
    async fn test_billing_router_trait_cancel_with_default() {
        let mut providers: HashMap<String, Arc<dyn BillingProvider>> = HashMap::new();
        providers.insert(
            "stripe".into(),
            Arc::new(StripeProvider::new("k".into(), "s".into())),
        );
        let geo = GeoRouter::new_for_test(vec![], "stripe".into());
        let router = BillingRouter::new(providers, geo);

        // Will fail because no real server, but should route to stripe (not crash)
        let result = router.cancel_subscription("sub_test", true).await;
        assert!(result.is_err());
        // Verify it's a ProviderApi error (attempted to call stripe)
        assert!(matches!(result.unwrap_err(), BillingError::ProviderApi(_)));
    }

    #[tokio::test]
    async fn test_billing_router_trait_get_subscription_with_default() {
        let mut providers: HashMap<String, Arc<dyn BillingProvider>> = HashMap::new();
        providers.insert(
            "stripe".into(),
            Arc::new(StripeProvider::new("k".into(), "s".into())),
        );
        let geo = GeoRouter::new_for_test(vec![], "stripe".into());
        let router = BillingRouter::new(providers, geo);

        let result = router.get_subscription("sub_test").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BillingError::ProviderApi(_)));
    }

    #[tokio::test]
    async fn test_billing_router_trait_portal_with_default() {
        let mut providers: HashMap<String, Arc<dyn BillingProvider>> = HashMap::new();
        providers.insert(
            "stripe".into(),
            Arc::new(StripeProvider::new("k".into(), "s".into())),
        );
        let geo = GeoRouter::new_for_test(vec![], "stripe".into());
        let router = BillingRouter::new(providers, geo);

        let result = router.create_portal_session("cus_1", "https://app.com").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BillingError::ProviderApi(_)));
    }

    #[tokio::test]
    async fn test_billing_router_cancel_for_specific_provider() {
        let mut providers: HashMap<String, Arc<dyn BillingProvider>> = HashMap::new();
        providers.insert(
            "revolut".into(),
            Arc::new(RevolutProvider::new("k".into(), "s".into())),
        );
        let geo = GeoRouter::new_for_test(vec![], "stripe".into());
        let router = BillingRouter::new(providers, geo);

        // Should route to revolut, not default (stripe which doesn't exist)
        let result = router.cancel_subscription_for_provider("revolut", "sub_1", true).await;
        assert!(result.is_err()); // No server, but routed correctly
        assert!(matches!(result.unwrap_err(), BillingError::ProviderApi(_)));
    }

    #[tokio::test]
    async fn test_billing_router_cancel_for_uninitialized_provider() {
        let providers: HashMap<String, Arc<dyn BillingProvider>> = HashMap::new();
        let geo = GeoRouter::new_for_test(vec![], "stripe".into());
        let router = BillingRouter::new(providers, geo);

        let result = router.cancel_subscription_for_provider("razorpay", "sub_1", true).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BillingError::Config(_)));
    }

    #[test]
    fn test_billing_router_provider_name_is_router() {
        let providers: HashMap<String, Arc<dyn BillingProvider>> = HashMap::new();
        let geo = GeoRouter::new_for_test(vec![], "stripe".into());
        let router = BillingRouter::new(providers, geo);
        assert_eq!(router.provider_name(), "router");
    }
}
