use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::{
    app::{LoginField, LoginState},
    components::layout::centered_rect,
    theme::ThemePalette,
};

pub fn draw_login(f: &mut Frame, area: Rect, state: &LoginState, palette: &ThemePalette) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(7),
                Constraint::Length(3),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(area);

    let title = Paragraph::new(Line::from(vec![
        Span::styled("● ", Style::default().fg(palette.accent)),
        Span::styled(
            "ruxlog",
            Style::default()
                .fg(palette.header_fg)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " TUI · auth + tags",
            Style::default().fg(palette.text_muted),
        ),
    ]))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(palette.header_border))
            .style(Style::default().bg(palette.panel_bg)),
    );
    f.render_widget(title, chunks[0]);

    let form_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(palette.accent))
        .title(Span::styled(
            " Credentials ",
            Style::default()
                .fg(palette.accent)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(palette.panel_bg));
    let form_area = chunks[1];
    f.render_widget(form_block, form_area);

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(2),
                Constraint::Length(2),
                Constraint::Length(1),
            ]
            .as_ref(),
        )
        .split(form_area);

    let username_label = "Email:";
    let username_value = &state.username_input;
    let username_style = Style::default().fg(if matches!(state.focused_field, LoginField::Username)
    {
        palette.input_label_focus
    } else {
        palette.input_label
    });
    let username = Paragraph::new(Line::from(vec![
        Span::styled(username_label, username_style),
        Span::raw(" "),
        Span::raw(username_value),
    ]));
    f.render_widget(username, inner[0]);

    let password_label = "Password:";
    let masked = "•".repeat(state.password_input.chars().count());
    let password_style = Style::default().fg(if matches!(state.focused_field, LoginField::Password)
    {
        palette.input_label_focus
    } else {
        palette.input_label
    });
    let password = Paragraph::new(Line::from(vec![
        Span::styled(password_label, password_style),
        Span::raw(" "),
        Span::raw(masked),
    ]));
    f.render_widget(password, inner[1]);

    let submit_text = if state.is_loading {
        "Logging in..."
    } else {
        "Press Enter to login"
    };
    let submit_style = if matches!(state.focused_field, LoginField::Submit) {
        Style::default()
            .fg(palette.highlight_fg)
            .bg(palette.highlight_bg)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(palette.submit_fg)
            .bg(palette.table_row_even_bg)
    };
    let submit = Paragraph::new(Span::styled(submit_text, submit_style))
        .alignment(Alignment::Center);
    f.render_widget(submit, inner[2]);

    if let Some(err) = &state.error {
        let area = centered_rect(60, 25, area);
        let block = Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(
                "Login Error",
                Style::default()
                    .fg(palette.error_fg)
                    .add_modifier(Modifier::BOLD),
            ))
            .style(Style::default().bg(palette.panel_bg));
        let lines = vec![
            Line::from(err.as_str()),
            Line::from(""),
            Line::from("Press any key to continue"),
        ];
        let error = Paragraph::new(lines)
            .block(block)
            .alignment(Alignment::Center);
        f.render_widget(error, area);
    }

    let footer = Paragraph::new("Tab ⇆  •  Enter ↵  •  Esc to quit")
        .style(Style::default().fg(palette.footer_fg).bg(palette.panel_bg))
        .alignment(Alignment::Center);
    f.render_widget(footer, chunks[3]);
}

