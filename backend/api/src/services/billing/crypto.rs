//! Crypto payment provider integration (No-KYC, direct wallet).
//!
//! Supports generating payment addresses and verifying on-chain transactions.
//! Uses a configurable blockchain API (e.g., NowNodes, BlockCypher, or self-hosted).

use async_trait::async_trait;

use super::provider::{
    BillingError, BillingProvider, CheckoutSession, ParsedWebhook, SubscriptionInfo, WebhookEvent,
};

/// Crypto billing provider.
pub struct CryptoProvider {
    /// Wallet address to receive payments
    pub wallet_address: String,
    /// Blockchain API base URL (e.g., "https://api.blockcypher.com/v1")
    pub api_url: String,
    /// API key for blockchain service
    pub api_key: String,
    /// Supported currency symbol (e.g., "BTC", "ETH", "XMR", "SOL")
    pub currency: String,
}

impl CryptoProvider {
    pub fn new(wallet_address: String, api_url: String, api_key: String, currency: String) -> Self {
        Self {
            wallet_address,
            api_url,
            api_key,
            currency: currency.to_uppercase(),
        }
    }

    /// Generate a BIP-21 (BTC), EIP-681 (ETH), or chain-specific payment URI.
    fn payment_uri(&self, amount: &str, memo: &str) -> String {
        match self.currency.as_str() {
            "BTC" => {
                // BIP-21: bitcoin:address?amount=X&label=Y
                let encoded = urlencoding::encode(memo);
                format!(
                    "bitcoin:{}?amount={}&label={}",
                    self.wallet_address, amount, encoded
                )
            }
            "ETH" => {
                // EIP-681: ethereum:address?value=X
                format!("ethereum:{}?value={}", self.wallet_address, amount)
            }
            "XMR" => {
                // Monero: monero:address?tx_description=X
                let encoded = urlencoding::encode(memo);
                format!("monero:{}?tx_description={}", self.wallet_address, encoded)
            }
            "SOL" => {
                // Solana: solana:address?amount=X&label=Y
                let encoded = urlencoding::encode(memo);
                format!(
                    "solana:{}?amount={}&label={}",
                    self.wallet_address, amount, encoded
                )
            }
            "LTC" => {
                let encoded = urlencoding::encode(memo);
                format!(
                    "litecoin:{}?amount={}&label={}",
                    self.wallet_address, amount, encoded
                )
            }
            "USDT" | "USDC" => {
                // Stablecoins on ETH: use EIP-681 with chain_id=1
                format!("ethereum:{}@1?value={}", self.wallet_address, amount)
            }
            _ => {
                // Generic fallback
                format!(
                    "{}:{}?amount={}&memo={}",
                    self.currency.to_lowercase(),
                    self.wallet_address,
                    amount,
                    urlencoding::encode(memo)
                )
            }
        }
    }

    /// Parse amount from plan_slug — expects format like "0.001" or "100_USD" (fiat amount for conversion).
    fn parse_amount(slug: &str) -> &str {
        // If slug contains underscore (e.g. "100_USD"), return just the numeric part
        if let Some(pos) = slug.find('_') {
            &slug[..pos]
        } else {
            slug
        }
    }

    /// Minimum confirmations required before marking payment as confirmed.
    /// Unused while crypto webhooks fail-closed (plan §1j); the on-chain
    /// confirmation-polling job will consult this once wired.
    #[allow(dead_code)]
    fn required_confirmations(&self) -> u64 {
        match self.currency.as_str() {
            "BTC" => 3,
            "ETH" => 12,
            "XMR" => 10,
            "SOL" => 32,
            "LTC" => 6,
            _ => 6,
        }
    }
}

#[async_trait]
impl BillingProvider for CryptoProvider {
    fn provider_name(&self) -> &'static str {
        "crypto"
    }

    async fn create_checkout(
        &self,
        plan_slug: &str,
        _customer_email: &str,
        user_id: i32,
        _success_url: &str,
        _cancel_url: &str,
    ) -> Result<CheckoutSession, BillingError> {
        let payment_id = format!("rux-{}-{}", user_id, uuid::Uuid::new_v4());
        let amount = Self::parse_amount(plan_slug);
        let checkout_url = self.payment_uri(amount, &payment_id);

        Ok(CheckoutSession {
            session_id: payment_id,
            checkout_url,
        })
    }

    async fn cancel_subscription(
        &self,
        _provider_subscription_id: &str,
        _immediately: bool,
    ) -> Result<(), BillingError> {
        Ok(())
    }

    async fn get_subscription(
        &self,
        provider_subscription_id: &str,
    ) -> Result<SubscriptionInfo, BillingError> {
        Ok(SubscriptionInfo {
            provider_subscription_id: provider_subscription_id.to_string(),
            status: "active".to_string(),
            current_period_end: None,
            cancel_at_period_end: false,
        })
    }

    async fn verify_webhook(&self, _event: WebhookEvent) -> Result<ParsedWebhook, BillingError> {
        // Crypto payments cannot be authenticated by a provider signature: an
        // on-chain payment notification carries no shared secret the merchant
        // could verify — anyone can POST a plausible `{hash, address, value}`
        // payload. Entitlement must therefore be granted only AFTER polling the
        // chain explorer (`CRYPTO_API_URL`) for the `tx_hash` and confirming
        // `required_confirmations()`. That confirmation-polling subsystem is not
        // wired yet; until it lands we reject every crypto webhook (fail closed)
        // rather than grant entitlement on unauthenticated JSON. The previous
        // implementation parsed and trusted the body verbatim — a forgery grants
        // paid content for free. See plan §1j and CRYPTO_AUDIT.md.
        Err(BillingError::WebhookVerification(
            "Crypto webhooks require on-chain confirmation polling (not yet implemented); \
             rejecting fail-closed"
                .into(),
        ))
    }

    async fn create_portal_session(
        &self,
        _provider_customer_id: &str,
        _return_url: &str,
    ) -> Result<String, BillingError> {
        Err(BillingError::InvalidRequest(
            "Crypto payments have no portal".to_string(),
        ))
    }
}

// ──────────────────────────────────────────────────────────────────────────
// Multi-chain crypto provider
// ──────────────────────────────────────────────────────────────────────────

/// A single chain's wallet configuration (parsed from JSON).
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ChainConfig {
    pub wallet: String,
    #[serde(default)]
    pub api_url: Option<String>,
    #[serde(default)]
    pub api_key: Option<String>,
}

/// Multi-chain checkout result — URIs for each chain.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MultiChainCheckout {
    pub payment_ref: String,
    pub options: std::collections::HashMap<String, String>,
}

/// Crypto provider supporting multiple chains simultaneously.
///
/// Configured via `CRYPTO_CHAINS` JSON env var:
/// ```json
/// {"BTC":{"wallet":"bc1q..."},"ETH":{"wallet":"0x..."},"SOL":{"wallet":"So1..."}}
/// ```
///
/// Returns ALL chain payment URIs at checkout so the user can choose.
pub struct MultiChainCryptoProvider {
    pub chains: std::collections::HashMap<String, CryptoProvider>,
}

impl MultiChainCryptoProvider {
    pub fn from_env() -> Result<Self, BillingError> {
        let raw_json = std::env::var("CRYPTO_CHAINS")
            .map_err(|_| BillingError::Config("CRYPTO_CHAINS not set".to_string()))?;

        let raw: std::collections::HashMap<String, ChainConfig> =
            serde_json::from_str(&raw_json)
                .map_err(|e| BillingError::Config(format!("Invalid CRYPTO_CHAINS JSON: {}", e)))?;

        let default_api_url = std::env::var("CRYPTO_API_URL")
            .unwrap_or_else(|_| "https://api.blockcypher.com/v1".into());
        let default_api_key = std::env::var("CRYPTO_API_KEY").unwrap_or_default();

        let mut chains = std::collections::HashMap::new();
        for (currency, cfg) in raw {
            let api_url = cfg.api_url.unwrap_or_else(|| default_api_url.clone());
            let api_key = cfg.api_key.unwrap_or_else(|| default_api_key.clone());
            chains.insert(
                currency.to_uppercase(),
                CryptoProvider::new(cfg.wallet, api_url, api_key, currency),
            );
        }

        if chains.is_empty() {
            return Err(BillingError::Config("CRYPTO_CHAINS is empty".to_string()));
        }

        Ok(Self { chains })
    }

    pub fn new(chains: Vec<(&str, &str)>) -> Self {
        let mut map = std::collections::HashMap::new();
        for (currency, wallet) in chains {
            map.insert(
                currency.to_uppercase(),
                CryptoProvider::new(
                    wallet.to_string(),
                    "https://api.example.com".to_string(),
                    String::new(),
                    currency.to_string(),
                ),
            );
        }
        Self { chains: map }
    }

    pub fn available_chains(&self) -> Vec<&str> {
        let mut c: Vec<&str> = self.chains.keys().map(|s| s.as_str()).collect();
        c.sort();
        c
    }
}

#[async_trait]
impl BillingProvider for MultiChainCryptoProvider {
    fn provider_name(&self) -> &'static str {
        "crypto_multi"
    }

    async fn create_checkout(
        &self,
        plan_slug: &str,
        _customer_email: &str,
        user_id: i32,
        _success_url: &str,
        _cancel_url: &str,
    ) -> Result<CheckoutSession, BillingError> {
        let payment_ref = format!("rux-{}-{}", user_id, uuid::Uuid::new_v4());
        let amount = CryptoProvider::parse_amount(plan_slug);

        let mut options = std::collections::HashMap::new();
        for (chain, provider) in &self.chains {
            options.insert(chain.clone(), provider.payment_uri(amount, &payment_ref));
        }

        let multi = MultiChainCheckout {
            payment_ref: payment_ref.clone(),
            options,
        };
        let checkout_url = serde_json::to_string(&multi)
            .map_err(|e| BillingError::Other(format!("Serialize error: {}", e)))?;

        Ok(CheckoutSession {
            session_id: payment_ref,
            checkout_url,
        })
    }

    async fn cancel_subscription(&self, _id: &str, _imm: bool) -> Result<(), BillingError> {
        Ok(())
    }

    async fn get_subscription(&self, id: &str) -> Result<SubscriptionInfo, BillingError> {
        Ok(SubscriptionInfo {
            provider_subscription_id: id.to_string(),
            status: "active".to_string(),
            current_period_end: None,
            cancel_at_period_end: false,
        })
    }

    async fn verify_webhook(&self, _event: WebhookEvent) -> Result<ParsedWebhook, BillingError> {
        // Multi-chain variant shares the same constraint as the single-chain
        // provider (see CryptoProvider::verify_webhook): crypto webhooks carry
        // no verifiable signature, so entitlement must come from on-chain
        // confirmation polling, not from this payload. Reject fail-closed
        // until that subsystem lands (plan §1j). The chain-routing logic that
        // used to live here was dead under the new model.
        Err(BillingError::WebhookVerification(
            "Crypto webhooks require on-chain confirmation polling (not yet implemented); \
             rejecting fail-closed"
                .into(),
        ))
    }

    async fn create_portal_session(&self, _: &str, _: &str) -> Result<String, BillingError> {
        Err(BillingError::InvalidRequest("Crypto has no portal".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn btc_provider() -> CryptoProvider {
        CryptoProvider::new(
            "bc1qexampleaddr".into(),
            "https://api.blockcypher.com/v1".into(),
            "api_key".into(),
            "BTC".into(),
        )
    }

    fn eth_provider() -> CryptoProvider {
        CryptoProvider::new(
            "0xDeadBeef1234".into(),
            "https://api.etherscan.io/api".into(),
            "eth_key".into(),
            "ETH".into(),
        )
    }

    fn xmr_provider() -> CryptoProvider {
        CryptoProvider::new(
            "4AdUndXHHZ6cfufTMvppY6JwXNouMBzSkbLYfpAV5Usx3skQNBYBXW".into(),
            "https://api.xmrchain.net".into(),
            "xmr_key".into(),
            "XMR".into(),
        )
    }

    #[test]
    fn test_crypto_provider_name() {
        assert_eq!(btc_provider().provider_name(), "crypto");
    }

    #[test]
    fn test_crypto_currency_uppercased() {
        let p = CryptoProvider::new("addr".into(), "url".into(), "key".into(), "btc".into());
        assert_eq!(p.currency, "BTC");
    }

    #[test]
    fn test_btc_payment_uri_bip21() {
        let p = btc_provider();
        let uri = p.payment_uri("0.001", "rux-42-abc");
        assert!(uri.starts_with("bitcoin:bc1qexampleaddr?"));
        assert!(uri.contains("amount=0.001"));
        assert!(uri.contains("label=rux-42-abc"));
    }

    #[test]
    fn test_eth_payment_uri_eip681() {
        let p = eth_provider();
        let uri = p.payment_uri("0.5", "rux-1-def");
        assert!(uri.starts_with("ethereum:0xDeadBeef1234?"));
        assert!(uri.contains("value=0.5"));
    }

    #[test]
    fn test_xmr_payment_uri() {
        let p = xmr_provider();
        let uri = p.payment_uri("1.5", "rux-5-xyz");
        assert!(uri.starts_with("monero:4AdUnd"));
        assert!(uri.contains("tx_description=rux-5-xyz"));
    }

    #[test]
    fn test_sol_payment_uri() {
        let p = CryptoProvider::new(
            "So1WalletAddress".into(),
            "https://api.mainnet-beta.solana.com".into(),
            "key".into(),
            "SOL".into(),
        );
        let uri = p.payment_uri("2.0", "rux-3-abc");
        assert!(uri.starts_with("solana:So1WalletAddress?"));
        assert!(uri.contains("amount=2.0"));
    }

    #[test]
    fn test_ltc_payment_uri() {
        let p = CryptoProvider::new(
            "ltc1qaddress".into(),
            "https://api.example.com".into(),
            "key".into(),
            "LTC".into(),
        );
        let uri = p.payment_uri("5.0", "rux-7-xyz");
        assert!(uri.starts_with("litecoin:ltc1qaddress?"));
    }

    #[test]
    fn test_usdt_payment_uri() {
        let p = CryptoProvider::new(
            "0xTokenAddr".into(),
            "https://api.example.com".into(),
            "key".into(),
            "USDT".into(),
        );
        let uri = p.payment_uri("100.0", "rux-1-abc");
        assert!(uri.starts_with("ethereum:0xTokenAddr@1?"));
    }

    #[test]
    fn test_generic_payment_uri_fallback() {
        let p = CryptoProvider::new(
            "DogeAddr123".into(),
            "https://api.example.com".into(),
            "key".into(),
            "DOGE".into(),
        );
        let uri = p.payment_uri("1000", "rux-1-abc");
        assert!(uri.starts_with("doge:DogeAddr123?"));
        assert!(uri.contains("amount=1000"));
    }

    #[test]
    fn test_parse_amount_plain_number() {
        assert_eq!(CryptoProvider::parse_amount("0.001"), "0.001");
    }

    #[test]
    fn test_parse_amount_fiat_suffix() {
        assert_eq!(CryptoProvider::parse_amount("100_USD"), "100");
    }

    #[tokio::test]
    async fn test_create_checkout_btc() {
        let p = btc_provider();
        let result = p
            .create_checkout(
                "0.001",
                "user@example.com",
                42,
                "https://s.cx/s",
                "https://s.cx/c",
            )
            .await
            .expect("checkout should succeed");

        assert!(result.session_id.starts_with("rux-42-"));
        assert!(result.checkout_url.starts_with("bitcoin:bc1qexampleaddr?"));
        assert!(result.checkout_url.contains("amount=0.001"));
    }

    #[tokio::test]
    async fn test_create_checkout_eth() {
        let p = eth_provider();
        let result = p
            .create_checkout(
                "0.5",
                "user@example.com",
                7,
                "https://s.cx/s",
                "https://s.cx/c",
            )
            .await
            .expect("checkout should succeed");

        assert!(result.session_id.starts_with("rux-7-"));
        assert!(result.checkout_url.starts_with("ethereum:0xDeadBeef1234?"));
    }

    #[tokio::test]
    async fn test_create_checkout_fiat_slug() {
        let p = btc_provider();
        let result = p
            .create_checkout(
                "100_USD",
                "user@example.com",
                1,
                "https://s.cx/s",
                "https://s.cx/c",
            )
            .await
            .expect("checkout should succeed");

        // Should extract "100" from "100_USD"
        assert!(result.checkout_url.contains("amount=100"));
    }

    #[tokio::test]
    async fn test_cancel_subscription_always_ok() {
        let p = btc_provider();
        assert!(p.cancel_subscription("any_id", true).await.is_ok());
    }

    #[tokio::test]
    async fn test_get_subscription_returns_active() {
        let p = btc_provider();
        let sub = p.get_subscription("payment_123").await.expect("ok");
        assert_eq!(sub.status, "active");
        assert_eq!(sub.provider_subscription_id, "payment_123");
    }

    #[tokio::test]
    async fn test_portal_session_returns_error() {
        let p = btc_provider();
        assert!(p.create_portal_session("c", "https://r").await.is_err());
    }

    #[test]
    fn test_required_confirmations() {
        assert_eq!(btc_provider().required_confirmations(), 3);
        assert_eq!(eth_provider().required_confirmations(), 12);
        assert_eq!(xmr_provider().required_confirmations(), 10);
        assert_eq!(
            CryptoProvider::new("a".into(), "u".into(), "k".into(), "SOL".into())
                .required_confirmations(),
            32
        );
        assert_eq!(
            CryptoProvider::new("a".into(), "u".into(), "k".into(), "LTC".into())
                .required_confirmations(),
            6
        );
    }

    #[tokio::test]
    async fn test_verify_webhook_rejected_fail_closed() {
        // Crypto webhooks cannot be cryptographically verified, so every
        // payload must be rejected until on-chain confirmation polling lands
        // (plan §1j). A confirmed-looking payload is NOT trusted.
        let p = btc_provider();
        let payload = serde_json::json!({
            "hash": "abc123def456",
            "confirmations": 5,
            "address": "bc1qexampleaddr",
            "value": 0.001,
            "memo": "rux-42-test"
        });

        let event = WebhookEvent {
            provider: "crypto".into(),
            payload: serde_json::to_vec(&payload).unwrap(),
            headers: axum::http::HeaderMap::new(),
        };

        let err = p
            .verify_webhook(event)
            .await
            .expect_err("must reject fail-closed");
        assert!(matches!(err, BillingError::WebhookVerification(_)));
    }

    // ──────────────────────────────────────────────────────────────────
    // Multi-chain tests
    // ──────────────────────────────────────────────────────────────────

    fn multi_provider() -> MultiChainCryptoProvider {
        MultiChainCryptoProvider::new(vec![
            ("BTC", "bc1qtest"),
            ("ETH", "0xEthTest"),
            ("SOL", "SolTestAddr"),
        ])
    }

    #[test]
    fn test_multi_chain_provider_name() {
        assert_eq!(multi_provider().provider_name(), "crypto_multi");
    }

    #[test]
    fn test_available_chains() {
        let p = multi_provider();
        let chains = p.available_chains();
        assert_eq!(chains, vec!["BTC", "ETH", "SOL"]);
    }

    #[tokio::test]
    async fn test_multi_chain_checkout_returns_all_options() {
        let p = multi_provider();
        let result = p
            .create_checkout("0.001", "u@t.com", 42, "https://s.cx/s", "https://s.cx/c")
            .await
            .expect("ok");

        assert!(result.session_id.starts_with("rux-42-"));

        let multi: MultiChainCheckout =
            serde_json::from_str(&result.checkout_url).expect("should parse");

        assert!(multi.options.contains_key("BTC"));
        assert!(multi.options.contains_key("ETH"));
        assert!(multi.options.contains_key("SOL"));

        assert!(multi.options["BTC"].starts_with("bitcoin:bc1qtest?"));
        assert!(multi.options["ETH"].starts_with("ethereum:0xEthTest?"));
        assert!(multi.options["SOL"].starts_with("solana:SolTestAddr?"));
    }

    #[tokio::test]
    async fn test_multi_chain_verify_rejected_fail_closed() {
        // Multi-chain variant also rejects fail-closed (plan §1j).
        let p = multi_provider();
        let payload = serde_json::json!({
            "hash": "btc_tx_123",
            "confirmations": 5,
            "address": "bc1qtest",
            "value": 0.001,
            "memo": "rux-42-test"
        });

        let event = WebhookEvent {
            provider: "crypto".into(),
            payload: serde_json::to_vec(&payload).unwrap(),
            headers: axum::http::HeaderMap::new(),
        };

        let err = p
            .verify_webhook(event)
            .await
            .expect_err("must reject fail-closed");
        assert!(matches!(err, BillingError::WebhookVerification(_)));
    }

    #[test]
    fn test_multi_from_env_missing() {
        std::env::remove_var("CRYPTO_CHAINS");
        assert!(MultiChainCryptoProvider::from_env().is_err());
    }

    #[tokio::test]
    async fn test_multi_cancel_ok() {
        assert!(multi_provider()
            .cancel_subscription("x", true)
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn test_multi_get_sub_active() {
        let sub = multi_provider().get_subscription("p1").await.expect("ok");
        assert_eq!(sub.status, "active");
    }

    #[tokio::test]
    async fn test_multi_portal_error() {
        assert!(multi_provider()
            .create_portal_session("c", "r")
            .await
            .is_err());
    }

    #[test]
    fn test_single_chain_works() {
        let p = MultiChainCryptoProvider::new(vec![("XMR", "4AMoneroAddr")]);
        assert_eq!(p.available_chains(), vec!["XMR"]);
    }
}
