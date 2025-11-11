# Dioxus Project Structure Guide

## Project Setup

### Prerequisites
1. Rust toolchain installed
2. Dioxus CLI: `cargo install dioxus-cli`

### Initial Setup
```bash
# Create new Dioxus project
dx new my-project
cd my-project
```

## Project Structure

```
project/
├─ assets/                    # Static assets (images, fonts, etc.)
├─ src/
│  ├─ main.rs                # Application entry point
│  ├─ router.rs              # Route definitions
│  ├─ config.rs              # Configuration settings
│  ├─ env.rs                 # Environment variables
│  ├─ components/            # App related UI components
│  │  └─ mod.rs
│  ├─ containers/            # App related Container components (logic + UI)
│  │  └─ mod.rs
│  ├─ screens/               # Page-level components
│  │  └─ mod.rs
│  ├─ hooks/                 # Custom hooks
│  │  ├─ mod.rs
│  │  ├─ use_form.rs
│  │  └─ use_previous.rs
│  ├─ services/              # API and external service clients
│  │  ├─ mod.rs
│  │  └─ http_client.rs
│  ├─ store/                 # State management
│  │  ├─ mod.rs
│  │  ├─ lib.rs
│  │  └─ [feature]/
│  │     ├─ mod.rs
│  │     ├─ state.rs
│  │     └─ actions.rs
│  └─ ui/                    # general purpose UI component library
│     ├─ mod.rs
├─ Cargo.toml                # Rust dependencies
├─ Dioxus.toml               # Dioxus configuration
└─ index.html                # HTML template
```

## Key Dependencies

### Core Dependencies (Cargo.toml)
```toml
[dependencies]
dioxus = { version = "0.7.0-alpha.0", features = ["router"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
validator = { version = "0.20", features = ["derive"] }
reqwest = { version = "0.12", features = ["json", "cookies"] }
chrono = { version = "0.4" }

# Web-specific
wasm-bindgen = "0.2"
web-sys = { version = "0.3", features = ["Document", "Window", "Element"] }
gloo-storage = "0.3"
gloo-timers = { version = "0.3", features = ["futures"] }
gloo-net = "0.6"

# UI and icons
hmziq-dioxus-free-icons = { version = "0.9", features = ["lucide", "simple-icons"] }
dioxus-toast = { version = "0.6", features = ["web"] }

[features]
default = ["web"]
web = ["dioxus/web"]
desktop = ["dioxus/desktop"]
mobile = ["dioxus/mobile"]
```

## Global State Management Pattern

### StateFrame - Async Operation State Wrapper

```rust
// src/store/lib.rs
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum StateFrameStatus {
    Init,
    Loading,
    Success,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateFrame<T: Clone> {
    pub status: StateFrameStatus,
    pub data: Option<T>,
    pub message: Option<String>,
}

impl<T: Clone> StateFrame<T> {
    pub fn new() -> Self {
        Self {
            status: StateFrameStatus::Init,
            data: None,
            message: None,
        }
    }

    pub fn is_loading(&self) -> bool {
        self.status == StateFrameStatus::Loading
    }

    pub fn is_success(&self) -> bool {
        self.status == StateFrameStatus::Success
    }

    pub fn set_loading(&mut self, message: Option<String>) {
        self.status = StateFrameStatus::Loading;
        self.message = message;
    }

    pub fn set_success(&mut self, data: Option<T>, message: Option<String>) {
        self.status = StateFrameStatus::Success;
        self.data = data;
        self.message = message;
    }

    pub fn set_failed(&mut self, message: Option<String>) {
        self.status = StateFrameStatus::Failed;
        self.message = message;
    }
}
```

### Store Implementation Pattern

```rust
// src/store/auth/state.rs
use std::sync::OnceLock;
use dioxus::prelude::*;
use crate::store::StateFrame;

pub struct AuthState {
    pub user: GlobalSignal<Option<User>>,
    pub login_status: GlobalSignal<StateFrame<bool>>,
    pub logout_status: GlobalSignal<StateFrame<bool>>,
    pub init_status: GlobalSignal<StateFrame<bool>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub avatar: Option<String>,
    pub is_verified: bool,
    pub role: UserRole,
}

static AUTH_STATE: OnceLock<AuthState> = OnceLock::new();

pub fn use_auth() -> &'static AuthState {
    AUTH_STATE.get_or_init(|| AuthState::new())
}
```

### Store Actions Implementation

```rust
// src/store/auth/actions.rs
impl AuthState {
    pub fn new() -> Self {
        AuthState {
            user: GlobalSignal::new(|| None),
            login_status: GlobalSignal::new(|| StateFrame::<bool>::new()),
            logout_status: GlobalSignal::new(|| StateFrame::<bool>::new()),
            init_status: GlobalSignal::new(|| StateFrame::<bool>::new()),
        }
    }

    pub async fn login(&self, email: String, password: String) {
        self.login_status.write().set_loading(None);

        let payload = LoginPayload { email, password };
        let result = http_client::post("/auth/v1/log_in", &payload).send().await;

        match result {
            Ok(response) => {
                if (200..300).contains(&response.status()) {
                    match response.json::<User>().await {
                        Ok(user) => {
                            *self.user.write() = Some(user);
                            self.login_status.write().set_success(None, None);
                        }
                        Err(e) => {
                            self.login_status.write().set_failed(Some(format!("Failed to parse user data: {}", e)));
                        }
                    }
                } else {
                    self.login_status.write().set_api_error(&response).await;
                }
            }
            Err(e) => {
                self.login_status.write().set_failed(Some(format!("Network error: {}", e)));
            }
        }
    }

    pub async fn logout(&self) {
        self.logout_status.write().set_loading(None);
        let result = http_client::post("/auth/v1/log_out", &{}).send().await;

        match result {
            Ok(response) => {
                if (200..300).contains(&response.status()) {
                    self.logout_status.write().set_success(None, None);
                    *self.user.write() = None;
                } else {
                    self.logout_status.write().set_api_error(&response).await;
                    *self.user.write() = None;
                }
            }
            Err(e) => {
                self.logout_status.write().set_failed(Some(e.to_string()));
                *self.user.write() = None;
            }
        }
    }
}
```

### Using Store in Components

```rust
#[component]
fn LoginForm() -> Element {
    let auth = use_auth();
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);

    // React to login status changes
    let login_status = auth.login_status.read();

    rsx! {
        div {
            if login_status.is_loading() {
                "Logging in..."
            }

            if let Some(error) = &login_status.message {
                div { class: "error",
                    "{error}"
                }
            }

            input {
                placeholder: "Email",
                value: "{email}",
                oninput: move |e| email.set(e.value().clone())
            }

            input {
                r#type: "password",
                placeholder: "Password",
                value: "{password}",
                oninput: move |e| password.set(e.value().clone())
            }

            button {
                disabled: login_status.is_loading(),
                onclick: move |_| {
                    let auth = auth.clone();
                    let email = email.read().clone();
                    let password = password.read().clone();
                    spawn(async move {
                        auth.login(email, password).await;
                    });
                },
                "Login"
            }
        }
    }
}
```

## Form Management Pattern

### Form State Structure

```rust
// src/hooks/use_form.rs
#[derive(Debug, Clone)]
pub struct OxFieldFrame {
    pub name: String,
    pub value: String,
    pub error: Option<String>,
    default_value: String,
    focused: bool,
    touched: bool,
    dirty: bool,
}

pub trait OxFormModel: Validate + Clone + PartialEq {
    fn to_map(&self) -> HashMap<String, String>;
    fn update_field(&mut self, name: String, value: &str);
}

#[derive(Debug, Clone)]
pub struct OxForm<T: OxFormModel> {
    pub data: T,
    pub fields: HashMap<String, OxFieldFrame>,
    pub has_errors: bool,
    pub active_field: Option<String>,
    pub submit_count: u32,
}

impl<T: OxFormModel> OxForm<T> {
    pub fn new(data: T) -> Self {
        let initial_map = data.to_map();
        let mut fields = HashMap::new();

        for (name, value) in &initial_map {
            fields.insert(name.clone(), OxFieldFrame::new(name.clone(), value.clone()));
        }

        OxForm {
            data,
            fields,
            active_field: None,
            has_errors: false,
            submit_count: 0,
        }
    }

    pub fn update_field(&mut self, name: &str, value: String) {
        // Update field state
        if let Some(field) = self.fields.get_mut(name) {
            field.value = value.clone();
            field.dirty = field.value != field.default_value;
        }

        // Update data model
        self.data.update_field(name.to_string(), &value);

        // Validate after update
        self.validate();
    }

    pub fn validate(&mut self) -> bool {
        if self.submit_count == 0 {
            return false;
        }

        // Clear previous errors
        for field in self.fields.values_mut() {
            field.set_error(None);
        }

        // Validate using validator crate
        match self.data.validate() {
            Ok(()) => {
                self.has_errors = false;
                true
            }
            Err(errors) => {
                self.apply_validation_errors(errors);
                self.has_errors = true;
                false
            }
        }
    }

    pub fn on_submit(&mut self, callback: impl Fn(T)) {
        self.submit_count += 1;
        if self.validate() {
            callback(self.data.clone());
        }
    }
}
```

### Form Data Model

```rust
// src/containers/user_form/form.rs
use validator::Validate;
use crate::hooks::{OxFormModel, OxForm};

#[derive(Debug, Validate, Clone, PartialEq)]
pub struct UserForm {
    #[validate(length(min = 1, message = "Name is required"))]
    pub name: String,

    #[validate(email(message = "Invalid email address"))]
    pub email: String,

    pub role: String,
    pub is_verified: bool,

    #[validate(custom(function = "validate_password"))]
    pub password: Option<String>,

    pub is_update: bool,
}

fn validate_password(password: &&String) -> Result<(), ValidationError> {
    if password.is_empty() {
        Err(ValidationError::new("Password cannot be empty"))
    } else {
        Ok(())
    }
}

impl UserForm {
    pub fn new() -> Self {
        UserForm {
            name: String::new(),
            email: String::new(),
            role: "user".to_string(),
            is_verified: false,
            password: Some(String::new()),
            is_update: false,
        }
    }
}

impl OxFormModel for UserForm {
    fn to_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("name".to_string(), self.name.clone());
        map.insert("email".to_string(), self.email.clone());
        map.insert("role".to_string(), self.role.clone());
        map.insert("is_verified".to_string(), self.is_verified.to_string());

        if let Some(password) = &self.password {
            map.insert("password".to_string(), password.clone());
        }

        map
    }

    fn update_field(&mut self, name: String, value: &str) {
        match name.as_str() {
            "name" => self.name = value.to_string(),
            "email" => self.email = value.to_string(),
            "role" => self.role = value.to_string(),
            "is_verified" => self.is_verified = value.parse().unwrap_or(false),
            "password" => self.password = Some(value.to_string()),
            _ => {}
        }
    }
}

// Hook to create form signal
pub fn use_user_form(initial_state: UserForm) -> Signal<OxForm<UserForm>> {
    use_signal(|| OxForm::new(initial_state))
}
```

### Form Component Usage

```rust
#[component]
fn UserFormComponent() -> Element {
    let mut form = use_user_form(UserForm::new());

    rsx! {
        form {
            onsubmit: move |e| {
                e.prevent_default();
                form.write().on_submit(|user_data| {
                    // Handle form submission
                    println!("User data: {:?}", user_data);
                });
            },

            div {
                label { "Name:" }
                input {
                    value: "{form.read().data.name}",
                    oninput: move |e| {
                        form.write().update_field("name", e.value().clone());
                    },
                    onfocus: move |_| {
                        form.write().focus_field("name");
                    },
                    onblur: move |_| {
                        form.write().blur_field("name");
                    }
                }

                if let Some(field) = form.read().get_field("name") {
                    if let Some(error) = &field.error {
                        div { class: "error", "{error}" }
                    }
                }
            }

            div {
                label { "Email:" }
                input {
                    r#type: "email",
                    value: "{form.read().data.email}",
                    oninput: move |e| {
                        form.write().update_field("email", e.value().clone());
                    }
                }

                if let Some(field) = form.read().get_field("email") {
                    if let Some(error) = &field.error {
                        div { class: "error", "{error}" }
                    }
                }
            }

            button {
                r#type: "submit",
                disabled: !form.read().is_valid(),
                "Submit"
            }
        }
    }
}
```

## Custom Hooks

### Previous Value Hook

```rust
// src/hooks/use_previous.rs
use dioxus::prelude::*;
use std::{cell::RefCell, rc::Rc};

pub fn use_previous<T>(current: T) -> Option<T>
where
    T: Clone + PartialEq + 'static,
{
    let state_ref = use_hook(|| Rc::new(RefCell::new(None::<T>)));
    let previous_value = state_ref.borrow().clone();

    use_effect(use_reactive!(|(current,)| {
        *state_ref.borrow_mut() = Some(current);
    }));

    previous_value
}

// Usage example
#[component]
fn Counter() -> Element {
    let mut count = use_signal(|| 0);
    let previous_count = use_previous(count.read().clone());

    rsx! {
        div {
            "Current: {count}"
            if let Some(prev) = previous_count {
                " Previous: {prev}"
            }

            button {
                onclick: move |_| count += 1,
                "Increment"
            }
        }
    }
}
```

## Router Configuration

### Route Definition

```rust
// src/router.rs
use dioxus::prelude::*;

#[derive(Debug, Clone, Routable, PartialEq)]
pub enum Route {
    #[layout(AuthGuardContainer)]
    #[layout(NavBarContainer)]
    #[route("/")]
    HomeScreen {},

    #[route("/login")]
    LoginScreen {},

    #[route("/user/new")]
    AddUserScreen {},

    #[route("/user/list")]
    UserListScreen {},
}
```

### Main App Structure

```rust
// src/main.rs
use dioxus::prelude::*;

pub mod components;
pub mod containers;
pub mod hooks;
pub mod router;
pub mod screens;
pub mod services;
pub mod store;
pub mod ui;
mod config;
mod env;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        Router::<crate::router::Route> {}
    }
}
```

## Best Practices

1. **Global State**: Use OnceLock + GlobalSignal pattern for global state management
2. **Async Operations**: Wrap API calls with StateFrame for consistent loading states
3. **Forms**: Use OxForm pattern with validator crate for robust form handling
4. **Component Hierarchy**: Separate concerns: components (UI), containers (logic), screens (pages)
5. **Custom Hooks**: Extract reusable logic into custom hooks
6. **Type Safety**: Leverage Rust's type system with proper serialization/deserialization

This structure provides a production-ready foundation for scalable Dioxus applications with proper state management, form handling, and routing patterns.
