use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use hal_lib::app::{App, Command, HarnessCommand as H, Tab, View};
use hal_lib::keymap;

/// Agents list and page key mapping. The key that gives the page the
/// keyboard is the one that gives a session's pane the keyboard on the
/// Sessions tab, and on the page it opens the selected thing: a run's
/// transcript, an artifact, a setting.
pub(super) fn map_harness_command(app: &App, key: &KeyEvent) -> Option<Command> {
    let maps = app.keys.maps();
    let editing = app.harness.detail.editing.is_some();
    let cmd = match (&app.view, key.code) {
        (View::Grid, _) if app.current_tab == Tab::Agents => keymap::resolve(maps.agents, key)?,

        // Typing a setting's value: every printable key is text.
        (View::AgentDetail, KeyCode::Esc) if editing => H::EditCancel,
        (View::AgentDetail, KeyCode::Enter) if editing => H::EditSubmit,
        (View::AgentDetail, KeyCode::Backspace) if editing => H::EditBackspace,
        (View::AgentDetail, KeyCode::Char(c))
            if editing && !key.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            H::EditChar(c)
        }
        (View::AgentDetail, _) if editing => return None,

        (View::AgentDetail, _) => keymap::resolve(maps.agent_page, key)?,
        _ => return None,
    };
    Some(Command::Harness(cmd))
}
