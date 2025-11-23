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

pub fn draw_home(f: &mut Frame, area: Rect, app: &App, palette: &ThemePalette) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(2),
            ]
            .as_ref(),
        )
        .split(area);

    let header = Paragraph::new("Ruxlog TUI · Home")
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
                    " Home ",
                    Style::default()
                        .fg(palette.header_border)
                        .add_modifier(Modifier::BOLD),
                ))
                .style(Style::default().bg(palette.panel_bg)),
        );
    f.render_widget(header, chunks[0]);

    let menu_labels = vec![
        "View tags",
        "View users",
        "Seed data",
        "Seed history / undo",
    ];
    let mut menu_items: Vec<ListItem> = Vec::new();
    for label in menu_labels {
        menu_items.push(ListItem::new(Line::from(vec![Span::styled(
            label,
            Style::default()
                .fg(palette.table_header_fg)
                .add_modifier(Modifier::BOLD),
        )])));
    }

    let list = List::new(menu_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(palette.table_header_bg))
                .title(Span::styled(
                    " Menu ",
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
    state.select(Some(app.selected_home_index));
    f.render_stateful_widget(list, chunks[1], &mut state);

    let footer_text = "[Enter] open  [↑/↓ or j/k] navigate  [Q/Esc] quit";
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(palette.footer_fg))
        .alignment(Alignment::Center);
    f.render_widget(footer, chunks[2]);

    if app.tags.error.is_some() && app.route == crate::tui::app::AppRoute::Home {
        if let Some(err) = &app.tags.error {
            let block = Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(
                    "Error",
                    Style::default()
                        .fg(palette.error_fg)
                        .add_modifier(Modifier::BOLD),
                ))
                .style(Style::default().bg(palette.panel_bg));
            let lines = vec![
                Line::from(err.as_str()),
                Line::from(""),
                Line::from("Press any key to dismiss"),
            ];
            let error = Paragraph::new(lines)
                .block(block)
                .alignment(Alignment::Left);
            f.render_widget(error, chunks[1]);
        }
    }
}
