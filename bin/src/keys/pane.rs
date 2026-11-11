use crossterm::event::KeyEvent;
use hal_lib::app::App;
use hal_lib::keymap::{self, PaneKey};

/// The pane table's keys belong to the hub; every other key goes to the
/// pane.
pub(super) fn handle(app: &mut App, key: KeyEvent) {
    match keymap::resolve(app.keys.maps().pane, &key) {
        Some(PaneKey::Release) => {
            app.release_pane();
        }
        Some(PaneKey::Paste) => {
            let status = match hal_lib::clipboard::paste() {
                Ok(text) if text.is_empty() => Some("clipboard empty".to_string()),
                Ok(text) => match app.focused_pane() {
                    Some(pane) => pane
                        .paste_text(&text)
                        .err()
                        .map(|e| format!("paste failed: {}", e)),
                    None => None,
                },
                Err(e) => Some(format!("paste failed: {}", e)),
            };
            if let Some(msg) = status {
                app.set_status(msg);
            }
        }
        None => {
            if let Some(pane) = app.focused_pane() {
                pane.send_key(key);
            }
        }
    }
}
