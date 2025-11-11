pub mod http_client;
#[cfg(not(target_arch = "wasm32"))]
pub mod reqwest_client;

// We can either export all contents like this:
// pub use http_client::*;
// Or let users explicitly import from the public module
