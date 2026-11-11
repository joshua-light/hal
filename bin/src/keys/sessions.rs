use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use hal_lib::app::{App, Command, Tab, View};
use hal_lib::keymap;

/// Sessions-tab command mapping, and the Enter that submits its two text
/// popups.
pub(super) fn map_sessions_command(app: &App, key: &KeyEvent) -> Option<Command> {
    use hal_lib::app::SessionsCommand as S;
    let enter = key.code == KeyCode::Enter;
    match app.view {
        View::Grid if app.current_tab == Tab::Sessions => {
            map_agent_hotkey(key).or_else(|| keymap::resolve(app.keys.maps().sessions, key))
        }
        View::SessionFinder if enter => Some(Command::Sessions(S::ConfirmSessionFinder)),
        View::RenameSession if enter => Some(Command::Sessions(S::SubmitRename)),
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
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            app.execute(Command::Sessions(S::ConfirmClose));
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc | KeyCode::Char('q') => {
            app.cancel_confirm_close();
        }
        _ => {}
    }
}

/// Session finder: printable keys (space included — titles and first
/// messages are prose) belong to the fuzzy filter; navigation is
/// arrows plus ctrl-j/k and ctrl-n/p. Enter is a command
/// ([`map_sessions_command`]).
pub(super) fn handle_finder(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.close_session_finder();
        }
        KeyCode::Down => app.session_finder_move(1),
        KeyCode::Up => app.session_finder_move(-1),
        KeyCode::Backspace => {
            if let Some(finder) = app.session_finder.as_mut() {
                finder.pop_filter();
            }
        }
        KeyCode::Char(c) if key.modifiers.contains(KeyModifiers::CONTROL) => match c {
            'j' | 'n' => app.session_finder_move(1),
            'k' | 'p' => app.session_finder_move(-1),
            _ => {}
        },
        KeyCode::Char(c) => {
            if let Some(finder) = app.session_finder.as_mut() {
                finder.push_filter(c);
            }
        }
        _ => {}
    }
}

pub(super) fn handle_rename(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.close_rename_session();
        }
        KeyCode::Backspace => {
            app.rename_buffer.pop();
        }
        KeyCode::Char(c) => {
            app.rename_buffer.push(c);
        }
        _ => {}
    }
}
