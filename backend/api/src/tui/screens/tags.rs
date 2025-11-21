use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::tui::{
    app::App,
    components::layout::centered_rect,
    theme::ThemePalette,
};

pub fn draw_tags(f: &mut Frame, area: Rect, app: &App, palette: &ThemePalette) {
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

    let header_text = "Tags".to_string();
    let header = Paragraph::new(header_text)
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
                    " Tags ",
                    Style::default()
                        .fg(palette.header_border)
                        .add_modifier(Modifier::BOLD),
                ))
                .style(Style::default().bg(palette.panel_bg)),
        );
    f.render_widget(header, chunks[0]);

    if app.tags.is_loading && app.tags.tags.is_empty() {
        let loading = Paragraph::new("Loading tags...")
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Tags")
                    .style(Style::default().bg(palette.panel_bg)),
            );
        f.render_widget(loading, chunks[1]);
    } else if let Some(err) = &app.tags.error {
        let area = centered_rect(60, 25, area);
        let block = Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(
                "Failed to load tags",
                Style::default()
                    .fg(palette.error_fg)
                    .add_modifier(Modifier::BOLD),
            ))
            .style(Style::default().bg(palette.panel_bg));
        let lines = vec![
            Line::from(err.as_str()),
            Line::from(""),
            Line::from("Press R to retry, Q/Esc to quit, any other key to dismiss"),
        ];
        let error = Paragraph::new(lines)
            .block(block)
            .alignment(Alignment::Center);
        f.render_widget(error, area);
    } else {
        let mut items: Vec<ListItem> = Vec::new();

        items.push(
            ListItem::new(Line::from(vec![
                Span::styled("#", Style::default().fg(palette.text_muted)),
                Span::raw("  "),
                Span::styled("Name", Style::default().fg(palette.table_header_fg)),
                Span::raw("  "),
                Span::styled("Slug", Style::default().fg(palette.text_muted)),
            ]))
            .style(Style::default().bg(palette.table_header_bg)),
        );

        for (idx, t) in app.tags.tags.iter().enumerate() {
            let line = Line::from(vec![
                Span::styled(
                    format!("{:>2}", idx + 1),
                    Style::default().fg(palette.text_muted),
                ),
                Span::raw("  "),
                Span::styled(
                    t.name.clone(),
                    Style::default()
                        .fg(palette.table_header_fg)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled(
                    t.slug.clone(),
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
                        " Tag list ",
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
        if !app.tags.tags.is_empty() {
            state.select(Some(app.tags.selected_index));
        }

        f.render_stateful_widget(list, chunks[1], &mut state);
    }

    let footer_text = "[↑/↓ or j/k] navigate  [R] reload  [Q/Esc] quit";
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(palette.footer_fg))
        .alignment(Alignment::Center);
    f.render_widget(footer, chunks[2]);
}
