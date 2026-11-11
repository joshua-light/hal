use crossterm::event::{KeyCode, KeyEvent};
use hal_lib::app::App;
use hal_lib::keymap::{self, SettingsKey};

/// The settings window. While a colour is being typed, keys belong to its
/// text box; otherwise the scheme's table says what they do.
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
    let Some(key) = keymap::resolve(app.keys.maps().settings, &key) else {
        return;
    };
    match key {
        SettingsKey::Close => app.close_settings(),
        SettingsKey::Down => app.settings_move(1),
        SettingsKey::Up => app.settings_move(-1),
        SettingsKey::Next => app.settings_step(1),
        SettingsKey::Prev => app.settings_step(-1),
        SettingsKey::Activate => app.settings_activate(),
        SettingsKey::Reset => app.settings_reset(),
    }
}
