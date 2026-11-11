use super::{typed, Typed};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use hal_lib::app::{App, Command, Tab, View};
use hal_lib::keymap::{self, ConfirmKey, FilterKey, TextKey};

/// Sessions-tab command mapping, and the key that submits its two text
/// popups.
pub(super) fn map_sessions_command(app: &App, key: &KeyEvent) -> Option<Command> {
    use hal_lib::app::SessionsCommand as S;
    let maps = app.keys.maps();
    let open = keymap::resolve(maps.session_finder, key) == Some(FilterKey::Confirm);
    let submit = keymap::resolve(maps.text_input, key) == Some(TextKey::Submit);
    match app.view {
        View::Grid if app.current_tab == Tab::Sessions => {
            map_agent_hotkey(key).or_else(|| keymap::resolve(maps.sessions, key))
        }
        View::SessionFinder if open => Some(Command::Sessions(S::ConfirmSessionFinder)),
        View::RenameSession if submit => Some(Command::Sessions(S::SubmitRename)),
        _ => None,
    }
}

/// `[agents.<id>].hotkey` bindings on the Sessions grid. Checked before the
/// scheme's table so a user who binds `N` to Claude gets Claude, not the
/// model picker — the config doc says custom hotkeys shadow built-ins.
/// Ctrl/Alt/Cmd chords are left alone; Shift is what makes an uppercase char.
fn map_agent_hotkey(key: &KeyEvent) -> Option<Command> {
    use hal_lib::app::SessionsCommand as S;
    if key
        .modifiers
        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER)
    {
        return None;
    }
    let KeyCode::Char(c) = key.code else {
        return None;
    };
    let agent_id = hal_lib::config::get().agent_for_hotkey(c)?;
    Some(Command::Sessions(S::SpawnAgentHereWith { agent_id }))
}

pub(super) fn handle_confirm_close(app: &mut App, key: KeyEvent) {
    use hal_lib::app::SessionsCommand as S;
    match keymap::resolve(app.keys.maps().confirm_close, &key) {
        Some(ConfirmKey::Yes) => {
            app.execute(Command::Sessions(S::ConfirmClose));
        }
        Some(ConfirmKey::No) => app.cancel_confirm_close(),
        None => {}
    }
}

/// Session finder: printable keys (space included — titles and first
/// messages are prose) belong to the fuzzy filter. Confirm is a command
/// ([`map_sessions_command`]).
pub(super) fn handle_finder(app: &mut App, key: KeyEvent) {
    match typed(app.keys.maps().session_finder, &key) {
        Typed::Do(FilterKey::Cancel) => app.close_session_finder(),
        Typed::Do(FilterKey::Down) => app.session_finder_move(1),
        Typed::Do(FilterKey::Up) => app.session_finder_move(-1),
        Typed::Do(FilterKey::Erase) => {
            if let Some(finder) = app.session_finder.as_mut() {
                finder.pop_filter();
            }
        }
        Typed::Char(c) => {
            if let Some(finder) = app.session_finder.as_mut() {
                finder.push_filter(c);
            }
        }
        Typed::Do(FilterKey::Confirm | FilterKey::Switch) | Typed::Nothing => {}
    }
}

pub(super) fn handle_rename(app: &mut App, key: KeyEvent) {
    match typed(app.keys.maps().text_input, &key) {
        Typed::Do(TextKey::Cancel) => app.close_rename_session(),
        Typed::Do(TextKey::Erase) => {
            app.rename_buffer.pop();
        }
        Typed::Char(c) => app.rename_buffer.push(c),
        Typed::Do(TextKey::Submit | TextKey::Switch) | Typed::Nothing => {}
    }
}
