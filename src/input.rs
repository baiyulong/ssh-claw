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
