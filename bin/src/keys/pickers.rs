use super::{typed, Typed};
use crossterm::event::KeyEvent;
use hal_lib::app::App;
use hal_lib::keymap::{self, FilterKey, ListKey};

/// Like the places picker, printable keys belong to the fuzzy filter; the
/// table's arrows and Ctrl chords navigate without stealing letters.
pub(super) fn model(app: &mut App, key: KeyEvent) {
    match typed(app.keys.maps().filter_picker, &key) {
        Typed::Do(FilterKey::Cancel) => app.close_model_picker(),
        Typed::Do(FilterKey::Confirm) => app.spawn_from_model_picker(),
        Typed::Do(FilterKey::Switch) => app.cycle_model_picker_agent(),
        Typed::Do(FilterKey::Down) => app.model_picker_move(1),
        Typed::Do(FilterKey::Up) => app.model_picker_move(-1),
        Typed::Do(FilterKey::Erase) => {
            if let Some(picker) = app.model_picker.as_mut() {
                picker.pop_filter();
            }
        }
        Typed::Char(c) => {
            if let Some(picker) = app.model_picker.as_mut() {
                picker.push_filter(c);
            }
        }
        Typed::Nothing => {}
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

/// Same interaction model as the model picker.
pub(super) fn task_link(app: &mut App, key: KeyEvent) {
    match typed(app.keys.maps().filter_picker, &key) {
        Typed::Do(FilterKey::Cancel) => app.close_task_link_picker(),
        Typed::Do(FilterKey::Confirm) => app.confirm_task_link_picker(),
        Typed::Do(FilterKey::Down) => app.task_link_picker_move(1),
        Typed::Do(FilterKey::Up) => app.task_link_picker_move(-1),
        Typed::Do(FilterKey::Erase) => {
            if let Some(picker) = app.task_link_picker.as_mut() {
                picker.pop_filter();
            }
        }
        Typed::Char(c) => {
            if let Some(picker) = app.task_link_picker.as_mut() {
                picker.push_filter(c);
            }
        }
        Typed::Do(FilterKey::Switch) | Typed::Nothing => {}
    }
}
