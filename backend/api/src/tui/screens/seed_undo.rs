use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::tui::{app::App, theme::ThemePalette};

pub fn draw_seed_undo(f: &mut Frame, area: Rect, app: &App, palette: &ThemePalette) {
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

    let header = Paragraph::new("Undo seed run")
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
                    " Undo ",
                    Style::default()
                        .fg(palette.header_border)
                        .add_modifier(Modifier::BOLD),
                ))
                .style(Style::default().bg(palette.panel_bg)),
        );
    f.render_widget(header, chunks[0]);

    if let Some(err) = &app.seed_undo.error {
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
    } else if let Some(run) = &app.seed_undo.selected_run {
        let mut items: Vec<ListItem> = Vec::new();
        let mut counts = run.counts.clone().into_iter().collect::<Vec<_>>();
        counts.sort_by(|a, b| a.0.cmp(&b.0));
        for (name, count) in counts {
            items.push(ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{:<25}", name),
                    Style::default()
                        .fg(palette.table_header_fg)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(count.to_string(), Style::default().fg(palette.text_muted)),
            ])));
        }

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(palette.table_header_bg))
                    .title(Span::styled(
                        format!(" Undo run #{} ", run.id),
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

        f.render_widget(list, chunks[1]);
    } else {
        let body = Paragraph::new("No run selected")
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(palette.table_header_bg))
                    .style(Style::default().bg(palette.panel_bg)),
            );
        f.render_widget(body, chunks[1]);
    }

    let footer_text = "[U/Enter] confirm undo  [Q/Esc] back";
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(palette.footer_fg))
        .alignment(Alignment::Center);
    f.render_widget(footer, chunks[2]);
}

