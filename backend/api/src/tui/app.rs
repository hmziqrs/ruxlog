use std::{error::Error, sync::Arc, time::Duration};

use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::Style,
    Terminal,
};
use sea_orm::DatabaseConnection;
use tokio::{sync::mpsc, time::sleep};
use tower_sessions_redis_store::fred::prelude::Pool as RedisPool;
use tuirealm::terminal::TerminalBridge;

use crate::{
    db::{
        sea_connect::init_db,
        sea_models::tag,
    },
    services::redis::init_redis_pool_only,
};

use super::{
    components::logs::draw_logs,
    screens::tags::draw_tags,
    theme::{theme_palette, ThemeKind},
};

#[derive(Clone)]
pub struct CoreState {
    pub db: DatabaseConnection,
    pub redis: RedisPool,
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

#[derive(Debug)]
pub enum TagError {
    LoadFailed(String),
}

#[derive(Debug)]
pub enum AppEvent {
    TagsLoaded(Result<Vec<tag::Model>, TagError>),
}

pub struct App {
    pub tags: TagsState,
    pub should_quit: bool,
    pub theme: ThemeKind,
    pub logs: Vec<String>,
    core: Arc<CoreState>,
}

impl App {
    fn new(core: Arc<CoreState>, theme: ThemeKind) -> Self {
        Self {
            tags: TagsState::default(),
            should_quit: false,
            theme,
            logs: Vec::new(),
            core,
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
        if let KeyCode::Char('t') | KeyCode::Char('T') = key.code {
            self.theme = self.theme.next();
            self.push_log(format!("theme: {}", self.theme.name()));
            return;
        }

        self.handle_key_tags(key, tx);
    }

    fn handle_key_tags(&mut self, key: KeyEvent, tx: &mpsc::UnboundedSender<AppEvent>) {
        if self.tags.error.is_some() {
            match key.code {
                KeyCode::Char('r') | KeyCode::Char('R') => {
                    self.load_tags(tx);
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    self.should_quit = true;
                }
                _ => {
                    self.tags.error = None;
                }
            }
            return;
        }

        if self.tags.is_loading {
            if key.code == KeyCode::Esc || key.code == KeyCode::Char('q') {
                self.should_quit = true;
            }
            return;
        }

        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.should_quit = true;
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
        }
    }
}

pub async fn run_tui(theme: ThemeKind) -> Result<(), Box<dyn Error>> {
    let db = init_db(false).await;
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
    app.load_tags(&tx);

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

            draw_tags(f, layout[0], &app, &palette);

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
