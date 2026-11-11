use super::{typed, Typed};
use crossterm::event::KeyEvent;
use hal_lib::app::{App, Command, HarnessCommand as H, Tab, View};
use hal_lib::keymap::{self, TextKey};

/// Agents list and page key mapping. The key that gives the page the
/// keyboard is the one that gives a session's pane the keyboard on the
/// Sessions tab, and on the page it opens the selected thing: a run's
/// transcript, an artifact, a setting.
pub(super) fn map_harness_command(app: &App, key: &KeyEvent) -> Option<Command> {
    let maps = app.keys.maps();
    let editing = app.harness.detail.editing.is_some();
    let cmd = match app.view {
        View::Grid if app.current_tab == Tab::Agents => keymap::resolve(maps.agents, key)?,
        // Typing a setting's value: every printable key is text.
        View::AgentDetail if editing => match typed(maps.text_input, key) {
            Typed::Do(TextKey::Cancel) => H::EditCancel,
            Typed::Do(TextKey::Submit) => H::EditSubmit,
            Typed::Do(TextKey::Erase) => H::EditBackspace,
            Typed::Char(c) => H::EditChar(c),
            Typed::Do(TextKey::Switch) | Typed::Nothing => return None,
        },
        View::AgentDetail => keymap::resolve(maps.agent_page, key)?,
        _ => return None,
    };
    Some(Command::Harness(cmd))
}
