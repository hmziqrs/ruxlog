use std::{error::Error, fs::OpenOptions, io::Write, path::PathBuf, sync::Arc, time::Duration};

use crossterm::event::{self, Event, KeyCode, KeyEvent};
use chrono::Utc;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::Style,
    Terminal,
};
use sea_orm::{DatabaseConnection, EntityTrait, QueryOrder};
use tokio::{sync::mpsc, time::sleep};
use tower_sessions_redis_store::fred::prelude::Pool as RedisPool;
use tuirealm::terminal::TerminalBridge;

use crate::{
    db::{
        sea_connect::init_db,
        sea_models::{tag, user},
    },
    services::{
        redis::init_redis_pool_only,
        seed::{self, SeedOutcome, SeedOutcomeRow, UndoOutcome},
    },
};

use super::{
    components::logs::draw_logs,
    screens::{
        home::draw_home, seed_history::draw_seed_history, seed_menu::draw_seed_menu,
        seed_progress::draw_seed_progress, seed_summary::draw_seed_summary, seed_undo::draw_seed_undo,
        tags::draw_tags, users::draw_users,
    },
    theme::{theme_palette, ThemeKind},
};

#[derive(Clone)]
pub struct CoreState {
    pub db: DatabaseConnection,
    pub redis: RedisPool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppRoute {
    Home,
    Tags,
    Users,
    SeedMenu,
    SeedProgress,
    SeedSummary,
    SeedHistory,
    SeedUndoConfirm,
}

#[derive(Debug, Clone)]
pub struct TagsState {
    pub tags: Vec<tag::Model>,
    pub selected_index: usize,
    pub is_loading: bool,
    pub error: Option<String>,
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

#[derive(Debug, Clone)]
pub struct UsersState {
    pub users: Vec<user::Model>,
    pub selected_index: usize,
    pub is_loading: bool,
    pub error: Option<String>,
}

impl Default for UsersState {
    fn default() -> Self {
        Self {
            users: Vec::new(),
            selected_index: 0,
            is_loading: false,
            error: None,
        }
    }
}

#[derive(Debug)]
pub enum TagError {
    LoadFailed(String),
}

#[derive(Debug)]
pub enum UserError {
    LoadFailed(String),
}

#[derive(Debug)]
pub enum SeedFlowError {
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct SeedSummaryState {
    pub is_loading: bool,
    pub outcome: Option<SeedOutcome>,
    pub error: Option<String>,
}

impl Default for SeedSummaryState {
    fn default() -> Self {
        Self {
            is_loading: false,
            outcome: None,
            error: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SeedHistoryState {
    pub is_loading: bool,
    pub runs: Vec<SeedOutcomeRow>,
    pub selected_index: usize,
    pub error: Option<String>,
}

impl Default for SeedHistoryState {
    fn default() -> Self {
        Self {
            is_loading: false,
            runs: Vec::new(),
            selected_index: 0,
            error: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SeedUndoState {
    pub is_loading: bool,
    pub selected_run: Option<SeedOutcomeRow>,
    pub outcome: Option<UndoOutcome>,
    pub error: Option<String>,
}

impl Default for SeedUndoState {
    fn default() -> Self {
        Self {
            is_loading: false,
            selected_run: None,
            outcome: None,
            error: None,
        }
    }
}

#[derive(Debug)]
pub enum AppEvent {
    TagsLoaded(Result<Vec<tag::Model>, TagError>),
    UsersLoaded(Result<Vec<user::Model>, UserError>),
    SeedCompleted(Result<SeedOutcome, SeedFlowError>),
    SeedHistoryLoaded(Result<Vec<SeedOutcomeRow>, SeedFlowError>),
    SeedUndoCompleted(Result<UndoOutcome, SeedFlowError>),
}

pub struct App {
    pub route: AppRoute,
    pub selected_home_index: usize,
    pub tags: TagsState,
    pub users: UsersState,
    pub seed_summary: SeedSummaryState,
    pub seed_history: SeedHistoryState,
    pub seed_undo: SeedUndoState,
    pub should_quit: bool,
    pub theme: ThemeKind,
    pub logs: Vec<String>,
    log_file: Option<std::fs::File>,
    core: Arc<CoreState>,
}

impl App {
    fn new(core: Arc<CoreState>, theme: ThemeKind) -> Self {
        let (log_file, init_log_msg) = Self::init_log_file();
        let mut app = Self {
            route: AppRoute::Home,
            selected_home_index: 0,
            tags: TagsState::default(),
            users: UsersState::default(),
            seed_summary: SeedSummaryState::default(),
            seed_history: SeedHistoryState::default(),
            seed_undo: SeedUndoState::default(),
            should_quit: false,
            theme,
            logs: Vec::new(),
            log_file,
            core,
        };

        if let Some(msg) = init_log_msg {
            app.push_log(msg);
        }

        app
    }

    fn init_log_file() -> (Option<std::fs::File>, Option<String>) {
        let path = std::env::var("TUI_LOG_PATH")
            .ok()
            .map(PathBuf::from)
            .or_else(|| std::env::current_dir().ok().map(|p| p.join("tui.log")));

        if let Some(path) = path {
            match OpenOptions::new().create(true).append(true).open(&path) {
                Ok(file) => (Some(file), Some(format!("logging to {}", path.display()))),
                Err(err) => (
                    None,
                    Some(format!("log file open failed at {}: {}", path.display(), err)),
                ),
            }
        } else {
            (None, None)
        }
    }

    fn push_log<S: Into<String>>(&mut self, line: S) {
        const MAX_LOG_LINES: usize = 200;
        let line = line.into();
        self.write_log_to_file(&line);
        self.logs.push(line);
        if self.logs.len() > MAX_LOG_LINES {
            let excess = self.logs.len() - MAX_LOG_LINES;
            self.logs.drain(0..excess);
        }
    }

    fn write_log_to_file(&mut self, line: &str) {
        if let Some(file) = &mut self.log_file {
            let ts = Utc::now().to_rfc3339();
            let _ = writeln!(file, "[{ts}] {line}");
        }
    }

    fn handle_key(&mut self, key: KeyEvent, tx: &mpsc::UnboundedSender<AppEvent>) {
        if let KeyCode::Char('t') | KeyCode::Char('T') = key.code {
            self.theme = self.theme.next();
            self.push_log(format!("theme: {}", self.theme.name()));
            return;
        }

        match self.route {
            AppRoute::Home => self.handle_key_home(key, tx),
            AppRoute::Tags => self.handle_key_tags(key, tx),
            AppRoute::Users => self.handle_key_users(key, tx),
            AppRoute::SeedMenu => self.handle_key_seed_menu(key, tx),
            AppRoute::SeedProgress => {}
            AppRoute::SeedSummary => self.handle_key_seed_summary(key),
            AppRoute::SeedHistory => self.handle_key_seed_history(key, tx),
            AppRoute::SeedUndoConfirm => self.handle_key_seed_undo(key, tx),
        }
    }

    fn handle_key_home(&mut self, key: KeyEvent, tx: &mpsc::UnboundedSender<AppEvent>) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.should_quit = true;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected_home_index > 0 {
                    self.selected_home_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let max_index = 3; // tags, users, seed, history
                if self.selected_home_index < max_index {
                    self.selected_home_index += 1;
                }
            }
            KeyCode::Enter => {
                match self.selected_home_index {
                    0 => self.open_tags(tx),
                    1 => self.open_users(tx),
                    2 => self.open_seed_menu(),
                    3 => self.open_seed_history(tx),
                    _ => self.open_tags(tx),
                }
            }
            _ => {}
        }
    }

    fn handle_key_tags(&mut self, key: KeyEvent, tx: &mpsc::UnboundedSender<AppEvent>) {
        if self.tags.error.is_some() {
            match key.code {
                KeyCode::Char('r') | KeyCode::Char('R') => {
                    self.load_tags(tx);
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    self.route = AppRoute::Home;
                }
                _ => {
                    self.tags.error = None;
                }
            }
            return;
        }

        if self.tags.is_loading {
            if key.code == KeyCode::Esc || key.code == KeyCode::Char('q') {
                self.route = AppRoute::Home;
            }
            return;
        }

        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.route = AppRoute::Home;
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

    fn handle_key_users(&mut self, key: KeyEvent, tx: &mpsc::UnboundedSender<AppEvent>) {
        if self.users.error.is_some() {
            match key.code {
                KeyCode::Char('r') | KeyCode::Char('R') => {
                    self.load_users(tx);
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    self.route = AppRoute::Home;
                }
                _ => {
                    self.users.error = None;
                }
            }
            return;
        }

        if self.users.is_loading {
            if key.code == KeyCode::Esc || key.code == KeyCode::Char('q') {
                self.route = AppRoute::Home;
            }
            return;
        }

        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.route = AppRoute::Home;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if !self.users.users.is_empty() && self.users.selected_index > 0 {
                    self.users.selected_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.users.users.is_empty()
                    && self.users.selected_index + 1 < self.users.users.len()
                {
                    self.users.selected_index += 1;
                }
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                self.load_users(tx);
            }
            _ => {}
        }
    }

    fn handle_key_seed_menu(&mut self, key: KeyEvent, tx: &mpsc::UnboundedSender<AppEvent>) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.route = AppRoute::Home;
            }
            KeyCode::Char('1') | KeyCode::Enter => {
                self.start_seed_all(tx);
            }
            _ => {}
        }
    }

    fn handle_key_seed_summary(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('h') => {
                self.route = AppRoute::Home;
            }
            KeyCode::Char('t') => {
                self.route = AppRoute::Tags;
            }
            KeyCode::Char('u') => {
                self.route = AppRoute::Users;
            }
            _ => {}
        }
    }

    fn handle_key_seed_history(&mut self, key: KeyEvent, tx: &mpsc::UnboundedSender<AppEvent>) {
        if self.seed_history.error.is_some() {
            match key.code {
                KeyCode::Char('r') | KeyCode::Char('R') => self.open_seed_history(tx),
                _ => {
                    self.seed_history.error = None;
                }
            }
            return;
        }

        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.route = AppRoute::Home;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.seed_history.selected_index > 0 {
                    self.seed_history.selected_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.seed_history.runs.is_empty()
                    && self.seed_history.selected_index + 1 < self.seed_history.runs.len()
                {
                    self.seed_history.selected_index += 1;
                }
            }
            KeyCode::Enter => {
                if let Some(run) = self
                    .seed_history
                    .runs
                    .get(self.seed_history.selected_index)
                    .cloned()
                {
                    self.seed_undo.selected_run = Some(run);
                    self.seed_undo.outcome = None;
                    self.seed_undo.error = None;
                    self.route = AppRoute::SeedUndoConfirm;
                }
            }
            _ => {}
        }
    }

    fn handle_key_seed_undo(&mut self, key: KeyEvent, tx: &mpsc::UnboundedSender<AppEvent>) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.route = AppRoute::SeedHistory;
            }
            KeyCode::Char('u') | KeyCode::Enter => {
                if let Some(run) = self.seed_undo.selected_run.clone() {
                    self.seed_undo.is_loading = true;
                    let core = self.core.clone();
                    let tx_clone = tx.clone();
                    tokio::spawn(async move {
                        let res = seed::undo_seed_run(&core.db, run.id)
                            .await
                            .map_err(|e| SeedFlowError::Failed(e.to_string()));
                        let _ = tx_clone.send(AppEvent::SeedUndoCompleted(res));
                    });
                    self.route = AppRoute::SeedProgress;
                }
            }
            _ => {}
        }
    }

    fn open_tags(&mut self, tx: &mpsc::UnboundedSender<AppEvent>) {
        self.route = AppRoute::Tags;
        self.tags = TagsState::default();
        self.load_tags(tx);
    }

    fn open_users(&mut self, tx: &mpsc::UnboundedSender<AppEvent>) {
        self.route = AppRoute::Users;
        self.users = UsersState::default();
        self.load_users(tx);
    }

    fn open_seed_menu(&mut self) {
        self.route = AppRoute::SeedMenu;
    }

    fn open_seed_history(&mut self, tx: &mpsc::UnboundedSender<AppEvent>) {
        if self.seed_history.is_loading {
            return;
        }
        self.seed_history.is_loading = true;
        self.seed_history.error = None;
        self.route = AppRoute::SeedHistory;

        let core = self.core.clone();
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            let res = seed::list_seed_runs(&core.db)
                .await
                .map_err(|e| SeedFlowError::Failed(e.to_string()));
            let _ = tx_clone.send(AppEvent::SeedHistoryLoaded(res));
        });
    }

    fn start_seed_all(&mut self, tx: &mpsc::UnboundedSender<AppEvent>) {
        if self.seed_summary.is_loading {
            return;
        }
        self.seed_summary.is_loading = true;
        self.seed_summary.error = None;
        self.seed_summary.outcome = None;
        self.route = AppRoute::SeedProgress;

        let core = self.core.clone();
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            let res = seed::seed_all(&core.db)
                .await
                .map_err(|e| SeedFlowError::Failed(e.to_string()));
            let _ = tx_clone.send(AppEvent::SeedCompleted(res));
        });
    }

    fn load_tags(&mut self, tx: &mpsc::UnboundedSender<AppEvent>) {
        if self.tags.is_loading {
            return;
        }

        self.tags.is_loading = true;
        self.tags.error = None;

        let core = self.core.clone();
        let tx_clone = tx.clone();

        tokio::spawn(async move {
            let result = fetch_tags(core).await;
            let _ = tx_clone.send(AppEvent::TagsLoaded(result));
        });
    }

    fn load_users(&mut self, tx: &mpsc::UnboundedSender<AppEvent>) {
        if self.users.is_loading {
            return;
        }

        self.users.is_loading = true;
        self.users.error = None;

        let core = self.core.clone();
        let tx_clone = tx.clone();

        tokio::spawn(async move {
            let result = fetch_users(core).await;
            let _ = tx_clone.send(AppEvent::UsersLoaded(result));
        });
    }

    fn handle_app_event(&mut self, event: AppEvent, _tx: &mpsc::UnboundedSender<AppEvent>) {
        match event {
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
                        self.push_log(format!("tags load error: {:?}", err));
                        self.tags.error = Some(format!("{:?}", err));
                    }
                }
            }
            AppEvent::UsersLoaded(result) => {
                self.users.is_loading = false;
                match result {
                    Ok(users) => {
                        self.push_log(format!("users loaded: {} items", users.len()));
                        self.users.users = users;
                        if !self.users.users.is_empty() {
                            self.users.selected_index = 0;
                        }
                    }
                    Err(err) => {
                        self.push_log(format!("users load error: {:?}", err));
                        self.users.error = Some(format!("{:?}", err));
                    }
                }
            }
            AppEvent::SeedCompleted(result) => {
                self.seed_summary.is_loading = false;
                match result {
                    Ok(outcome) => {
                        self.seed_summary.outcome = Some(outcome);
                        self.route = AppRoute::SeedSummary;
                        self.push_log("seed completed");
                    }
                    Err(err) => {
                        self.seed_summary.error = Some(format!("{:?}", err));
                        self.route = AppRoute::SeedSummary;
                    }
                }
            }
            AppEvent::SeedHistoryLoaded(result) => {
                self.seed_history.is_loading = false;
                match result {
                    Ok(runs) => {
                        self.seed_history.runs = runs;
                        if !self.seed_history.runs.is_empty() {
                            self.seed_history.selected_index = 0;
                        }
                    }
                    Err(err) => {
                        self.seed_history.error = Some(format!("{:?}", err));
                    }
                }
            }
            AppEvent::SeedUndoCompleted(result) => match result {
                Ok(outcome) => {
                    self.seed_undo.is_loading = false;
                    self.seed_undo.outcome = Some(outcome);
                    self.route = AppRoute::SeedSummary;
                }
                Err(err) => {
                    self.seed_undo.is_loading = false;
                    self.seed_undo.error = Some(format!("{:?}", err));
                    self.route = AppRoute::SeedHistory;
                }
            },
        }
    }
}

pub async fn run_tui(theme: ThemeKind) -> Result<(), Box<dyn Error>> {
    // Run migrations to ensure tables exist before loading tags; but return
    // a clean error instead of panicking if DB is unreachable.
    let db = init_db(true).await;
    let redis = init_redis_pool_only().await?;
    let core = Arc::new(CoreState { db, redis });

    let mut bridge =
        TerminalBridge::init_crossterm().map_err(|e| format!("terminal init error: {e}"))?;
    let res = run_app(bridge.raw_mut(), core, theme).await;
    bridge
        .restore()
        .map_err(|e| format!("terminal restore error: {e}"))?;
    res
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    core: Arc<CoreState>,
    theme: ThemeKind,
) -> Result<(), Box<dyn Error>> {
    let (tx, mut rx) = mpsc::unbounded_channel::<AppEvent>();
    let mut app = App::new(core, theme);

    loop {
        terminal.draw(|f| {
            let palette = theme_palette(app.theme);
            let root = f.area();
            let bg = ratatui::widgets::Block::default().style(Style::default().bg(palette.bg));
            f.render_widget(bg, root);

            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(6)].as_ref())
                .split(root);

            match app.route {
                AppRoute::Home => draw_home(f, layout[0], &app, &palette),
                AppRoute::Tags => draw_tags(f, layout[0], &app, &palette),
                AppRoute::Users => draw_users(f, layout[0], &app, &palette),
                AppRoute::SeedMenu => draw_seed_menu(f, layout[0], &palette),
                AppRoute::SeedProgress => draw_seed_progress(f, layout[0], &app, &palette),
                AppRoute::SeedSummary => draw_seed_summary(f, layout[0], &app, &palette),
                AppRoute::SeedHistory => draw_seed_history(f, layout[0], &app, &palette),
                AppRoute::SeedUndoConfirm => draw_seed_undo(f, layout[0], &app, &palette),
            }

            draw_logs(f, layout[1], &app.logs, &palette);
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

async fn fetch_tags(core: Arc<CoreState>) -> Result<Vec<tag::Model>, TagError> {
    tag::Entity::find_all(&core.db)
        .await
        .map_err(|e| TagError::LoadFailed(e.to_string()))
}

async fn fetch_users(core: Arc<CoreState>) -> Result<Vec<user::Model>, UserError> {
    user::Entity::find()
        .order_by_desc(user::Column::Id)
        .all(&core.db)
        .await
        .map_err(|e| UserError::LoadFailed(e.to_string()))
}
