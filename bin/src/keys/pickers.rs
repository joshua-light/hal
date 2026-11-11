use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use hal_lib::app::App;
use hal_lib::keymap::{self, ListKey};

/// Like the places picker, printable keys belong to the fuzzy filter;
/// use arrows or ctrl-j/k/n/p to navigate without stealing letters.
pub(super) fn model(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => app.close_model_picker(),
        KeyCode::Enter | KeyCode::Char(' ') => app.spawn_from_model_picker(),
        KeyCode::Tab => app.cycle_model_picker_agent(),
        KeyCode::Down => app.model_picker_move(1),
        KeyCode::Up => app.model_picker_move(-1),
        KeyCode::Backspace => {
            if let Some(picker) = app.model_picker.as_mut() {
                picker.pop_filter();
            }
        }
        KeyCode::Char(c) if key.modifiers.contains(KeyModifiers::CONTROL) => match c {
            'j' | 'n' => app.model_picker_move(1),
            'k' | 'p' => app.model_picker_move(-1),
            _ => {}
        },
        KeyCode::Char(c) => {
            if let Some(picker) = app.model_picker.as_mut() {
                picker.push_filter(c);
            }
        }
        _ => {}
    }
}

/// The plain list pickers below take the scheme's list keys.
fn list_key(app: &App, key: &KeyEvent) -> Option<ListKey> {
    keymap::resolve(app.keys.maps().list_picker, key)
}

pub(super) fn agent(app: &mut App, key: KeyEvent) {
    match list_key(app, &key) {
        Some(ListKey::Cancel) => app.close_agent_picker(),
        Some(ListKey::Confirm) => {
            app.confirm_default_session_agent();
        }
        Some(ListKey::Down) => app.agent_picker_move(1),
        Some(ListKey::Up) => app.agent_picker_move(-1),
        None => {}
    }
}

pub(super) fn respawn(app: &mut App, key: KeyEvent) {
    match list_key(app, &key) {
        Some(ListKey::Cancel) => app.close_respawn_picker(),
        Some(ListKey::Confirm) => {
            app.confirm_respawn_picker();
        }
        Some(ListKey::Down) => app.respawn_picker_move(1),
        Some(ListKey::Up) => app.respawn_picker_move(-1),
        None => {}
    }
}

pub(super) fn task_kind(app: &mut App, key: KeyEvent) {
    match list_key(app, &key) {
        Some(ListKey::Cancel) => app.close_task_kind_picker(),
        Some(ListKey::Confirm) => {
            app.confirm_task_kind();
        }
        Some(ListKey::Down) => app.task_kind_picker_move(1),
        Some(ListKey::Up) => app.task_kind_picker_move(-1),
        None => {}
    }
}

/// Same interaction model as the model picker: printable keys belong
/// to the fuzzy filter; arrows or ctrl-j/k/n/p navigate.
pub(super) fn task_link(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => app.close_task_link_picker(),
        KeyCode::Enter | KeyCode::Char(' ') => app.confirm_task_link_picker(),
        KeyCode::Down => app.task_link_picker_move(1),
        KeyCode::Up => app.task_link_picker_move(-1),
        KeyCode::Backspace => {
            if let Some(picker) = app.task_link_picker.as_mut() {
                picker.pop_filter();
            }
        }
        KeyCode::Char(c) if key.modifiers.contains(KeyModifiers::CONTROL) => match c {
            'j' | 'n' => app.task_link_picker_move(1),
            'k' | 'p' => app.task_link_picker_move(-1),
            _ => {}
        },
        KeyCode::Char(c) => {
            if let Some(picker) = app.task_link_picker.as_mut() {
                picker.push_filter(c);
            }
        }
        _ => {}
    }
}
