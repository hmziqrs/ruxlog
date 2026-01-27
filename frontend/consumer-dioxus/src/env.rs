pub const APP_API_URL: &str = match std::option_env!("SITE_URL") {
    Some(url) => url,
    None => "http://localhost:1100",
};

pub const APP_CSRF_TOKEN: &str = match std::option_env!("CSRF_KEY") {
    Some(key) => key,
    None => "dev-csrf-key",
};

// Firebase Analytics Configuration
#[cfg(feature = "analytics")]
pub const FIREBASE_API_KEY: &str = match std::option_env!("FIREBASE_API_KEY") {
    Some(key) => key,
    None => "",
};

#[cfg(feature = "analytics")]
pub const FIREBASE_AUTH_DOMAIN: &str = match std::option_env!("FIREBASE_AUTH_DOMAIN") {
    Some(domain) => domain,
    None => "",
};

#[cfg(feature = "analytics")]
pub const FIREBASE_PROJECT_ID: &str = match std::option_env!("FIREBASE_PROJECT_ID") {
    Some(id) => id,
    None => "",
};

#[cfg(feature = "analytics")]
pub const FIREBASE_STORAGE_BUCKET: &str = match std::option_env!("FIREBASE_STORAGE_BUCKET") {
    Some(bucket) => bucket,
    None => "",
};

#[cfg(feature = "analytics")]
pub const FIREBASE_MESSAGING_SENDER_ID: &str =
    match std::option_env!("FIREBASE_MESSAGING_SENDER_ID") {
        Some(id) => id,
        None => "",
    };

#[cfg(feature = "analytics")]
pub const FIREBASE_APP_ID: &str = match std::option_env!("FIREBASE_APP_ID") {
    Some(id) => id,
    None => "",
};

#[cfg(feature = "analytics")]
pub const FIREBASE_MEASUREMENT_ID: &str = match std::option_env!("FIREBASE_MEASUREMENT_ID") {
    Some(id) => id,
    None => "",
};
