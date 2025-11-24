use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::tui::{app::App, theme::ThemePalette};

pub fn draw_seed_menu(f: &mut Frame, area: Rect, app: &App, palette: &ThemePalette) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(2),
            ]
            .as_ref(),
        )
        .split(area);

    let header = Paragraph::new("Seed menu")
        .style(
            Style::default()
                .fg(palette.header_fg)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(palette.header_border))
                .title(Span::styled(
                    " Seeding ",
                    Style::default()
                        .fg(palette.header_border)
                        .add_modifier(Modifier::BOLD),
                ))
                .style(Style::default().bg(palette.panel_bg)),
        );
    f.render_widget(header, chunks[0]);

    let size_label = app.custom_seed_size.label();
    let menu_labels = vec![
        "1) Seed ALL with RANDOM data (unique each time)".to_string(),
        "2) Seed ALL with STATIC data (enter custom seed)".to_string(),
        "3) Seed ALL with PRESET (demo/test/showcase)".to_string(),
        "4) List available presets".to_string(),
        format!("5) Seed MORE USERS (preset: {})", size_label),
        format!("6) Seed MORE CATEGORIES (preset: {})", size_label),
        format!("7) Seed MORE TAGS (preset: {})", size_label),
        format!("8) Seed MORE POSTS (preset: {})", size_label),
        format!("9) Seed MORE POST COMMENTS (preset: {})", size_label),
        format!("10) Seed MORE COMMENT FLAGS (preset: {})", size_label),
        format!("11) Seed MORE POST VIEWS (preset: {})", size_label),
        format!("12) Seed MORE USER SESSIONS (preset: {})", size_label),
        format!("13) Seed MORE EMAIL VERIFICATIONS (preset: {})", size_label),
        format!("14) Seed MORE FORGOT PASSWORDS (preset: {})", size_label),
        format!("15) Seed MORE POST REVISIONS (preset: {})", size_label),
        format!("16) Seed MORE POST SERIES (preset: {})", size_label),
        format!("17) Seed MORE SCHEDULED POSTS (preset: {})", size_label),
        format!("18) Seed MORE MEDIA (preset: {})", size_label),
        format!("19) Seed MORE MEDIA VARIANTS (preset: {})", size_label),
        format!("20) Seed MORE MEDIA USAGE (preset: {})", size_label),
        format!("21) Seed MORE NEWSLETTER SUBSCRIBERS (preset: {})", size_label),
        format!("22) Seed MORE ROUTE STATUS (preset: {})", size_label),
    ];

    let menu_items: Vec<ListItem> = menu_labels
        .iter()
        .map(|label| {
            ListItem::new(Line::from(vec![Span::styled(
                label,
                Style::default()
                    .fg(palette.table_header_fg)
                    .add_modifier(Modifier::BOLD),
            )]))
        })
        .collect();

    let list = List::new(menu_items)
        .highlight_symbol("▸ ")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(palette.table_header_bg))
                .title(Span::styled(
                    " Actions ",
                    Style::default()
                        .fg(palette.table_header_fg)
                        .add_modifier(Modifier::BOLD),
                ))
                .style(Style::default().bg(palette.panel_bg)),
        )
        .highlight_style(
            Style::default()
                .bg(palette.highlight_bg)
                .fg(palette.highlight_fg)
                .add_modifier(Modifier::BOLD),
        );

    f.render_stateful_widget(list, chunks[1], &mut app.seed_menu_state.clone());

    let footer_text =
        "[1-22/Enter] select  [c] custom size  [s] custom seed  [p] cycle seed mode  [[]/]] size preset  [Q/Esc] back";
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(palette.footer_fg))
        .alignment(Alignment::Center);
    f.render_widget(footer, chunks[2]);
}
