//! Mock HTTP tests for billing providers.
//!
//! Uses wiremock to simulate provider APIs without real credentials.
//! Run: cargo test --test billing_providers --features "full"

#[cfg(feature = "billing")]
mod billing_mock_tests {
    use hmac::{Hmac, Mac};
    use ruxlog::services::billing::crypto::CryptoProvider;
    use ruxlog::services::billing::provider::{BillingError, BillingProvider, WebhookEvent};
    use ruxlog::services::billing::stripe::StripeProvider;
    use serde_json::json;
    use sha2::Sha256;
    use wiremock::matchers::{body_string_contains, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    type HmacSha256 = Hmac<Sha256>;

    fn stripe_mock_provider(server: &MockServer) -> StripeProvider {
        StripeProvider::new("sk_test_123".into(), "whsec_test".into()).with_base_url(server.uri())
    }

    fn stripe_webhook_provider(secret: &str) -> StripeProvider {
        StripeProvider::new("sk_test".into(), secret.into())
    }

    // Import providers
    #[cfg(feature = "billing")]
    use ruxlog::services::billing::lemon_squeezy::LemonSqueezyProvider;
    #[cfg(feature = "billing")]
    use ruxlog::services::billing::paddle::PaddleProvider;
    #[cfg(feature = "billing")]
    use ruxlog::services::billing::polar::PolarProvider;

    fn sign_payload(secret: &str, payload: &[u8]) -> String {
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC key");
        mac.update(payload);
        hex::encode(mac.finalize().into_bytes())
    }

    // ── Stripe: create_checkout ──────────────────────────────────────────

    #[tokio::test]
    async fn stripe_create_checkout_returns_session() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/checkout/sessions"))
            .and(header("Authorization", "Bearer sk_test_123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "cs_test_abc123",
                "url": format!("{}/checkout/redirect", server.uri()),
                "object": "checkout.session"
            })))
            .mount(&server)
            .await;

        let provider = stripe_mock_provider(&server);

        let result = provider
            .create_checkout(
                "price_123",
                "user@example.com",
                42,
                "https://app.example.com/success",
                "https://app.example.com/cancel",
            )
            .await
            .expect("checkout should succeed");

        assert_eq!(result.session_id, "cs_test_abc123");
        assert!(result.checkout_url.contains("/checkout/redirect"));
    }

    #[tokio::test]
    async fn stripe_create_checkout_propagates_api_error() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/checkout/sessions"))
            .respond_with(ResponseTemplate::new(400).set_body_json(json!({
                "error": { "message": "Invalid price" }
            })))
            .mount(&server)
            .await;

        let provider = stripe_mock_provider(&server);

        let result = provider
            .create_checkout(
                "price_invalid",
                "user@example.com",
                1,
                "https://s.cx/s",
                "https://s.cx/c",
            )
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, BillingError::ProviderApi(_)));
    }

    // ── Stripe: cancel_subscription ──────────────────────────────────────

    #[tokio::test]
    async fn stripe_cancel_subscription_immediate() {
        let server = MockServer::start().await;

        Mock::given(method("DELETE"))
            .and(path("/v1/subscriptions/sub_123"))
            .and(header("Authorization", "Bearer sk_test_123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "sub_123",
                "status": "canceled"
            })))
            .mount(&server)
            .await;

        let provider = stripe_mock_provider(&server);

        let result = provider.cancel_subscription("sub_123", true).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn stripe_cancel_subscription_at_period_end() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/subscriptions/sub_456"))
            .and(header("Authorization", "Bearer sk_test_123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "sub_456",
                "cancel_at_period_end": true
            })))
            .mount(&server)
            .await;

        let provider = stripe_mock_provider(&server);

        let result = provider.cancel_subscription("sub_456", false).await;

        assert!(result.is_ok());
    }

    // ── Stripe: get_subscription ─────────────────────────────────────────

    #[tokio::test]
    async fn stripe_get_subscription_returns_info() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v1/subscriptions/sub_789"))
            .and(header("Authorization", "Bearer sk_test_123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "sub_789",
                "status": "active",
                "current_period_end": 1735689600,
                "cancel_at_period_end": false
            })))
            .mount(&server)
            .await;

        let provider = stripe_mock_provider(&server);

        let info = provider
            .get_subscription("sub_789")
            .await
            .expect("should get subscription");

        assert_eq!(info.provider_subscription_id, "sub_789");
        assert_eq!(info.status, "active");
        assert!(!info.cancel_at_period_end);
        assert!(info.current_period_end.is_some());
    }

    // ── Stripe: verify_webhook ───────────────────────────────────────────

    #[tokio::test]
    async fn stripe_verify_webhook_valid_signature() {
        let webhook_secret = "whsec_secret_key_123";
        let provider = stripe_webhook_provider(webhook_secret);

        let payload = json!({
            "type": "checkout.session.completed",
            "data": {
                "object": {
                    "customer": "cus_abc",
                    "subscription": "sub_xyz",
                    "payment_intent": "pi_123",
                    "metadata": { "user_id": "42" }
                }
            }
        });
        let payload_bytes = serde_json::to_vec(&payload).unwrap();
        let signature = sign_payload(webhook_secret, &payload_bytes);

        let event = WebhookEvent {
            provider: "stripe".into(),
            payload: payload_bytes,
            signature,
        };

        let parsed = provider.verify_webhook(event).await.expect("should verify");

        assert_eq!(parsed.event_type, "checkout.session.completed");
        assert_eq!(parsed.customer_id, "cus_abc");
        assert_eq!(parsed.subscription_id, Some("sub_xyz".to_string()));
        assert_eq!(parsed.payment_id, Some("pi_123".to_string()));
    }

    #[tokio::test]
    async fn stripe_verify_webhook_invalid_signature() {
        let provider = stripe_webhook_provider("whsec_real");

        let payload = json!({"type": "test", "data": {"object": {}}});
        let event = WebhookEvent {
            provider: "stripe".into(),
            payload: serde_json::to_vec(&payload).unwrap(),
            signature: "badsignature".to_string(),
        };

        let result = provider.verify_webhook(event).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            BillingError::WebhookVerification(_)
        ));
    }

    // ── Stripe: create_portal_session ────────────────────────────────────

    #[tokio::test]
    async fn stripe_create_portal_session() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/billing_portal/sessions"))
            .and(header("Authorization", "Bearer sk_test_123"))
            .and(body_string_contains("customer"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "bps_test_123",
                "url": format!("{}/portal/session", server.uri()),
            })))
            .mount(&server)
            .await;

        let provider = stripe_mock_provider(&server);

        let url = provider
            .create_portal_session("cus_abc", "https://app.example.com/account")
            .await
            .expect("should create portal session");

        assert!(url.contains("/portal/session"));
    }

    // ── Provider trait consistency ────────────────────────────────────────

    #[test]
    fn stripe_provider_name() {
        let provider = StripeProvider::new("sk_test".into(), "whsec_test".into());
        assert_eq!(provider.provider_name(), "stripe");
    }

    // ── Full webhook event flow: checkout -> parsed -> event_type matches ─

    #[tokio::test]
    async fn stripe_full_webhook_flow_subscription_events() {
        let webhook_secret = "whsec_flow_test";
        let provider = stripe_webhook_provider(webhook_secret);

        // Test subscription.updated event
        let updated_payload = json!({
            "type": "customer.subscription.updated",
            "data": {
                "object": {
                    "customer": "cus_flow",
                    "subscription": "sub_flow",
                    "status": "past_due"
                }
            }
        });
        let bytes = serde_json::to_vec(&updated_payload).unwrap();
        let sig = sign_payload(webhook_secret, &bytes);

        let parsed = provider
            .verify_webhook(WebhookEvent {
                provider: "stripe".into(),
                payload: bytes,
                signature: sig,
            })
            .await
            .expect("should verify");

        assert_eq!(parsed.event_type, "customer.subscription.updated");
        assert_eq!(parsed.customer_id, "cus_flow");

        // Test subscription.deleted event
        let deleted_payload = json!({
            "type": "customer.subscription.deleted",
            "data": {
                "object": {
                    "customer": "cus_flow",
                    "subscription": "sub_flow",
                    "status": "canceled"
                }
            }
        });
        let bytes = serde_json::to_vec(&deleted_payload).unwrap();
        let sig = sign_payload(webhook_secret, &bytes);

        let parsed = provider
            .verify_webhook(WebhookEvent {
                provider: "stripe".into(),
                payload: bytes,
                signature: sig,
            })
            .await
            .expect("should verify");

        assert_eq!(parsed.event_type, "customer.subscription.deleted");
    }

    #[tokio::test]
    async fn stripe_invoice_payment_succeeded_webhook() {
        let webhook_secret = "whsec_invoice_test";
        let provider = stripe_webhook_provider(webhook_secret);

        let payload = json!({
            "type": "invoice.payment_succeeded",
            "data": {
                "object": {
                    "customer": "cus_inv",
                    "subscription": "sub_inv",
                    "payment_intent": "pi_inv",
                    "amount_paid": 999,
                    "currency": "usd"
                }
            }
        });
        let bytes = serde_json::to_vec(&payload).unwrap();
        let sig = sign_payload(webhook_secret, &bytes);

        let parsed = provider
            .verify_webhook(WebhookEvent {
                provider: "stripe".into(),
                payload: bytes,
                signature: sig,
            })
            .await
            .expect("should verify");

        assert_eq!(parsed.event_type, "invoice.payment_succeeded");
        assert_eq!(parsed.payment_id, Some("pi_inv".to_string()));
    }

    // ══════════════════════════════════════════════════════════════════════
    // Polar.sh mock tests
    // ══════════════════════════════════════════════════════════════════════

    mod polar_tests {
        use super::*;

        fn polar_mock(server: &MockServer) -> PolarProvider {
            PolarProvider::new("polar_token".into(), "polar_whsec".into())
                .with_base_url(server.uri())
        }

        #[tokio::test]
        async fn create_checkout() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/v1/checkouts/"))
                .and(header("Authorization", "Bearer polar_token"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "id": "polar_sess_1",
                    "url": format!("{}/polar/checkout", server.uri()),
                })))
                .mount(&server)
                .await;

            let result = polar_mock(&server)
                .create_checkout(
                    "prod_abc",
                    "user@test.com",
                    5,
                    "https://s.cx/s",
                    "https://s.cx/c",
                )
                .await
                .expect("ok");
            assert_eq!(result.session_id, "polar_sess_1");
        }

        #[tokio::test]
        async fn cancel_subscription() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/v1/subscriptions/sub_pol/cancel"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": "sub_pol"})))
                .mount(&server)
                .await;

            assert!(polar_mock(&server)
                .cancel_subscription("sub_pol", true)
                .await
                .is_ok());
        }

        #[tokio::test]
        async fn get_subscription() {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/v1/subscriptions/sub_pol2"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "id": "sub_pol2",
                    "status": "active",
                    "current_period_end": "2026-12-31T00:00:00Z",
                    "cancel_at_period_end": false,
                })))
                .mount(&server)
                .await;

            let info = polar_mock(&server)
                .get_subscription("sub_pol2")
                .await
                .expect("ok");
            assert_eq!(info.status, "active");
        }

        #[tokio::test]
        async fn verify_webhook() {
            let provider = PolarProvider::new("tok".into(), "sec".into());
            let payload = json!({
                "type": "subscription.created",
                "data": { "customer_id": "cus_pol", "subscription_id": "sub_pol3", "order_id": "ord_1" }
            });
            let bytes = serde_json::to_vec(&payload).unwrap();
            let parsed = provider
                .verify_webhook(WebhookEvent {
                    provider: "polar".into(),
                    payload: bytes,
                    signature: String::new(),
                })
                .await
                .expect("polar doesn't verify sig");
            assert_eq!(parsed.event_type, "subscription.created");
        }

        #[test]
        fn provider_name() {
            assert_eq!(
                PolarProvider::new("t".into(), "s".into()).provider_name(),
                "polar"
            );
        }
    }

    // ══════════════════════════════════════════════════════════════════════
    // LemonSqueezy mock tests
    // ══════════════════════════════════════════════════════════════════════

    mod lemon_tests {
        use super::*;

        fn lemon_mock(server: &MockServer) -> LemonSqueezyProvider {
            LemonSqueezyProvider::new("ls_key".into(), "ls_whsec".into(), "store_1".into())
                .with_base_url(server.uri())
        }

        #[tokio::test]
        async fn create_checkout() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/v1/checkouts"))
                .and(header("Authorization", "Bearer ls_key"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "data": {
                        "id": "ls_sess_1",
                        "attributes": { "url": format!("{}/ls/checkout", server.uri()) }
                    }
                })))
                .mount(&server)
                .await;

            let result = lemon_mock(&server)
                .create_checkout(
                    "var_123",
                    "user@test.com",
                    3,
                    "https://s.cx/s",
                    "https://s.cx/c",
                )
                .await
                .expect("ok");
            assert_eq!(result.session_id, "ls_sess_1");
        }

        #[tokio::test]
        async fn cancel_subscription() {
            let server = MockServer::start().await;
            Mock::given(method("PATCH"))
                .and(path("/v1/subscriptions/sub_ls"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "data": { "id": "sub_ls", "attributes": { "cancelled": true } }
                })))
                .mount(&server)
                .await;

            assert!(lemon_mock(&server)
                .cancel_subscription("sub_ls", true)
                .await
                .is_ok());
        }

        #[tokio::test]
        async fn get_subscription() {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/v1/subscriptions/sub_ls2"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "data": {
                        "id": "sub_ls2",
                        "attributes": { "status": "active", "renews_at": "2026-12-31T00:00:00Z", "cancelled": false }
                    }
                })))
                .mount(&server)
                .await;

            let info = lemon_mock(&server)
                .get_subscription("sub_ls2")
                .await
                .expect("ok");
            assert_eq!(info.status, "active");
        }

        #[tokio::test]
        async fn verify_webhook() {
            let provider = LemonSqueezyProvider::new("k".into(), "secret123".into(), "s".into());
            let payload = json!({
                "meta": { "event_name": "subscription_created" },
                "data": {
                    "id": "sub_lsw",
                    "attributes": { "customer_id": "cus_ls", "order_id": "ord_ls" }
                }
            });
            let payload_bytes = serde_json::to_vec(&payload).unwrap();
            let sig = sign_payload("secret123", &payload_bytes);

            let parsed = provider
                .verify_webhook(WebhookEvent {
                    provider: "lemonsqueezy".into(),
                    payload: payload_bytes,
                    signature: sig,
                })
                .await
                .expect("should verify");
            assert_eq!(parsed.event_type, "subscription_created");
        }

        #[test]
        fn provider_name() {
            assert_eq!(
                LemonSqueezyProvider::new("k".into(), "s".into(), "st".into()).provider_name(),
                "lemon_squeezy"
            );
        }
    }

    // ══════════════════════════════════════════════════════════════════════
    // Paddle mock tests
    // ══════════════════════════════════════════════════════════════════════

    mod paddle_tests {
        use super::*;

        fn paddle_mock(server: &MockServer) -> PaddleProvider {
            PaddleProvider::new("paddle_tok".into(), "paddle_whsec".into())
                .with_base_url(server.uri())
        }

        #[tokio::test]
        async fn create_checkout() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/transactions"))
                .and(header("Authorization", "Bearer paddle_tok"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "data": {
                        "id": "paddle_tx_1",
                        "checkout": { "url": format!("{}/paddle/checkout", server.uri()) }
                    }
                })))
                .mount(&server)
                .await;

            let result = paddle_mock(&server)
                .create_checkout(
                    "pri_123",
                    "user@test.com",
                    9,
                    "https://s.cx/s",
                    "https://s.cx/c",
                )
                .await
                .expect("ok");
            assert_eq!(result.session_id, "paddle_tx_1");
        }

        #[tokio::test]
        async fn cancel_subscription() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/subscriptions/sub_pad"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "data": { "id": "sub_pad", "status": "canceled" }
                })))
                .mount(&server)
                .await;

            assert!(paddle_mock(&server)
                .cancel_subscription("sub_pad", true)
                .await
                .is_ok());
        }

        #[tokio::test]
        async fn get_subscription() {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/subscriptions/sub_pad2"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "data": {
                        "id": "sub_pad2",
                        "status": "active",
                        "next_billed_at": "2026-12-31T00:00:00Z",
                        "scheduled_change": null,
                    }
                })))
                .mount(&server)
                .await;

            let info = paddle_mock(&server)
                .get_subscription("sub_pad2")
                .await
                .expect("ok");
            assert_eq!(info.status, "active");
            assert!(!info.cancel_at_period_end);
        }

        #[tokio::test]
        async fn verify_webhook() {
            let provider = PaddleProvider::new("tok".into(), "padsecret".into());
            let payload = json!({
                "event_type": "subscription.created",
                "data": { "customer_id": "cus_pad", "id": "sub_padw", "transaction_id": "txn_1" }
            });
            let payload_bytes = serde_json::to_vec(&payload).unwrap();
            let sig = sign_payload("padsecret", &payload_bytes);

            let parsed = provider
                .verify_webhook(WebhookEvent {
                    provider: "paddle".into(),
                    payload: payload_bytes,
                    signature: sig,
                })
                .await
                .expect("should verify");
            assert_eq!(parsed.event_type, "subscription.created");
            assert_eq!(parsed.subscription_id, Some("sub_padw".to_string()));
            assert_eq!(parsed.payment_id, Some("txn_1".to_string()));
        }

        #[test]
        fn provider_name() {
            assert_eq!(
                PaddleProvider::new("t".into(), "s".into()).provider_name(),
                "paddle"
            );
        }
    }

    // ══════════════════════════════════════════════════════════════════════
    // Razorpay mock tests
    // ══════════════════════════════════════════════════════════════════════

    mod razorpay_tests {
        use super::*;
        use ruxlog::services::billing::razorpay::RazorpayProvider;

        fn razorpay_mock(server: &MockServer) -> RazorpayProvider {
            RazorpayProvider::new("rzp_key".into(), "rzp_secret".into(), "rzp_whsec".into())
                .with_base_url(server.uri())
        }

        #[tokio::test]
        async fn create_checkout() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/payment_links"))
                .and(header("Content-Type", "application/json"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "id": "plink_abc123",
                    "short_url": format!("{}/rzp/checkout", server.uri()),
                })))
                .mount(&server)
                .await;

            let result = razorpay_mock(&server)
                .create_checkout("99900", "user@test.com", 5, "https://s.cx/s", "https://s.cx/c")
                .await
                .expect("ok");
            assert_eq!(result.session_id, "plink_abc123");
            assert!(result.checkout_url.contains("/rzp/checkout"));
        }

        #[tokio::test]
        async fn create_checkout_api_error() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/payment_links"))
                .respond_with(ResponseTemplate::new(400).set_body_json(json!({
                    "error": { "code": "BAD_REQUEST", "description": "Invalid amount" }
                })))
                .mount(&server)
                .await;

            let result = razorpay_mock(&server)
                .create_checkout("bad", "u@t.com", 1, "https://s", "https://c")
                .await;
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), BillingError::ProviderApi(_)));
        }

        #[tokio::test]
        async fn cancel_subscription_immediate() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/subscriptions/sub_rzp/cancel"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": "sub_rzp"})))
                .mount(&server)
                .await;

            assert!(razorpay_mock(&server)
                .cancel_subscription("sub_rzp", true)
                .await
                .is_ok());
        }

        #[tokio::test]
        async fn cancel_subscription_at_period_end() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/subscriptions/sub_rzp2/cancel"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": "sub_rzp2"})))
                .mount(&server)
                .await;

            assert!(razorpay_mock(&server)
                .cancel_subscription("sub_rzp2", false)
                .await
                .is_ok());
        }

        #[tokio::test]
        async fn get_subscription() {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/subscriptions/sub_rzp3"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "id": "sub_rzp3",
                    "status": "active",
                    "current_end": 1735689600,
                })))
                .mount(&server)
                .await;

            let info = razorpay_mock(&server)
                .get_subscription("sub_rzp3")
                .await
                .expect("ok");
            assert_eq!(info.provider_subscription_id, "sub_rzp3");
            assert_eq!(info.status, "active");
        }

        #[tokio::test]
        async fn verify_webhook_valid_signature() {
            let secret = "rzp_secret_123";
            let provider = RazorpayProvider::new("k".into(), "s".into(), secret.into());
            let payload = json!({
                "event": "payment.captured",
                "payload": {
                    "payment": { "entity": { "id": "pay_abc", "customer_id": "cus_rzp" } },
                    "subscription": { "entity": { "id": "sub_rzp_w" } }
                }
            });
            let bytes = serde_json::to_vec(&payload).unwrap();
            let sig = sign_payload(secret, &bytes);

            let parsed = provider
                .verify_webhook(WebhookEvent {
                    provider: "razorpay".into(),
                    payload: bytes,
                    signature: sig,
                })
                .await
                .expect("should verify");
            assert_eq!(parsed.event_type, "payment.captured");
            assert_eq!(parsed.customer_id, "cus_rzp");
            assert_eq!(parsed.subscription_id, Some("sub_rzp_w".to_string()));
        }

        #[tokio::test]
        async fn verify_webhook_invalid_signature() {
            let provider = RazorpayProvider::new("k".into(), "s".into(), "real_secret".into());
            let payload = json!({"event": "test"});
            let result = provider
                .verify_webhook(WebhookEvent {
                    provider: "razorpay".into(),
                    payload: serde_json::to_vec(&payload).unwrap(),
                    signature: "badsig".to_string(),
                })
                .await;
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), BillingError::WebhookVerification(_)));
        }

        #[tokio::test]
        async fn portal_session_returns_url() {
            let provider = RazorpayProvider::new("k".into(), "s".into(), "w".into());
            let url = provider
                .create_portal_session("cus_123", "https://app.example.com/account")
                .await
                .expect("ok");
            assert!(url.contains("cus_123"));
            assert!(url.contains("return_url="));
        }

        #[test]
        fn provider_name() {
            assert_eq!(
                RazorpayProvider::new("k".into(), "s".into(), "w".into()).provider_name(),
                "razorpay"
            );
        }
    }

    // ══════════════════════════════════════════════════════════════════════
    // Mercado Pago mock tests
    // ══════════════════════════════════════════════════════════════════════

    mod mercado_pago_tests {
        use super::*;
        use ruxlog::services::billing::mercado_pago::MercadoPagoProvider;

        fn mp_mock(server: &MockServer) -> MercadoPagoProvider {
            MercadoPagoProvider::new("mp_token".into(), "mp_whsec".into())
                .with_base_url(server.uri())
        }

        #[tokio::test]
        async fn create_checkout() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/checkout/preferences"))
                .and(header("Authorization", "Bearer mp_token"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "id": "mp_pref_1",
                    "init_point": format!("{}/mp/checkout", server.uri()),
                })))
                .mount(&server)
                .await;

            let result = mp_mock(&server)
                .create_checkout("99.90", "user@test.com", 3, "https://s.cx/s", "https://s.cx/c")
                .await
                .expect("ok");
            assert_eq!(result.session_id, "mp_pref_1");
            assert!(result.checkout_url.contains("/mp/checkout"));
        }

        #[tokio::test]
        async fn cancel_subscription() {
            let server = MockServer::start().await;
            Mock::given(method("PUT"))
                .and(path("/preapproval/sub_mp"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": "sub_mp", "status": "cancelled"})))
                .mount(&server)
                .await;

            assert!(mp_mock(&server).cancel_subscription("sub_mp", true).await.is_ok());
        }

        #[tokio::test]
        async fn get_subscription() {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/preapproval/sub_mp2"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "id": "sub_mp2",
                    "status": "authorized",
                    "next_payment_date": "2026-12-31T00:00:00Z",
                })))
                .mount(&server)
                .await;

            let info = mp_mock(&server).get_subscription("sub_mp2").await.expect("ok");
            assert_eq!(info.status, "authorized");
            assert!(info.current_period_end.is_some());
        }

        #[tokio::test]
        async fn verify_webhook_valid_signature() {
            let secret = "mp_secret_123";
            let provider = MercadoPagoProvider::new("tok".into(), secret.into());
            let payload = json!({
                "type": "payment",
                "data": { "id": "pay_mp1", "payer_id": "cus_mp1", "preapproval_id": "sub_mp_w" }
            });
            let payload_bytes = serde_json::to_vec(&payload).unwrap();
            let payload_str = String::from_utf8(payload_bytes.clone()).unwrap();

            // Mercado Pago signature format: ts=<timestamp>,v1=<hmac>
            let ts = "1234567890";
            let manifest = format!("{}{}", ts, payload_str);
            let v1 = sign_payload(secret, manifest.as_bytes());
            let signature = format!("ts={},v1={}", ts, v1);

            let parsed = provider
                .verify_webhook(WebhookEvent {
                    provider: "mercado_pago".into(),
                    payload: payload_bytes,
                    signature,
                })
                .await
                .expect("should verify");
            assert_eq!(parsed.event_type, "payment");
            assert_eq!(parsed.customer_id, "cus_mp1");
            assert_eq!(parsed.subscription_id, Some("sub_mp_w".to_string()));
            assert_eq!(parsed.payment_id, Some("pay_mp1".to_string()));
        }

        #[tokio::test]
        async fn verify_webhook_invalid_signature() {
            let provider = MercadoPagoProvider::new("t".into(), "real_secret".into());
            let result = provider
                .verify_webhook(WebhookEvent {
                    provider: "mercado_pago".into(),
                    payload: b"{}".to_vec(),
                    signature: "ts=0,v1=bad".to_string(),
                })
                .await;
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), BillingError::WebhookVerification(_)));
        }

        #[tokio::test]
        async fn portal_session_returns_url() {
            let provider = MercadoPagoProvider::new("t".into(), "w".into());
            let url = provider
                .create_portal_session("cus_mp", "https://app.com/account")
                .await
                .expect("ok");
            assert!(url.contains("cus_mp"));
        }

        #[test]
        fn provider_name() {
            assert_eq!(
                MercadoPagoProvider::new("t".into(), "w".into()).provider_name(),
                "mercado_pago"
            );
        }
    }

    // ══════════════════════════════════════════════════════════════════════
    // Airwallex mock tests
    // ══════════════════════════════════════════════════════════════════════

    mod airwallex_tests {
        use super::*;
        use ruxlog::services::billing::airwallex::AirwallexProvider;

        fn awx_mock(server: &MockServer) -> AirwallexProvider {
            AirwallexProvider::new("awx_client".into(), "awx_key".into(), "awx_whsec".into())
                .with_base_url(server.uri())
        }

        #[tokio::test]
        async fn create_checkout() {
            let server = MockServer::start().await;

            // Mock auth endpoint
            Mock::given(method("POST"))
                .and(path("/authentication/login"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({"token": "awx_bearer_tok"})))
                .mount(&server)
                .await;

            // Mock payment intent creation
            Mock::given(method("POST"))
                .and(path("/pa/payment_intents/create"))
                .and(header("Authorization", "Bearer awx_bearer_tok"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "id": "int_awx_1",
                    "client_secret": "secret_abc123",
                })))
                .mount(&server)
                .await;

            let result = awx_mock(&server)
                .create_checkout("99.00", "user@test.com", 7, "https://s.cx/s", "https://s.cx/c")
                .await
                .expect("ok");
            assert_eq!(result.session_id, "int_awx_1");
            assert!(result.checkout_url.contains("int_awx_1"));
            assert!(result.checkout_url.contains("secret_abc123"));
        }

        #[tokio::test]
        async fn cancel_subscription() {
            let server = MockServer::start().await;

            Mock::given(method("POST"))
                .and(path("/authentication/login"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({"token": "tok"})))
                .mount(&server)
                .await;

            Mock::given(method("POST"))
                .and(path("/pa/subscriptions/sub_awx/cancel"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": "sub_awx"})))
                .mount(&server)
                .await;

            assert!(awx_mock(&server).cancel_subscription("sub_awx", true).await.is_ok());
        }

        #[tokio::test]
        async fn get_subscription() {
            let server = MockServer::start().await;

            Mock::given(method("POST"))
                .and(path("/authentication/login"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({"token": "tok"})))
                .mount(&server)
                .await;

            Mock::given(method("GET"))
                .and(path("/pa/subscriptions/sub_awx2"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "id": "sub_awx2",
                    "status": "ACTIVE",
                    "current_period_end": "2026-12-31T00:00:00Z",
                    "cancel_at_period_end": false,
                })))
                .mount(&server)
                .await;

            let info = awx_mock(&server).get_subscription("sub_awx2").await.expect("ok");
            assert_eq!(info.status, "ACTIVE");
            assert!(!info.cancel_at_period_end);
        }

        #[tokio::test]
        async fn verify_webhook_valid_signature() {
            let secret = "awx_secret_123";
            let provider = AirwallexProvider::new("c".into(), "k".into(), secret.into());
            let payload = json!({
                "event_type": "payment_intent.created",
                "data": {
                    "entity": {
                        "customer_id": "cus_awx",
                        "subscription_id": "sub_awx_w",
                        "payment_intent_id": "pi_awx_1",
                    }
                }
            });
            let bytes = serde_json::to_vec(&payload).unwrap();
            let sig = sign_payload(secret, &bytes);

            let parsed = provider
                .verify_webhook(WebhookEvent {
                    provider: "airwallex".into(),
                    payload: bytes,
                    signature: sig,
                })
                .await
                .expect("should verify");
            assert_eq!(parsed.event_type, "payment_intent.created");
            assert_eq!(parsed.customer_id, "cus_awx");
            assert_eq!(parsed.subscription_id, Some("sub_awx_w".to_string()));
            assert_eq!(parsed.payment_id, Some("pi_awx_1".to_string()));
        }

        #[tokio::test]
        async fn verify_webhook_invalid_signature() {
            let provider = AirwallexProvider::new("c".into(), "k".into(), "secret".into());
            let result = provider
                .verify_webhook(WebhookEvent {
                    provider: "airwallex".into(),
                    payload: b"{}".to_vec(),
                    signature: "badsig".to_string(),
                })
                .await;
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), BillingError::WebhookVerification(_)));
        }

        #[test]
        fn provider_name() {
            assert_eq!(
                AirwallexProvider::new("c".into(), "k".into(), "w".into()).provider_name(),
                "airwallex"
            );
        }
    }

    // ══════════════════════════════════════════════════════════════════════
    // Revolut mock tests
    // ══════════════════════════════════════════════════════════════════════

    mod revolut_tests {
        use super::*;
        use ruxlog::services::billing::revolut::RevolutProvider;

        fn rev_mock(server: &MockServer) -> RevolutProvider {
            RevolutProvider::new("rev_api_key".into(), "rev_whsec".into())
                .with_base_url(server.uri())
        }

        #[tokio::test]
        async fn create_checkout() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/orders"))
                .and(header("Authorization", "Bearer rev_api_key"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "id": "ord_rev_1",
                    "checkout_url": format!("{}/rev/checkout", server.uri()),
                })))
                .mount(&server)
                .await;

            let result = rev_mock(&server)
                .create_checkout("9999", "user@test.com", 4, "https://s.cx/s", "https://s.cx/c")
                .await
                .expect("ok");
            assert_eq!(result.session_id, "ord_rev_1");
            assert!(result.checkout_url.contains("/rev/checkout"));
        }

        #[tokio::test]
        async fn cancel_subscription() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/subscriptions/sub_rev/cancel"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": "sub_rev"})))
                .mount(&server)
                .await;

            assert!(rev_mock(&server).cancel_subscription("sub_rev", true).await.is_ok());
        }

        #[tokio::test]
        async fn get_subscription() {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/subscriptions/sub_rev2"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "id": "sub_rev2",
                    "state": "active",
                    "current_period_end": "2026-12-31T00:00:00Z",
                })))
                .mount(&server)
                .await;

            let info = rev_mock(&server).get_subscription("sub_rev2").await.expect("ok");
            assert_eq!(info.status, "active");
            assert!(info.current_period_end.is_some());
        }

        #[tokio::test]
        async fn verify_webhook_valid_signature() {
            let secret = "rev_secret_123";
            let provider = RevolutProvider::new("k".into(), secret.into());
            let payload = json!({
                "event": "order.completed",
                "order": { "id": "ord_rev_w", "customer_id": "cus_rev" },
                "subscription": { "id": "sub_rev_w" }
            });
            let bytes = serde_json::to_vec(&payload).unwrap();
            let sig = sign_payload(secret, &bytes);

            let parsed = provider
                .verify_webhook(WebhookEvent {
                    provider: "revolut".into(),
                    payload: bytes,
                    signature: sig,
                })
                .await
                .expect("should verify");
            assert_eq!(parsed.event_type, "order.completed");
            assert_eq!(parsed.customer_id, "cus_rev");
            assert_eq!(parsed.subscription_id, Some("sub_rev_w".to_string()));
            assert_eq!(parsed.payment_id, Some("ord_rev_w".to_string()));
        }

        #[tokio::test]
        async fn verify_webhook_invalid_signature() {
            let provider = RevolutProvider::new("k".into(), "real_secret".into());
            let result = provider
                .verify_webhook(WebhookEvent {
                    provider: "revolut".into(),
                    payload: b"{}".to_vec(),
                    signature: "badsig".to_string(),
                })
                .await;
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), BillingError::WebhookVerification(_)));
        }

        #[tokio::test]
        async fn portal_session_returns_url() {
            let provider = RevolutProvider::new("k".into(), "w".into());
            let url = provider
                .create_portal_session("cus_rev", "https://app.com/account")
                .await
                .expect("ok");
            assert!(url.contains("cus_rev"));
            assert!(url.contains("return_url="));
        }

        #[test]
        fn provider_name() {
            assert_eq!(
                RevolutProvider::new("k".into(), "w".into()).provider_name(),
                "revolut"
            );
        }
    }

    // ══════════════════════════════════════════════════════════════════════
    // PayPal mock tests
    // ══════════════════════════════════════════════════════════════════════

    mod paypal_tests {
        use super::*;
        use ruxlog::services::billing::paypal::PayPalProvider;

        fn paypal_mock(server: &MockServer) -> PayPalProvider {
            PayPalProvider::new("pp_client".into(), "pp_secret".into(), "pp_whsec".into())
                .with_base_url(server.uri())
        }

        #[tokio::test]
        async fn create_checkout() {
            let server = MockServer::start().await;

            // Mock OAuth token
            Mock::given(method("POST"))
                .and(path("/v1/oauth2/token"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "access_token": "pp_bearer_tok",
                    "token_type": "Bearer",
                })))
                .mount(&server)
                .await;

            // Mock checkout order
            Mock::given(method("POST"))
                .and(path("/v2/checkout/orders"))
                .and(header("Authorization", "Bearer pp_bearer_tok"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "id": "pp_order_1",
                    "links": [
                        { "rel": "approve", "href": format!("{}/pp/approve", server.uri()) },
                        { "rel": "self", "href": format!("{}/pp/self", server.uri()) },
                    ],
                })))
                .mount(&server)
                .await;

            let result = paypal_mock(&server)
                .create_checkout("99.99", "user@test.com", 8, "https://s.cx/s", "https://s.cx/c")
                .await
                .expect("ok");
            assert_eq!(result.session_id, "pp_order_1");
            assert!(result.checkout_url.contains("/pp/approve"));
        }

        #[tokio::test]
        async fn cancel_subscription() {
            let server = MockServer::start().await;

            Mock::given(method("POST"))
                .and(path("/v1/oauth2/token"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({"access_token": "tok"})))
                .mount(&server)
                .await;

            Mock::given(method("POST"))
                .and(path("/v1/billing/subscriptions/sub_pp/cancel"))
                .respond_with(ResponseTemplate::new(204))
                .mount(&server)
                .await;

            assert!(paypal_mock(&server).cancel_subscription("sub_pp", true).await.is_ok());
        }

        #[tokio::test]
        async fn get_subscription() {
            let server = MockServer::start().await;

            Mock::given(method("POST"))
                .and(path("/v1/oauth2/token"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({"access_token": "tok"})))
                .mount(&server)
                .await;

            Mock::given(method("GET"))
                .and(path("/v1/billing/subscriptions/sub_pp2"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "id": "sub_pp2",
                    "status": "ACTIVE",
                    "billing_info": { "next_billing_time": "2026-12-31T00:00:00Z" },
                })))
                .mount(&server)
                .await;

            let info = paypal_mock(&server).get_subscription("sub_pp2").await.expect("ok");
            assert_eq!(info.provider_subscription_id, "sub_pp2");
            assert_eq!(info.status, "ACTIVE");
            assert!(info.current_period_end.is_some());
        }

        #[tokio::test]
        async fn verify_webhook_valid_signature() {
            let secret = "pp_secret_123";
            let provider = PayPalProvider::new("c".into(), "s".into(), secret.into());
            let payload = json!({
                "event_type": "PAYMENT.SALE.COMPLETED",
                "resource": {
                    "id": "pay_pp1",
                    "subscriber": { "payer_id": "cus_pp1" },
                },
            });
            let bytes = serde_json::to_vec(&payload).unwrap();
            let sig = sign_payload(secret, &bytes);

            let parsed = provider
                .verify_webhook(WebhookEvent {
                    provider: "paypal".into(),
                    payload: bytes,
                    signature: sig,
                })
                .await
                .expect("should verify");
            assert_eq!(parsed.event_type, "PAYMENT.SALE.COMPLETED");
            assert_eq!(parsed.customer_id, "cus_pp1");
        }

        #[tokio::test]
        async fn verify_webhook_invalid_signature() {
            let provider = PayPalProvider::new("c".into(), "s".into(), "secret".into());
            let result = provider
                .verify_webhook(WebhookEvent {
                    provider: "paypal".into(),
                    payload: b"{}".to_vec(),
                    signature: "badsig".to_string(),
                })
                .await;
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), BillingError::WebhookVerification(_)));
        }

        #[tokio::test]
        async fn portal_session_returns_url() {
            let provider = PayPalProvider::new("c".into(), "s".into(), "w".into());
            let url = provider
                .create_portal_session("cus_pp", "https://app.com/account")
                .await
                .expect("ok");
            assert!(url.contains("cus_pp"));
        }

        #[test]
        fn provider_name() {
            assert_eq!(
                PayPalProvider::new("c".into(), "s".into(), "w".into()).provider_name(),
                "paypal"
            );
        }
    }

    // ══════════════════════════════════════════════════════════════════════
    // BillingRouter integration tests
    // ══════════════════════════════════════════════════════════════════════

    mod router_tests {
        use super::*;
        use ruxlog::services::billing::router::{BillingRouter, GeoRouter, GeoRulesConfig};

        fn stripe_at_server(server: &MockServer) -> std::sync::Arc<dyn BillingProvider> {
            std::sync::Arc::new(
                StripeProvider::new("sk_test".into(), "whsec".into()).with_base_url(server.uri()),
            ) as std::sync::Arc<dyn BillingProvider>
        }

        fn make_router(
            providers: std::collections::HashMap<String, std::sync::Arc<dyn BillingProvider>>,
            default: &str,
        ) -> BillingRouter {
            let config = GeoRulesConfig {
                default_provider: default.to_string(),
                rules: vec![],
            };
            let geo = GeoRouter::new(config);
            BillingRouter::new(providers, geo)
        }

        #[test]
        fn router_provider_names_empty() {
            let providers: std::collections::HashMap<String, std::sync::Arc<dyn BillingProvider>> =
                std::collections::HashMap::new();
            let router = make_router(providers, "stripe");
            assert!(router.provider_names().is_empty());
        }

        #[test]
        fn router_has_provider_empty_map() {
            let providers: std::collections::HashMap<String, std::sync::Arc<dyn BillingProvider>> =
                std::collections::HashMap::new();
            let router = make_router(providers, "stripe");
            assert!(!router.has_provider("stripe"));
        }

        #[tokio::test]
        async fn router_verify_webhook_routes_to_correct_provider() {
            let secret = "router_whsec";
            let stripe_provider: std::sync::Arc<dyn BillingProvider> =
                std::sync::Arc::new(StripeProvider::new("sk".into(), secret.into()));
            let mut providers: std::collections::HashMap<String, std::sync::Arc<dyn BillingProvider>> =
                std::collections::HashMap::new();
            providers.insert("stripe".into(), stripe_provider);

            let router = make_router(providers, "stripe");

            let payload = json!({
                "type": "checkout.session.completed",
                "data": { "object": { "customer": "cus_1", "subscription": "sub_1" } }
            });
            let bytes = serde_json::to_vec(&payload).unwrap();
            let sig = sign_payload(secret, &bytes);

            let parsed = router
                .verify_webhook(WebhookEvent {
                    provider: "stripe".into(),
                    payload: bytes,
                    signature: sig,
                })
                .await
                .expect("should route and verify");
            assert_eq!(parsed.event_type, "checkout.session.completed");
        }

        #[tokio::test]
        async fn router_verify_webhook_unknown_provider_error() {
            let providers: std::collections::HashMap<String, std::sync::Arc<dyn BillingProvider>> =
                std::collections::HashMap::new();
            let router = make_router(providers, "stripe");

            let result = router
                .verify_webhook(WebhookEvent {
                    provider: "unknown_provider".into(),
                    payload: b"{}".to_vec(),
                    signature: String::new(),
                })
                .await;
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(matches!(err, BillingError::WebhookVerification(_)));
        }

        #[tokio::test]
        async fn router_default_provider_fallback() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/v1/checkout/sessions"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "id": "cs_fallback",
                    "url": format!("{}/fallback", server.uri()),
                })))
                .mount(&server)
                .await;

            let mut providers: std::collections::HashMap<String, std::sync::Arc<dyn BillingProvider>> =
                std::collections::HashMap::new();
            providers.insert("stripe".into(), stripe_at_server(&server));

            let router = make_router(providers, "stripe");

            // Calling the BillingProvider trait method (no IP) uses default
            let session = router
                .create_checkout("plan_1", "u@t.com", 1, "https://s", "https://c")
                .await
                .expect("ok");
            assert_eq!(session.session_id, "cs_fallback");
        }

        #[tokio::test]
        async fn router_default_provider_not_in_map_errors() {
            let providers: std::collections::HashMap<String, std::sync::Arc<dyn BillingProvider>> =
                std::collections::HashMap::new();
            let router = make_router(providers, "stripe");

            let result = router
                .create_checkout("plan", "u@t.com", 1, "https://s", "https://c")
                .await;
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), BillingError::Config(_)));
        }
    }

    // ══════════════════════════════════════════════════════════════════════
    // Crypto end-to-end tests
    // ══════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn crypto_checkout_generates_bip21_uri() {
        let provider = CryptoProvider::new(
            "bc1qtestwallet".into(),
            "https://api.blockcypher.com/v1".into(),
            "bcy_key".into(),
            "BTC".into(),
        );

        let session = provider
            .create_checkout(
                "0.001",
                "user@test.com",
                42,
                "https://s.cx/s",
                "https://s.cx/c",
            )
            .await
            .expect("ok");

        assert!(session.session_id.starts_with("rux-42-"));
        assert!(session.checkout_url.starts_with("bitcoin:bc1qtestwallet?"));
        assert!(session.checkout_url.contains("amount=0.001"));
        assert!(session.checkout_url.contains("label=rux-42-"));
    }

    #[tokio::test]
    async fn crypto_webhook_confirmed_flow() {
        let provider = CryptoProvider::new(
            "bc1qtestwallet".into(),
            "https://api.blockcypher.com/v1".into(),
            "bcy_key".into(),
            "BTC".into(),
        );

        // Simulate a BlockCypher webhook payload
        let webhook_payload = json!({
            "hash": "a1b2c3d4e5f6",
            "confirmations": 6,
            "address": "bc1qtestwallet",
            "value": 0.001,
            "memo": "rux-42-some-uuid"
        });
        let bytes = serde_json::to_vec(&webhook_payload).unwrap();

        let parsed = provider
            .verify_webhook(WebhookEvent {
                provider: "crypto".into(),
                payload: bytes,
                signature: String::new(),
            })
            .await
            .expect("should parse");

        assert_eq!(parsed.event_type, "payment.confirmed");
        assert_eq!(parsed.payment_id, Some("a1b2c3d4e5f6".to_string()));
        assert_eq!(parsed.data["memo"], "rux-42-some-uuid");
        assert_eq!(parsed.data["currency"], "BTC");
        assert_eq!(parsed.data["confirmations"], 6);
    }

    #[tokio::test]
    async fn crypto_webhook_pending_flow() {
        let provider = CryptoProvider::new(
            "0xEthWallet".into(),
            "https://api.etherscan.io".into(),
            "eth_key".into(),
            "ETH".into(),
        );

        let webhook_payload = json!({
            "hash": "0xethtx123",
            "confirmations": 5,
            "address": "0xEthWallet",
            "value": 0.5,
            "memo": "rux-7-tx-uuid"
        });
        let bytes = serde_json::to_vec(&webhook_payload).unwrap();

        let parsed = provider
            .verify_webhook(WebhookEvent {
                provider: "crypto".into(),
                payload: bytes,
                signature: String::new(),
            })
            .await
            .expect("should parse");

        // ETH requires 12 confirmations, 5 < 12 = pending
        assert_eq!(parsed.event_type, "payment.pending");
    }

    #[tokio::test]
    async fn crypto_multi_chain_uris() {
        let btc = CryptoProvider::new("bc1qaddr".into(), "url".into(), "k".into(), "BTC".into());
        let eth = CryptoProvider::new("0xAddr".into(), "url".into(), "k".into(), "ETH".into());
        let xmr = CryptoProvider::new(
            "4AMoneroAddr".into(),
            "url".into(),
            "k".into(),
            "XMR".into(),
        );
        let sol = CryptoProvider::new("SolAddr".into(), "url".into(), "k".into(), "SOL".into());

        let btc_session = btc
            .create_checkout("0.01", "u@t.com", 1, "https://s", "https://c")
            .await
            .expect("ok");
        let eth_session = eth
            .create_checkout("0.5", "u@t.com", 2, "https://s", "https://c")
            .await
            .expect("ok");
        let xmr_session = xmr
            .create_checkout("1.0", "u@t.com", 3, "https://s", "https://c")
            .await
            .expect("ok");
        let sol_session = sol
            .create_checkout("10", "u@t.com", 4, "https://s", "https://c")
            .await
            .expect("ok");

        assert!(btc_session.checkout_url.starts_with("bitcoin:"));
        assert!(eth_session.checkout_url.starts_with("ethereum:"));
        assert!(xmr_session.checkout_url.starts_with("monero:"));
        assert!(sol_session.checkout_url.starts_with("solana:"));
    }
}

#[cfg(not(feature = "billing"))]
mod billing_no_feature {
    #[test]
    fn billing_tests_require_feature() {
        eprintln!("NOTE: Run with --features full to enable billing provider tests");
    }
}
