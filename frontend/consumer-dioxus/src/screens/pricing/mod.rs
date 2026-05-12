use crate::config::BRAND;
use crate::seo::{breadcrumb_schema, SeoHead, SeoMetadataBuilder, StructuredData};
use dioxus::prelude::*;
use hmziq_dioxus_free_icons::icons::ld_icons::LdCheck;
use hmziq_dioxus_free_icons::Icon;

#[derive(Debug, Clone, serde::Deserialize)]
#[allow(dead_code)]
struct PlanData {
    id: i32,
    name: String,
    slug: String,
    description: Option<String>,
    price_cents: i32,
    currency: String,
    interval: String,
    trial_days: i32,
    features: Option<serde_json::Value>,
    is_active: bool,
}

#[component]
pub fn PricingScreen() -> Element {
    let mut plans = use_signal(Vec::<PlanData>::new);
    let mut loading = use_signal(|| true);
    let mut error_msg = use_signal(|| Option::<String>::None);

    let seo_metadata = SeoMetadataBuilder::new()
        .title("Pricing")
        .description(&format!(
            "Choose the right plan for you on {}",
            BRAND.app_name
        ))
        .canonical("/pricing")
        .build();

    use_effect(move || {
        spawn(async move {
            let response = oxcore::http::get("/billing/v1/plans").send().await;
            match response {
                Ok(resp) => {
                    if (200..300).contains(&resp.status()) {
                        match resp.json::<Vec<PlanData>>().await {
                            Ok(data) => {
                                plans.set(data);
                                loading.set(false);
                            }
                            Err(e) => {
                                error_msg.set(Some(format!("Failed to parse plans: {}", e)));
                                loading.set(false);
                            }
                        }
                    } else {
                        error_msg.set(Some("Plans are not available at this time.".to_string()));
                        loading.set(false);
                    }
                }
                Err(_) => {
                    error_msg.set(Some("Unable to connect to server.".to_string()));
                    loading.set(false);
                }
            }
        });
    });

    rsx! {
        SeoHead { metadata: seo_metadata }
        StructuredData {
            json_ld: breadcrumb_schema(vec![
                ("Home", "/"),
                ("Pricing", "/pricing"),
            ])
        }

        div { class: "min-h-screen",
            div { class: "container mx-auto px-4 py-8 md:py-12 lg:py-16 max-w-5xl",
                div { class: "text-center mb-12",
                    h1 { class: "text-4xl md:text-5xl font-bold mb-4", "Pricing" }
                    p { class: "text-lg text-muted-foreground max-w-xl mx-auto",
                        "Choose the plan that fits your reading needs."
                    }
                }

                if loading() {
                    div { class: "flex items-center justify-center py-20",
                        div { class: "animate-pulse text-muted-foreground", "Loading plans..." }
                    }
                } else if let Some(err) = error_msg() {
                    div { class: "text-center py-20",
                        p { class: "text-muted-foreground", "{err}" }
                    }
                } else if plans().is_empty() {
                    div { class: "text-center py-20",
                        h2 { class: "text-xl font-semibold mb-2", "No plans available" }
                        p { class: "text-muted-foreground",
                            "Check back later for subscription options."
                        }
                    }
                } else {
                    div { class: "grid md:grid-cols-{plans().len().min(3)} gap-6 justify-center",
                        for plan in plans().iter() {
                            {
                                let price_display = format!(
                                    "{}{:.2}",
                                    plan.currency,
                                    plan.price_cents as f64 / 100.0
                                );
                                let interval_display = match plan.interval.as_str() {
                                    "month" => "/month",
                                    "year" => "/year",
                                    other => other,
                                };
                                let feature_list: Vec<String> = match &plan.features {
                                    Some(serde_json::Value::Array(arr)) => arr
                                        .iter()
                                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                        .collect(),
                                    _ => vec![],
                                };

                                rsx! {
                                    div { class: "rounded-xl border border-border bg-card p-6 flex flex-col max-w-sm w-full",
                                        h3 { class: "text-lg font-semibold mb-1", "{plan.name}" }
                                        if let Some(desc) = &plan.description {
                                            p { class: "text-sm text-muted-foreground mb-4", "{desc}" }
                                        }
                                        div { class: "mb-6",
                                            span { class: "text-3xl font-bold", "{price_display}" }
                                            span { class: "text-muted-foreground", "{interval_display}" }
                                        }
                                        if plan.trial_days > 0 {
                                            p { class: "text-sm text-primary mb-4",
                                                "{plan.trial_days}-day free trial"
                                            }
                                        }
                                        if !feature_list.is_empty() {
                                            ul { class: "space-y-2 mb-6 flex-1",
                                                for feature in feature_list.iter() {
                                                    li { class: "flex items-start gap-2 text-sm",
                                                        Icon { icon: LdCheck, class: "w-4 h-4 text-primary mt-0.5 shrink-0" }
                                                        span { "{feature}" }
                                                    }
                                                }
                                            }
                                        }
                                        Link {
                                            to: crate::router::Route::ContactScreen {},
                                            class: "inline-flex items-center justify-center px-4 py-2 rounded-lg bg-primary text-primary-foreground font-medium hover:opacity-90 transition-opacity",
                                            "Get Started"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
