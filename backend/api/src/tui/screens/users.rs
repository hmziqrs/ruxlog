use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::tui::{app::App, theme::ThemePalette};

pub fn draw_users(f: &mut Frame, area: Rect, app: &App, palette: &ThemePalette) {
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

    let header = Paragraph::new("Users")
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
                    " Users ",
                    Style::default()
                        .fg(palette.header_border)
                        .add_modifier(Modifier::BOLD),
                ))
                .style(Style::default().bg(palette.panel_bg)),
        );
    f.render_widget(header, chunks[0]);

    if app.users.is_loading && app.users.users.is_empty() {
        let loading = Paragraph::new("Loading users...")
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Users")
                    .style(Style::default().bg(palette.panel_bg)),
            );
        f.render_widget(loading, chunks[1]);
    } else if let Some(err) = &app.users.error {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(
                "Failed to load users",
                Style::default()
                    .fg(palette.error_fg)
                    .add_modifier(Modifier::BOLD),
            ))
            .style(Style::default().bg(palette.panel_bg));
        let lines = vec![
            Line::from(err.as_str()),
            Line::from(""),
            Line::from("Press R to retry, Q/Esc to go back, any other key to dismiss"),
        ];
        let error = Paragraph::new(lines)
            .block(block)
            .alignment(Alignment::Left);
        f.render_widget(error, chunks[1]);
    } else {
        let mut items: Vec<ListItem> = Vec::new();

        items.push(
            ListItem::new(Line::from(vec![
                Span::styled("ID", Style::default().fg(palette.text_muted)),
                Span::raw("  "),
                Span::styled("Email", Style::default().fg(palette.table_header_fg)),
                Span::raw("  "),
                Span::styled("Role", Style::default().fg(palette.text_muted)),
            ]))
            .style(Style::default().bg(palette.table_header_bg)),
        );

        for (idx, u) in app.users.users.iter().enumerate() {
            let line = Line::from(vec![
                Span::styled(
                    format!("{:>4}", u.id),
                    Style::default().fg(palette.text_muted),
                ),
                Span::raw("  "),
                Span::styled(
                    u.email.clone(),
                    Style::default()
                        .fg(palette.table_header_fg)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled(
                    format!("{:?}", u.role),
                    Style::default().fg(palette.table_slug_fg),
                ),
            ]);
            let row_style = if idx % 2 == 0 {
                Style::default().bg(palette.table_row_even_bg)
            } else {
                Style::default().bg(palette.table_row_odd_bg)
            };
            items.push(ListItem::new(line).style(row_style));
        }

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(palette.table_header_bg))
                    .title(Span::styled(
                        " User list ",
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
        if !app.users.users.is_empty() {
            state.select(Some(app.users.selected_index));
        }

        f.render_stateful_widget(list, chunks[1], &mut state);
    }

    let footer_text = "[↑/↓ or j/k] navigate  [R] reload  [Q/Esc] back";
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(palette.footer_fg))
        .alignment(Alignment::Center);
    f.render_widget(footer, chunks[2]);
}
