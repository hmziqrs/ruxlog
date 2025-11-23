use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::tui::{app::App, theme::ThemePalette};

pub fn draw_seed_progress(f: &mut Frame, area: Rect, app: &App, palette: &ThemePalette) {
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

    let header = Paragraph::new("Seeding in progress...")
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
                    " Seed ",
                    Style::default()
                        .fg(palette.header_border)
                        .add_modifier(Modifier::BOLD),
                ))
                .style(Style::default().bg(palette.panel_bg)),
        );
    f.render_widget(header, chunks[0]);

    // Show recent logs from the seeding process
    let log_count = (chunks[1].height as usize).saturating_sub(2).min(20);
    let recent_logs: Vec<String> = app
        .logs
        .iter()
        .rev()
        .take(log_count)
        .rev()
        .cloned()
        .collect();

    if !recent_logs.is_empty() {
        let log_items: Vec<ListItem> = recent_logs
            .iter()
            .map(|log| {
                let content = if log.starts_with("Failed")
                    || log.contains("error")
                    || log.contains("Error")
                {
                    Line::from(vec![
                        Span::styled("✗ ", Style::default().fg(palette.error_fg)),
                        Span::raw(log.as_str()),
                    ])
                } else if log.contains("Creating") || log.contains("Seeding") {
                    Line::from(vec![
                        Span::styled("▸ ", Style::default().fg(palette.table_header_fg)),
                        Span::raw(log.as_str()),
                    ])
                } else if log.contains("Created") || log.contains("completed") {
                    Line::from(vec![
                        Span::styled("✓ ", Style::default().fg(palette.highlight_fg)),
                        Span::raw(log.as_str()),
                    ])
                } else {
                    Line::from(log.as_str())
                };
                ListItem::new(content)
            })
            .collect();

        let log_list = List::new(log_items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(palette.table_header_bg))
                .title(Span::styled(
                    " Progress ",
                    Style::default()
                        .fg(palette.table_header_fg)
                        .add_modifier(Modifier::BOLD),
                ))
                .style(Style::default().bg(palette.panel_bg)),
        );
        f.render_widget(log_list, chunks[1]);
    } else {
        let body = Paragraph::new("Initializing...")
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(palette.table_header_bg))
                    .style(Style::default().bg(palette.panel_bg)),
            );
        f.render_widget(body, chunks[1]);
    }

    let footer_text = if app.seed_summary.is_loading {
        "[Seeding…]  [Q/Esc] cancel to home"
    } else {
        "[Done] press any key"
    };
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(palette.footer_fg))
        .alignment(Alignment::Center);
    f.render_widget(footer, chunks[2]);
}
