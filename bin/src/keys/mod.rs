//! Key dispatch for the TUI. A key first maps onto a lib [`Command`]
//! ([`map_command`] -> `App::execute` -> [`crate::effects`]); keys no command
//! covers go to the current view's bin-side handler.
//!
//! Which key does what comes from the tables of the key scheme in force
//! ([`hal_lib::keymap`]), so `[keymap]` in the config can move any of them.
//! No handler here names a key: text inputs and filter pickers ask their
//! table first ([`typed`]) and type what it does not bind. A test below
//! fails on a handler that matches a key code itself.
//!
//! - `sessions`: Sessions-tab commands, close confirmation, session finder,
//!   rename.
//! - `tasks`, `harness`: the Tasks and Agents tabs.
//! - `folder_picker`: the folder picker and its `gh repo create` input.
//! - `pickers`: model, agent, respawn, task-kind and task-link pickers.
//! - `metrics`: the Metrics tab.
//! - `pane`: the embedded tmux pane, popup or docked.
//! - `settings`: the settings window.

use crate::scan_msg::ScanMsg;
use crate::term::Term;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use hal_lib::app::{App, Command, Tab, View};
use hal_lib::keymap::{self, Keymap, Scheme, ScrollKey};
use tokio::sync::mpsc;

mod folder_picker;
mod harness;
mod metrics;
mod pane;
mod pickers;
mod sessions;
mod settings;
mod tasks;

/// What a key means where printable keys are text.
pub(super) enum Typed<A> {
    Do(A),
    Char(char),
    Nothing,
}

/// `key` in a text box or filter picker: its table's action, else the
/// character it types. A Ctrl, Alt or Cmd chord types nothing.
pub(super) fn typed<A: Copy>(map: Keymap<A>, key: &KeyEvent) -> Typed<A> {
    if let Some(action) = keymap::resolve(map, key) {
        return Typed::Do(action);
    }
    let chord = KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER;
    match key.code {
        KeyCode::Char(c) if !key.modifiers.intersects(chord) => Typed::Char(c),
        _ => Typed::Nothing,
    }
}

/// Map a key press onto a [`Command`] when a converted arm covers it.
///
/// `None` sends the key on to [`handle_key`]'s per-view handlers. A tab's
/// own keys come first, then the ones every tab shares; the scheme's tables
/// keep the two apart.
pub(super) fn map_command(app: &App, key: &KeyEvent) -> Option<Command> {
    let maps = app.keys.maps();
    let on_grid = app.view == View::Grid;
    tasks::map_tasks_command(app, key)
        .or_else(|| harness::map_harness_command(app, key))
        .or_else(|| sessions::map_sessions_command(app, key))
        .or_else(|| {
            on_grid
                .then(|| keymap::resolve(maps.global, key))
                .flatten()
                .map(Command::Global)
        })
}

/// Dispatch a single key press: commands first, then the Tasks modal
/// editors, then the view's own bin-side handler.
pub(crate) async fn handle_key(
    app: &mut App,
    key: KeyEvent,
    terminal: &Term,
    scan_tx: &mpsc::Sender<ScanMsg>,
    detail_tx: &mpsc::Sender<String>,
) {
    let mut effects = match map_command(app, &key) {
        Some(cmd) => app.execute(cmd),
        None => {
            if is_stray_vim_key(app, &key) {
                app.set_status("j/k are Vim keys: press , and set Keys to Vim".into());
            }
            handle_view_key(app, key, scan_tx);
            Vec::new()
        }
    };
    // A picker that spawns a session opens it from here: its handler has no
    // way to hand back effects.
    effects.extend(app.take_queued_effects());
    for effect in effects {
        crate::effects::apply_effect(app, effect, terminal, scan_tx, detail_tx).await;
    }
}

/// Whether the key is `j` or `k` pressed on a tab under the IDE scheme,
/// which binds neither: most likely a Vim user after the default changed.
fn is_stray_vim_key(app: &App, key: &KeyEvent) -> bool {
    app.keys == Scheme::Ide
        && app.view == View::Grid
        && matches!(key.code, KeyCode::Char(c) if matches!(c, 'j' | 'k'))
        && !key
            .modifiers
            .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER)
}

/// Keys no command covers: the Tasks modal editors, then the view's own
/// handler.
fn handle_view_key(app: &mut App, key: KeyEvent, scan_tx: &mpsc::Sender<ScanMsg>) {
    let on_metrics = app.view == View::Grid && app.current_tab == Tab::Metrics;
    if tasks::handle(app, key) {
        return;
    }
    // Every view's arms are disjoint, so routing on the view first keeps
    // each view's own arm order.
    match app.view {
        View::Grid if on_metrics => metrics::handle(app, key, scan_tx),
        View::TmuxPane | View::Dock => pane::handle(app, key),
        View::ConfirmClose => sessions::handle_confirm_close(app, key),
        View::FolderPicker => folder_picker::handle(app, key),
        View::GhCreateInput => folder_picker::handle_gh_create(app, key, scan_tx),
        View::ModelPicker => pickers::model(app, key),
        View::AgentPicker => pickers::agent(app, key),
        View::RespawnPicker => pickers::respawn(app, key),
        View::TaskKindPicker => pickers::task_kind(app, key),
        View::TaskLinkPicker => pickers::task_link(app, key),
        View::SessionFinder => sessions::handle_finder(app, key),
        View::RenameSession => sessions::handle_rename(app, key),
        View::Settings => settings::handle(app, key),
        View::Popup => match keymap::resolve(app.keys.maps().popup, &key) {
            Some(ScrollKey::Close) => app.close_popup(),
            Some(ScrollKey::Down) => app.scroll_down(),
            Some(ScrollKey::Up) => app.scroll_up(),
            Some(ScrollKey::Bottom) | None => {}
        },
        View::LiveTail => {
            let key = keymap::resolve(app.keys.maps().live_tail, &key);
            if key == Some(ScrollKey::Close) {
                app.close_live_tail();
            } else if let Some(lv) = app.live_view.as_mut() {
                match key {
                    Some(ScrollKey::Down) => lv.scroll_down(),
                    Some(ScrollKey::Up) => lv.scroll_up(),
                    Some(ScrollKey::Bottom) => lv.scroll_bottom(),
                    Some(ScrollKey::Close) | None => {}
                }
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    /// `[keymap]` can only move a key that a table binds. A handler that
    /// matches `KeyCode::Esc` or `KeyCode::Char('x')` itself puts that key
    /// out of the user's reach, so none may: bind it in `hal_lib::keymap`
    /// and resolve it. `KeyCode::Char(c)`, a typed character, is fine.
    #[test]
    fn no_handler_names_a_key() {
        let sources = [
            ("keys/mod.rs", include_str!("mod.rs")),
            ("keys/folder_picker.rs", include_str!("folder_picker.rs")),
            ("keys/harness.rs", include_str!("harness.rs")),
            ("keys/metrics.rs", include_str!("metrics.rs")),
            ("keys/pane.rs", include_str!("pane.rs")),
            ("keys/pickers.rs", include_str!("pickers.rs")),
            ("keys/sessions.rs", include_str!("sessions.rs")),
            ("keys/settings.rs", include_str!("settings.rs")),
            ("keys/tasks.rs", include_str!("tasks.rs")),
            (
                "event_loop/input.rs",
                include_str!("../event_loop/input.rs"),
            ),
        ];
        // Built here so this file does not trip its own check.
        let needle = ["Key", "Code::"].concat();
        for (file, source) in sources {
            let code = source.split("#[cfg(test)]\nmod tests").next().unwrap();
            for (n, line) in code.lines().enumerate() {
                if line.trim_start().starts_with("//") {
                    continue;
                }
                for (at, _) in line.match_indices(&needle) {
                    let rest = &line[at + needle.len()..];
                    assert!(
                        rest.starts_with("Char(c)"),
                        "{}:{} names a key; bind it in a keymap table instead:\n{}",
                        file,
                        n + 1,
                        line.trim()
                    );
                }
            }
        }
    }

    /// Every file under `keys/` is in the list above.
    #[test]
    fn the_check_covers_every_handler_file() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/keys");
        let count = std::fs::read_dir(dir).unwrap().count();
        assert_eq!(count, 9, "add the new file to no_handler_names_a_key");
    }
}
