use crossterm::event::{KeyCode, KeyEvent};
use hal_lib::app::App;

/// The settings window: j/k move, h/l change, Enter edits a colour (or steps
/// any other row), d drops the row back to its default. While a colour is
/// being typed, keys belong to its text box.
pub(super) fn handle(app: &mut App, key: KeyEvent) {
    if let Some(text) = app.settings.editing.as_mut() {
        match key.code {
            KeyCode::Esc => app.settings.editing = None,
            KeyCode::Enter => app.settings_submit_color(),
            KeyCode::Backspace => {
                text.pop();
            }
            KeyCode::Char(c) => text.push(c),
            _ => {}
        }
        return;
    }
    match key.code {
        KeyCode::Esc | KeyCode::Char('q' | ',') => app.close_settings(),
        KeyCode::Down | KeyCode::Char('j') => app.settings_move(1),
        KeyCode::Up | KeyCode::Char('k') => app.settings_move(-1),
        KeyCode::Right | KeyCode::Char('l') => app.settings_step(1),
        KeyCode::Left | KeyCode::Char('h') => app.settings_step(-1),
        KeyCode::Enter | KeyCode::Char(' ') => app.settings_activate(),
        KeyCode::Char('d') | KeyCode::Delete | KeyCode::Backspace => app.settings_reset(),
        _ => {}
    }
}
