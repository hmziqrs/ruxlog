use ratatui::{
    layout::Rect,
    style::Style,
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::theme::ThemePalette;

pub fn draw_logs(f: &mut Frame, area: Rect, logs: &[String], palette: &ThemePalette) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(palette.header_border))
        .style(Style::default().bg(palette.panel_bg))
        .title(" logs ");

    let lines: Vec<Line> = logs
        .iter()
        .rev()
        .take(3)
        .rev()
        .map(|l| Line::from(format!("  {l}")))
        .collect();

    let paragraph = Paragraph::new(lines)
        .style(Style::default().fg(palette.text_muted))
        .block(block);

    f.render_widget(paragraph, area);
}

