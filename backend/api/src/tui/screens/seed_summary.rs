use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::tui::{app::App, theme::ThemePalette};

pub fn draw_seed_summary(f: &mut Frame, area: Rect, app: &App, palette: &ThemePalette) {
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

    let header = Paragraph::new("Seed summary")
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

    if let Some(err) = &app.seed_summary.error {
        // Truncate error message to prevent layout breaks
        let max_width = chunks[1].width.saturating_sub(4) as usize;
        let truncated_err = if err.len() > max_width {
            format!("{}...", &err[..max_width.saturating_sub(3)])
        } else {
            err.clone()
        };

        let body = Paragraph::new(Line::from(truncated_err.as_str()))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(palette.error_fg))
                    .title("Error")
                    .style(Style::default().bg(palette.panel_bg)),
            );
        f.render_widget(body, chunks[1]);
    } else if let Some(outcome) = &app.seed_summary.outcome {
        // Split the body area to show errors/warnings if present
        let has_issues = !outcome.errors.is_empty() || !outcome.warnings.is_empty();
        let body_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(if has_issues {
                [Constraint::Percentage(60), Constraint::Percentage(40)].as_ref()
            } else {
                [Constraint::Percentage(100)].as_ref()
            })
            .split(chunks[1]);

        // Show inserted counts
        let mut items: Vec<ListItem> = Vec::new();
        let mut counts = outcome.counts().into_iter().collect::<Vec<_>>();
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

        let status_text = if !outcome.errors.is_empty() {
            format!(" Inserted rows ({} errors) ", outcome.errors.len())
        } else if !outcome.warnings.is_empty() {
            format!(" Inserted rows ({} warnings) ", outcome.warnings.len())
        } else {
            " Inserted rows ".to_string()
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(palette.table_header_bg))
                    .title(Span::styled(
                        status_text,
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
        f.render_widget(list, body_chunks[0]);

        // Show errors and warnings if present
        if has_issues {
            let mut issue_items: Vec<ListItem> = Vec::new();
            let max_width = body_chunks[1].width.saturating_sub(4) as usize;

            for err in &outcome.errors {
                let truncated = if err.len() > max_width {
                    format!("{}...", &err[..max_width.saturating_sub(3)])
                } else {
                    err.clone()
                };
                issue_items.push(ListItem::new(Line::from(vec![
                    Span::styled("✗ ", Style::default().fg(palette.error_fg)),
                    Span::raw(truncated),
                ])));
            }

            for warn in &outcome.warnings {
                let truncated = if warn.len() > max_width {
                    format!("{}...", &warn[..max_width.saturating_sub(3)])
                } else {
                    warn.clone()
                };
                issue_items.push(ListItem::new(Line::from(vec![
                    Span::styled("⚠ ", Style::default().fg(palette.text_muted)),
                    Span::raw(truncated),
                ])));
            }

            let issues_list = List::new(issue_items).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(if !outcome.errors.is_empty() {
                        palette.error_fg
                    } else {
                        palette.text_muted
                    }))
                    .title(Span::styled(
                        if !outcome.errors.is_empty() {
                            " Errors "
                        } else {
                            " Warnings "
                        },
                        Style::default()
                            .fg(if !outcome.errors.is_empty() {
                                palette.error_fg
                            } else {
                                palette.text_muted
                            })
                            .add_modifier(Modifier::BOLD),
                    ))
                    .style(Style::default().bg(palette.panel_bg)),
            );
            f.render_widget(issues_list, body_chunks[1]);
        }
    } else {
        let body = Paragraph::new("No summary yet")
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(palette.table_header_bg))
                    .style(Style::default().bg(palette.panel_bg)),
            );
        f.render_widget(body, chunks[1]);
    }

    let footer_text = "[T] tags  [U] users  [H/Q/Esc] home";
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(palette.footer_fg))
        .alignment(Alignment::Center);
    f.render_widget(footer, chunks[2]);
}
