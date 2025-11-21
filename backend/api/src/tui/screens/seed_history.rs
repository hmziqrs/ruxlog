use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::tui::{
    app::App,
    theme::ThemePalette,
};

pub fn draw_seed_history(f: &mut Frame, area: Rect, app: &App, palette: &ThemePalette) {
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

    let header = Paragraph::new("Seed history")
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
                    " Seed runs ",
                    Style::default()
                        .fg(palette.header_border)
                        .add_modifier(Modifier::BOLD),
                ))
                .style(Style::default().bg(palette.panel_bg)),
        );
    f.render_widget(header, chunks[0]);

    if let Some(err) = &app.seed_history.error {
        let body = Paragraph::new(Line::from(err.as_str()))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(palette.error_fg))
                    .title("Error")
                    .style(Style::default().bg(palette.panel_bg)),
            );
        f.render_widget(body, chunks[1]);
    } else if app.seed_history.is_loading {
        let body = Paragraph::new("Loading seed runs...")
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().bg(palette.panel_bg)),
            );
        f.render_widget(body, chunks[1]);
    } else {
        let mut items: Vec<ListItem> = Vec::new();
        for run in &app.seed_history.runs {
            let total: i32 = run.counts.values().sum();
            items.push(ListItem::new(Line::from(vec![
                Span::styled(
                    format!("#{} {}", run.id, run.key),
                    Style::default()
                        .fg(palette.table_header_fg)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(
                    format!("{} items", total),
                    Style::default().fg(palette.text_muted),
                ),
                Span::raw(" "),
                Span::styled(
                    run.ran_at.to_rfc3339(),
                    Style::default().fg(palette.text_muted),
                ),
            ])));
        }

        if items.is_empty() {
            items.push(ListItem::new("No seed runs yet"));
        }

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(palette.table_header_bg))
                    .title(Span::styled(
                        " Runs ",
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

        let mut state = ratatui::widgets::ListState::default();
        if !app.seed_history.runs.is_empty() {
            state.select(Some(app.seed_history.selected_index));
        }
        f.render_stateful_widget(list, chunks[1], &mut state);
    }

    let footer_text = "[Enter] undo selected  [R] reload  [Q/Esc] back";
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(palette.footer_fg))
        .alignment(Alignment::Center);
    f.render_widget(footer, chunks[2]);
}

