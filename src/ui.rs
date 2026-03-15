use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, Screen, FORM_FIELDS};
use crate::ssh::SshSession;

pub fn draw(f: &mut Frame, app: &App) {
    // SSH session takes over the entire frame — no chrome
    if let Screen::SshSession(session) = &app.screen {
        draw_ssh_session(f, session);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // header
            Constraint::Min(5),    // main
            Constraint::Length(3), // status
        ])
        .split(f.area());

    draw_header(f, chunks[0]);
    draw_server_list(f, chunks[1], app);
    draw_status_bar(f, chunks[2], app);

    // Overlays
    match &app.screen {
        Screen::AddForm => draw_form(f, app, "Add Server"),
        Screen::EditForm(_) => draw_form(f, app, "Edit Server"),
        Screen::ConfirmDelete(idx) => draw_confirm_delete(f, app, *idx),
        Screen::Dashboard | Screen::SshSession(_) => {}
    }
}

/// Render the vt100 terminal grid, cell by cell, grouping spans by style.
fn draw_ssh_session(f: &mut Frame, session: &SshSession) {
    let area = f.area();
    let parser = session.parser.lock().unwrap();
    let screen = parser.screen();

    let (rows, cols) = screen.size();
    let rows = rows as usize;
    let cols = cols as usize;

    let mut lines: Vec<Line> = Vec::with_capacity(rows);

    for row in 0..rows {
        let mut spans: Vec<Span> = Vec::new();
        let mut run = String::new();
        let mut run_style = Style::default();

        for col in 0..cols {
            let (content, style) = match screen.cell(row as u16, col as u16) {
                Some(cell) => {
                    // Skip the right half of wide chars — the parent cell covers them
                    if cell.is_wide_continuation() {
                        continue;
                    }
                    let s = cell_style(cell);
                    let c = if cell.has_contents() {
                        cell.contents()
                    } else {
                        " ".to_string()
                    };
                    (c, s)
                }
                None => (" ".to_string(), Style::default()),
            };

            if style == run_style {
                run.push_str(&content);
            } else {
                if !run.is_empty() {
                    spans.push(Span::styled(run.clone(), run_style));
                    run.clear();
                }
                run = content;
                run_style = style;
            }
        }
        if !run.is_empty() {
            spans.push(Span::styled(run, run_style));
        }
        lines.push(Line::from(spans));
    }

    f.render_widget(Paragraph::new(lines), area);

    // Position cursor
    if !screen.hide_cursor() {
        let (crow, ccol) = screen.cursor_position();
        let x = area.x + ccol;
        let y = area.y + crow;
        if x < area.x + area.width && y < area.y + area.height {
            f.set_cursor_position((x, y));
        }
    }
}

fn cell_style(cell: &vt100::Cell) -> Style {
    let mut style = Style::default()
        .fg(vt_color(cell.fgcolor()))
        .bg(vt_color(cell.bgcolor()));
    if cell.bold() {
        style = style.add_modifier(Modifier::BOLD);
    }
    if cell.italic() {
        style = style.add_modifier(Modifier::ITALIC);
    }
    if cell.underline() {
        style = style.add_modifier(Modifier::UNDERLINED);
    }
    style
}

fn vt_color(c: vt100::Color) -> Color {
    match c {
        vt100::Color::Default => Color::Reset,
        vt100::Color::Idx(i) => Color::Indexed(i),
        vt100::Color::Rgb(r, g, b) => Color::Rgb(r, g, b),
    }
}


fn draw_header(f: &mut Frame, area: Rect) {
    let text = Line::from(vec![
        Span::styled(
            " SSH Manager ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " │ a:Add  e:Edit  d:Delete  Enter:Connect  q:Quit ",
            Style::default().fg(Color::DarkGray),
        ),
    ]);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .style(Style::default().fg(Color::DarkGray));
    let para = Paragraph::new(text).block(block);
    f.render_widget(para, area);
}

fn draw_server_list(f: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = app
        .servers
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let tag_str = if s.tags.is_empty() {
                String::new()
            } else {
                format!("  [{}]", s.tags)
            };
            let line = Line::from(vec![
                Span::styled(
                    &s.alias,
                    Style::default().add_modifier(Modifier::BOLD).fg(
                        if i == app.selected {
                            Color::Black
                        } else {
                            Color::White
                        },
                    ),
                ),
                Span::styled(
                    format!("  {}", s.display_connection()),
                    Style::default().fg(if i == app.selected {
                        Color::Black
                    } else {
                        Color::DarkGray
                    }),
                ),
                Span::styled(
                    tag_str,
                    Style::default().fg(if i == app.selected {
                        Color::Black
                    } else {
                        Color::Green
                    }),
                ),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(" Servers ")
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Rounded),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Cyan)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▸ ");

    let mut state = ListState::default();
    if !app.servers.is_empty() {
        state.select(Some(app.selected));
    }
    f.render_stateful_widget(list, area, &mut state);
}

fn draw_status_bar(f: &mut Frame, area: Rect, app: &App) {
    let total = app.servers.len();
    let idx_info = if total > 0 {
        format!(" [{}/{}] ", app.selected + 1, total)
    } else {
        " [empty] ".to_string()
    };
    let line = Line::from(vec![
        Span::styled(idx_info, Style::default().fg(Color::Yellow)),
        Span::styled(&app.status_msg, Style::default().fg(Color::Green)),
    ]);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .style(Style::default().fg(Color::DarkGray));
    let para = Paragraph::new(line).block(block);
    f.render_widget(para, area);
}

fn draw_form(f: &mut Frame, app: &App, title: &str) {
    let area = centered_rect(60, 60, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Each field gets 3 rows: label (1) + input (2)
    let constraints: Vec<Constraint> = FORM_FIELDS
        .iter()
        .flat_map(|_| [Constraint::Length(1), Constraint::Length(3)])
        .chain(std::iter::once(Constraint::Length(2))) // help line
        .chain(std::iter::once(Constraint::Min(0)))
        .collect();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(constraints)
        .split(inner);

    for (i, field) in FORM_FIELDS.iter().enumerate() {
        let label_area = chunks[i * 2];
        let input_area = chunks[i * 2 + 1];
        let is_focused = app.form.focused == *field;

        // Label
        let label = Paragraph::new(Span::styled(
            field.label(),
            Style::default()
                .fg(if is_focused { Color::Cyan } else { Color::White })
                .add_modifier(if is_focused {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                }),
        ));
        f.render_widget(label, label_area);

        // Input box
        let value = app.form.get_field(*field);
        let input_block = Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .style(if is_focused {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::DarkGray)
            });

        let input = Paragraph::new(value).block(input_block);
        f.render_widget(input, input_area);

        // Show cursor in focused field
        if is_focused {
            let cursor_x = input_area.x + 1 + value.len() as u16;
            let cursor_y = input_area.y + 1;
            f.set_cursor_position((cursor_x, cursor_y));
        }
    }

    // Help text
    let help_idx = FORM_FIELDS.len() * 2;
    if help_idx < chunks.len() {
        let help = Paragraph::new(Span::styled(
            " Tab: next field  Shift+Tab: prev  Enter: save  Esc: cancel",
            Style::default().fg(Color::DarkGray),
        ));
        f.render_widget(help, chunks[help_idx]);
    }
}

fn draw_confirm_delete(f: &mut Frame, app: &App, idx: usize) {
    let area = centered_rect(50, 20, f.area());
    f.render_widget(Clear, area);

    let name = app
        .servers
        .get(idx)
        .map(|s| s.alias.as_str())
        .unwrap_or("?");

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("Delete '{}'?", name),
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "y: confirm  n/Esc: cancel",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let block = Block::default()
        .title(" Confirm Delete ")
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .style(Style::default().fg(Color::Red));

    let para = Paragraph::new(text)
        .block(block)
        .wrap(Wrap { trim: true })
        .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(para, area);
}

/// Returns a centered rect of `percent_x` × `percent_y` of `r`.
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
