pub mod controller;
pub mod validator;

use axum::{
    extract::DefaultBodyLimit,
    middleware,
    routing::{get, post},
    Router,
};

use crate::{config, middlewares::auth_guard, AppState};

pub fn routes() -> Router<AppState> {
    let admin = Router::<AppState>::new()
        // Plan CRUD
        .route("/plan/list", post(controller::admin_list_plans))
        .route("/plan/create", post(controller::admin_create_plan))
        .route(
            "/plan/update/{plan_id}",
            post(controller::admin_update_plan),
        )
        .route(
            "/plan/delete/{plan_id}",
            post(controller::admin_delete_plan),
        )
        // Subscription management
        .route(
            "/subscription/list",
            post(controller::admin_list_subscriptions),
        )
        .route(
            "/subscription/cancel/{subscription_id}",
            post(controller::admin_cancel_subscription),
        )
        // Payments & invoices
        .route("/payment/list", post(controller::admin_list_payments))
        .route("/invoice/list", post(controller::admin_list_invoices))
        // Discount codes
        .route(
            "/discount/list",
            post(controller::admin_list_discount_codes),
        )
        .route(
            "/discount/create",
            post(controller::admin_create_discount_code),
        )
        .route(
            "/discount/delete/{code_id}",
            post(controller::admin_delete_discount_code),
        )
        // Paywall: set post access
        .route(
            "/post/access/{post_id}",
            post(controller::admin_set_post_access),
        )
        .route_layer(middleware::from_fn(
            auth_guard::verified_with_role::<{ auth_guard::ROLE_ADMIN }>,
        ));

    let authenticated = Router::<AppState>::new()
        // Checkout
        .route("/checkout", post(controller::create_checkout))
        // Per-post one-time purchase checkout
        .route("/checkout/post", post(controller::create_post_checkout))
        // My subscriptions
        .route("/subscriptions", get(controller::my_subscriptions))
        // My payments
        .route("/payments", get(controller::my_payments))
        .route_layer(middleware::from_fn(auth_guard::authenticated));

    let public = Router::<AppState>::new()
        // Public active plans listing
        .route("/plans", get(controller::public_list_plans))
        // Paywall check (no auth required — tells client if access is needed)
        .route("/access/{post_id}", get(controller::check_post_access))
        // Webhook receiver (per-provider path)
        .route("/webhook/{provider}", post(controller::webhook_receiver));

    // Cap request bodies across every billing route. The PUBLIC webhook receiver
    // (/webhook/{provider}) is the key surface: axum 0.8 applies no default body
    // limit, so without this an attacker could POST an unbounded body to exhaust
    // handler memory (CWE-400). 64 KiB is generous for provider webhooks and
    // billing JSON, which are all small.
    public
        .merge(authenticated)
        .merge(admin)
        .layer(DefaultBodyLimit::max(config::body_limits::DEFAULT))
}
