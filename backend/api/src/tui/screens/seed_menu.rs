use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::tui::theme::ThemePalette;

pub fn draw_seed_menu(f: &mut Frame, area: Rect, palette: &ThemePalette) {
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

    let menu_items = vec![ListItem::new(Line::from(vec![
        Span::styled(
            "1) Seed everything (local DB)",
            Style::default()
                .fg(palette.table_header_fg)
                .add_modifier(Modifier::BOLD),
        ),
    ]))];

    let list = List::new(menu_items)
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

    f.render_widget(list, chunks[1]);

    let footer_text = "[Enter or 1] seed all  [Q/Esc] back";
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(palette.footer_fg))
        .alignment(Alignment::Center);
    f.render_widget(footer, chunks[2]);
}

