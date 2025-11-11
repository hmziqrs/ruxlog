#[derive(Clone, PartialEq)]
pub struct DarkMode(pub bool);

impl DarkMode {
    pub fn toggle(&mut self) {
        self.0 = !self.0;
    }
}

// pub struct Config {
//     values: HashMap<String, String>,
// }

// static CONFIG: OnceLock<Config> = OnceLock::new();

// impl Config {
//     fn new() -> Self {
//         // // Try to load .env file
//         let _ = dotenv();

//         let mut values = HashMap::new();

//         // Add your environment variables here
//         if let Ok(api_url) = env::var("API_URL") {
//             values.insert("API_URL".to_string(), api_url);
//         } else {
//             // Default fallback value
//             values.insert("API_URL".to_string(), "localhost:8888".to_string());
//         }

//         Config { values }
//     }

//     pub fn get(key: &str) -> Option<String> {
//         let config = CONFIG.get_or_init(Config::new);
//         config.values.get(key).cloned()
//     }

//     pub fn api_url() -> String {
//         Self::get("API_URL").unwrap_or_else(|| "localhost:8888".to_string())
//     }

//     pub fn api_base_url() -> String {
//         format!("http://{}", Self::api_url())
//     }
// }
