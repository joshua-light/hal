use super::{typed, Typed};
use crossterm::event::KeyEvent;
use hal_lib::app::App;
use hal_lib::keymap::{self, SettingsKey, TextKey};

/// The settings window. While a colour is being typed, keys belong to its
/// text box; otherwise the scheme's table says what they do.
pub(super) fn handle(app: &mut App, key: KeyEvent) {
    if let Some(text) = app.settings.editing.as_mut() {
        match typed(app.keys.maps().text_input, &key) {
            Typed::Do(TextKey::Cancel) => app.settings.editing = None,
            Typed::Do(TextKey::Submit) => app.settings_submit_color(),
            Typed::Do(TextKey::Erase) => {
                text.pop();
            }
            Typed::Char(c) => text.push(c),
            Typed::Do(TextKey::Switch) | Typed::Nothing => {}
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
