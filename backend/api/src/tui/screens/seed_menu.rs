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

    let menu_items = vec![
        ListItem::new(Line::from(vec![Span::styled(
            "1) Seed with RANDOM data (unique each time)",
            Style::default()
                .fg(palette.table_header_fg)
                .add_modifier(Modifier::BOLD),
        )])),
        ListItem::new(Line::from(vec![Span::styled(
            "2) Seed with STATIC data (enter custom seed)",
            Style::default()
                .fg(palette.table_header_fg)
                .add_modifier(Modifier::BOLD),
        )])),
        ListItem::new(Line::from(vec![Span::styled(
            "3) Seed with PRESET (demo/test/showcase)",
            Style::default()
                .fg(palette.table_header_fg)
                .add_modifier(Modifier::BOLD),
        )])),
        ListItem::new(Line::from(vec![Span::styled(
            "4) List available presets",
            Style::default()
                .fg(palette.table_header_fg)
                .add_modifier(Modifier::BOLD),
        )])),
    ];

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

    let footer_text = "[1-4 or Enter] select  [Q/Esc] back  [↑↓] navigate";
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(palette.footer_fg))
        .alignment(Alignment::Center);
    f.render_widget(footer, chunks[2]);
}
