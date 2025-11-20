use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use tokio::{sync::mpsc, time::sleep};
use ratatui::Terminal;
use tuirealm::terminal::{CrosstermTerminalAdapter, TerminalBridge};

use ruxlog::core::{
    auth::AuthService,
    config::CoreConfig,
    context::CoreContext,
    db::init_db,
    redis::init_redis_pool,
    tags::TagService,
    types::{AuthError, Session, TagError, TagSummary, UserCredentials},
};

#[derive(Parser, Debug)]
#[command(name = "ruxlog_tui", about = "Ruxlog TUI (auth + tags)")]
struct Args {
    /// Log level (e.g. info, debug, trace)
    #[arg(long, default_value = "info")]
    _log_level: String,
}

#[derive(Debug, Clone, Copy)]
enum AppRoute {
    Login,
    Tags,
}

#[derive(Debug, Clone, Copy)]
enum LoginField {
    Username,
    Password,
    Submit,
}

#[derive(Debug, Clone)]
struct LoginState {
    username_input: String,
    password_input: String,
    focused_field: LoginField,
    is_loading: bool,
    error: Option<String>,
}

impl Default for LoginState {
    fn default() -> Self {
        Self {
            username_input: String::new(),
            password_input: String::new(),
            focused_field: LoginField::Username,
            is_loading: false,
            error: None,
        }
    }
}

#[derive(Debug, Clone)]
struct TagsState {
    tags: Vec<TagSummary>,
    selected_index: usize,
    is_loading: bool,
    error: Option<String>,
}

impl Default for TagsState {
    fn default() -> Self {
        Self {
            tags: Vec::new(),
            selected_index: 0,
            is_loading: false,
            error: None,
        }
    }
}

#[derive(Debug)]
enum AppEvent {
    LoginResult(Result<Session, AuthError>),
    TagsLoaded(Result<Vec<TagSummary>, TagError>),
}

struct App {
    route: AppRoute,
    core: Arc<CoreContext>,
    auth_service: AuthService,
    tag_service: TagService,
    session: Option<Session>,
    login: LoginState,
    tags: TagsState,
    should_quit: bool,
    logs: Vec<String>,
}

impl App {
    fn new(core: Arc<CoreContext>) -> Self {
        let auth_service = AuthService::new(core.clone());
        let tag_service = TagService::new(core.clone());

        Self {
            route: AppRoute::Login,
            core,
            auth_service,
            tag_service,
            session: None,
            login: LoginState::default(),
            tags: TagsState::default(),
            should_quit: false,
            logs: Vec::new(),
        }
    }

    fn push_log<S: Into<String>>(&mut self, line: S) {
        const MAX_LOG_LINES: usize = 200;
        self.logs.push(line.into());
        if self.logs.len() > MAX_LOG_LINES {
            let excess = self.logs.len() - MAX_LOG_LINES;
            self.logs.drain(0..excess);
        }
    }

    fn handle_key(&mut self, key: KeyEvent, tx: &mpsc::UnboundedSender<AppEvent>) {
        match self.route {
            AppRoute::Login => self.handle_key_login(key, tx),
            AppRoute::Tags => self.handle_key_tags(key, tx),
        }
    }

    fn handle_key_login(&mut self, key: KeyEvent, tx: &mpsc::UnboundedSender<AppEvent>) {
        // When an error modal is shown, any key dismisses it.
        if self.login.error.is_some() {
            self.login.error = None;
            return;
        }

        if self.login.is_loading {
            if key.code == KeyCode::Esc {
                self.should_quit = true;
            }
            return;
        }

        match key.code {
            KeyCode::Esc => {
                self.should_quit = true;
            }
            KeyCode::Tab => {
                self.login.focused_field = match self.login.focused_field {
                    LoginField::Username => LoginField::Password,
                    LoginField::Password => LoginField::Submit,
                    LoginField::Submit => LoginField::Username,
                };
            }
            KeyCode::BackTab => {
                self.login.focused_field = match self.login.focused_field {
                    LoginField::Username => LoginField::Submit,
                    LoginField::Password => LoginField::Username,
                    LoginField::Submit => LoginField::Password,
                };
            }
            KeyCode::Enter => {
                if matches!(self.login.focused_field, LoginField::Submit) {
                    self.submit_login(tx);
                } else {
                    self.login.focused_field = match self.login.focused_field {
                        LoginField::Username => LoginField::Password,
                        LoginField::Password => LoginField::Submit,
                        LoginField::Submit => LoginField::Submit,
                    };
                }
            }
            KeyCode::Char(c) => match self.login.focused_field {
                LoginField::Username => self.login.username_input.push(c),
                LoginField::Password => self.login.password_input.push(c),
                LoginField::Submit => {}
            },
            KeyCode::Backspace => match self.login.focused_field {
                LoginField::Username => {
                    self.login.username_input.pop();
                }
                LoginField::Password => {
                    self.login.password_input.pop();
                }
                LoginField::Submit => {}
            },
            _ => {}
        }
    }

    fn handle_key_tags(&mut self, key: KeyEvent, tx: &mpsc::UnboundedSender<AppEvent>) {
        // When an error modal is shown, allow reload/logout or dismiss.
        if self.tags.error.is_some() {
            match key.code {
                KeyCode::Char('r') | KeyCode::Char('R') => {
                    self.load_tags(tx);
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    self.logout_to_login();
                }
                _ => {
                    self.tags.error = None;
                }
            }
            return;
        }

        if self.tags.is_loading {
            if key.code == KeyCode::Esc || key.code == KeyCode::Char('q') {
                self.logout_to_login();
            }
            return;
        }

        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.logout_to_login();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if !self.tags.tags.is_empty() && self.tags.selected_index > 0 {
                    self.tags.selected_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.tags.tags.is_empty()
                    && self.tags.selected_index + 1 < self.tags.tags.len()
                {
                    self.tags.selected_index += 1;
                }
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                self.load_tags(tx);
            }
            _ => {}
        }
    }

    fn logout_to_login(&mut self) {
        self.session = None;
        self.route = AppRoute::Login;
        self.login = LoginState::default();
    }

    fn submit_login(&mut self, tx: &mpsc::UnboundedSender<AppEvent>) {
        if self.login.is_loading {
            return;
        }

        self.login.is_loading = true;
        self.login.error = None;

        let creds = UserCredentials {
            username: self.login.username_input.clone(),
            password: self.login.password_input.clone(),
        };

        let auth = self.auth_service.clone();
        let tx_clone = tx.clone();

        tokio::spawn(async move {
            let result = auth.login(creds).await;
            let _ = tx_clone.send(AppEvent::LoginResult(result));
        });
    }

    fn load_tags(&mut self, tx: &mpsc::UnboundedSender<AppEvent>) {
        if self.tags.is_loading {
            return;
        }

        self.tags.is_loading = true;
        self.tags.error = None;

        let tag_service = self.tag_service.clone();
        let tx_clone = tx.clone();

        tokio::spawn(async move {
            let result = tag_service.list_tags().await;
            let _ = tx_clone.send(AppEvent::TagsLoaded(result));
        });
    }

    fn handle_app_event(&mut self, event: AppEvent, tx: &mpsc::UnboundedSender<AppEvent>) {
        match event {
            AppEvent::LoginResult(result) => {
                self.login.is_loading = false;
                match result {
                    Ok(session) => {
                        self.push_log(format!("login ok: user_id={}", session.user_id));
                        self.session = Some(session);
                        self.route = AppRoute::Tags;
                        self.tags = TagsState::default();
                        self.load_tags(tx);
                    }
                    Err(err) => {
                        self.push_log(format!("login error: {}", err));
                        self.login.error = Some(err.to_string());
                    }
                }
            }
            AppEvent::TagsLoaded(result) => {
                self.tags.is_loading = false;
                match result {
                    Ok(tags) => {
                        self.push_log(format!("tags loaded: {} items", tags.len()));
                        self.tags.tags = tags;
                        if !self.tags.tags.is_empty() {
                            self.tags.selected_index = 0;
                        }
                    }
                    Err(err) => {
                        self.push_log(format!("tags load error: {}", err));
                        self.tags.error = Some(err.to_string());
                    }
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();

    let args = Args::parse();
    let _ = args;

    let core_config = CoreConfig::from_env();
    let db = init_db(&core_config).await;
    let redis = init_redis_pool(&core_config).await;
    let core = Arc::new(CoreContext {
        config: core_config,
        db,
        redis,
    });

    if let Err(err) = run_tui(core).await {
        eprintln!("Error: {}", err);
    }

    Ok(())
}

async fn run_tui(core: Arc<CoreContext>) -> Result<(), Box<dyn Error>> {
    let mut bridge =
        TerminalBridge::init_crossterm().map_err(|e| format!("terminal init error: {e}"))?;
    let res = run_app(bridge.raw_mut(), core).await;
    bridge
        .restore()
        .map_err(|e| format!("terminal restore error: {e}"))?;
    res
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    core: Arc<CoreContext>,
) -> Result<(), Box<dyn Error>> {
    let (tx, mut rx) = mpsc::unbounded_channel::<AppEvent>();
    let mut app = App::new(core);

    loop {
        terminal.draw(|f| {
            let root = f.area();
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Min(1),
                        Constraint::Length(6),
                    ]
                    .as_ref(),
                )
                .split(root);

            match app.route {
                AppRoute::Login => draw_login(f, layout[0], &app.login),
                AppRoute::Tags => draw_tags(f, layout[0], &app),
            }

            draw_logs(f, layout[1], &app.logs);
        })?;

        while let Ok(evt) = rx.try_recv() {
            app.handle_app_event(evt, &tx);
        }

        if app.should_quit {
            break;
        }

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                app.handle_key(key, &tx);
            }
        } else {
            sleep(Duration::from_millis(50)).await;
        }
    }

    Ok(())
}

fn draw_login(
    f: &mut ratatui::Frame,
    area: Rect,
    state: &LoginState,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(7),
                Constraint::Length(3),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(area);

    let title = Paragraph::new(Line::from(Span::styled(
        "ruxlog TUI - Login",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )))
    .alignment(Alignment::Center);
    f.render_widget(title, chunks[0]);

    let form_block = Block::default()
        .borders(Borders::ALL)
        .title("Credentials");
    let form_area = chunks[1];
    f.render_widget(form_block, form_area);

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(2),
                Constraint::Length(2),
                Constraint::Length(1),
            ]
            .as_ref(),
        )
        .split(form_area);

    let username_label = "Email:";
    let username_value = &state.username_input;
    let username_style = if matches!(state.focused_field, LoginField::Username) {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let username = Paragraph::new(Line::from(vec![
        Span::styled(username_label, username_style),
        Span::raw(" "),
        Span::raw(username_value),
    ]));
    f.render_widget(username, inner[0]);

    let password_label = "Password:";
    let masked = "•".repeat(state.password_input.chars().count());
    let password_style = if matches!(state.focused_field, LoginField::Password) {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let password = Paragraph::new(Line::from(vec![
        Span::styled(password_label, password_style),
        Span::raw(" "),
        Span::raw(masked),
    ]));
    f.render_widget(password, inner[1]);

    let submit_text = if state.is_loading {
        "Logging in..."
    } else {
        "Press Enter to login"
    };
    let submit_style = if matches!(state.focused_field, LoginField::Submit) {
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Green)
    };
    let submit = Paragraph::new(Span::styled(submit_text, submit_style)).alignment(Alignment::Center);
    f.render_widget(submit, inner[2]);

    if let Some(err) = &state.error {
        let area = centered_rect(60, 25, area);
        let block = Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(
                "Login Error",
                Style::default()
                    .fg(Color::Red)
                    .add_modifier(Modifier::BOLD),
            ));
        let lines = vec![
            Line::from(err.as_str()),
            Line::from(""),
            Line::from("Press any key to continue"),
        ];
        let error = Paragraph::new(lines)
            .block(block)
            .alignment(Alignment::Center);
        f.render_widget(error, area);
    }

    let footer = Paragraph::new("Tab to switch fields, Enter to submit, Esc to quit.")
        .alignment(Alignment::Center);
    f.render_widget(footer, chunks[3]);
}

fn draw_tags(
    f: &mut ratatui::Frame,
    area: Rect,
    app: &App,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(2),
            ]
            .as_ref(),
        )
        .split(area);

    let header_text = match &app.session {
        Some(session) => format!("Tags - User ID {}", session.user_id),
        None => "Tags".to_string(),
    };
    let header = Paragraph::new(header_text)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    f.render_widget(header, chunks[0]);

    if app.tags.is_loading && app.tags.tags.is_empty() {
        let loading = Paragraph::new("Loading tags...")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Tags"));
        f.render_widget(loading, chunks[1]);
    } else if let Some(err) = &app.tags.error {
        let area = centered_rect(60, 25, area);
        let block = Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(
                "Failed to load tags",
                Style::default()
                    .fg(Color::Red)
                    .add_modifier(Modifier::BOLD),
            ));
        let lines = vec![
            Line::from(err.as_str()),
            Line::from(""),
            Line::from("Press R to retry, Q/Esc to logout, any other key to dismiss"),
        ];
        let error = Paragraph::new(lines)
            .block(block)
            .alignment(Alignment::Center);
        f.render_widget(error, area);
    } else {
        let items: Vec<ListItem> = app
            .tags
            .tags
            .iter()
            .map(|t| {
                let content = format!("{} ({})", t.name, t.slug);
                ListItem::new(content)
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Tags"))
            .highlight_style(
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            );

        let mut state = ratatui::widgets::ListState::default();
        if !app.tags.tags.is_empty() {
            state.select(Some(app.tags.selected_index));
        }

        f.render_stateful_widget(list, chunks[1], &mut state);
    }

    let footer_text = "[↑/↓ or j/k] navigate  [R] reload  [Q/Esc] logout";
    let footer = Paragraph::new(footer_text).alignment(Alignment::Center);
    f.render_widget(footer, chunks[2]);
}

fn draw_logs(f: &mut ratatui::Frame, area: Rect, logs: &[String]) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Logs");

    let lines: Vec<Line> = logs
        .iter()
        .rev()
        .take(3)
        .rev()
        .map(|l| Line::from(l.as_str()))
        .collect();

    let paragraph = Paragraph::new(lines).block(block);

    f.render_widget(paragraph, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    let vertical_chunk = popup_layout[1];

    let horizontal_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(vertical_chunk);

    horizontal_layout[1]
}
