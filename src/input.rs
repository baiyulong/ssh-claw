use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, Screen};

pub fn handle_key(app: &mut App, key: KeyEvent) {
    match &app.screen {
        Screen::Dashboard => handle_dashboard(app, key),
        Screen::AddForm | Screen::EditForm(_) => handle_form(app, key),
        Screen::ConfirmDelete(idx) => {
            let idx = *idx;
            handle_confirm_delete(app, key, idx);
        }
        Screen::SshSession(_) => handle_ssh_session(app, key),
    }
}

fn handle_dashboard(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char('j') | KeyCode::Down => app.move_selection_down(),
        KeyCode::Char('k') | KeyCode::Up => app.move_selection_up(),
        KeyCode::Char('a') => app.start_add(),
        KeyCode::Char('e') => app.start_edit(),
        KeyCode::Char('d') => app.confirm_delete(),
        KeyCode::Enter => app.initiate_ssh(),
        _ => {}
    }
}

fn handle_form(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => app.cancel_form(),
        KeyCode::Enter => app.submit_form(),
        KeyCode::Tab => {
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                app.form.focused = app.form.focused.prev();
            } else {
                app.form.focused = app.form.focused.next();
            }
        }
        KeyCode::BackTab => {
            app.form.focused = app.form.focused.prev();
        }
        KeyCode::Backspace => {
            let field = app.form.get_field_mut(app.form.focused);
            field.pop();
        }
        KeyCode::Char(c) => {
            let field = app.form.get_field_mut(app.form.focused);
            field.push(c);
        }
        _ => {}
    }
}

fn handle_confirm_delete(app: &mut App, key: KeyEvent, idx: usize) {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => app.do_delete(idx),
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.screen = Screen::Dashboard;
        }
        _ => {}
    }
}

/// Forward every key as raw terminal bytes to the PTY writer.
/// If the session has already exited, any keypress returns to the dashboard.
fn handle_ssh_session(app: &mut App, key: KeyEvent) {
    if let Screen::SshSession(ref mut session) = app.screen {
        if session.is_exited() {
            // Session ended — any key returns to the server list
            app.screen = Screen::Dashboard;
            app.status_msg = "SSH session ended.".to_string();
            return;
        }
        let bytes = key_to_bytes(key);
        if !bytes.is_empty() {
            let _ = session.write_bytes(&bytes);
        }
    }
}

/// Convert a crossterm `KeyEvent` to the byte sequence a terminal expects.
fn key_to_bytes(key: KeyEvent) -> Vec<u8> {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    let alt = key.modifiers.contains(KeyModifiers::ALT);

    match key.code {
        // Ctrl+letter → ASCII control code 1–26
        KeyCode::Char(c) if ctrl => {
            if c.is_ascii_alphabetic() {
                vec![(c.to_ascii_lowercase() as u8) - b'a' + 1]
            } else {
                match c {
                    '[' => vec![27],
                    '\\' => vec![28],
                    ']' => vec![29],
                    '^' => vec![30],
                    '_' => vec![31],
                    _ => vec![c as u8],
                }
            }
        }
        // Alt+char → ESC prefix
        KeyCode::Char(c) if alt => {
            let mut buf = [0u8; 4];
            let s = c.encode_utf8(&mut buf);
            let mut out = vec![27u8];
            out.extend_from_slice(s.as_bytes());
            out
        }
        KeyCode::Char(c) => {
            let mut buf = [0u8; 4];
            c.encode_utf8(&mut buf).as_bytes().to_vec()
        }
        KeyCode::Enter => vec![b'\r'],
        KeyCode::Backspace => vec![127],
        KeyCode::Delete => vec![27, b'[', b'3', b'~'],
        KeyCode::Tab => vec![b'\t'],
        KeyCode::BackTab => vec![27, b'[', b'Z'],
        KeyCode::Esc => vec![27],
        KeyCode::Up => vec![27, b'[', b'A'],
        KeyCode::Down => vec![27, b'[', b'B'],
        KeyCode::Right => vec![27, b'[', b'C'],
        KeyCode::Left => vec![27, b'[', b'D'],
        KeyCode::Home => vec![27, b'[', b'H'],
        KeyCode::End => vec![27, b'[', b'F'],
        KeyCode::PageUp => vec![27, b'[', b'5', b'~'],
        KeyCode::PageDown => vec![27, b'[', b'6', b'~'],
        KeyCode::F(n) => match n {
            1 => vec![27, b'O', b'P'],
            2 => vec![27, b'O', b'Q'],
            3 => vec![27, b'O', b'R'],
            4 => vec![27, b'O', b'S'],
            5 => vec![27, b'[', b'1', b'5', b'~'],
            6 => vec![27, b'[', b'1', b'7', b'~'],
            7 => vec![27, b'[', b'1', b'8', b'~'],
            8 => vec![27, b'[', b'1', b'9', b'~'],
            9 => vec![27, b'[', b'2', b'0', b'~'],
            10 => vec![27, b'[', b'2', b'1', b'~'],
            11 => vec![27, b'[', b'2', b'3', b'~'],
            12 => vec![27, b'[', b'2', b'4', b'~'],
            _ => vec![],
        },
        _ => vec![],
    }
}
