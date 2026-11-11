use super::{typed, Typed};
use crate::scan_msg::ScanMsg;
use crossterm::event::KeyEvent;
use hal_lib::app::App;
use hal_lib::folder_picker::PickerMode;
use hal_lib::keymap::{self, FilterKey, FolderKey, TextKey};
use hal_lib::spawn;
use tokio::sync::mpsc;

pub(super) fn handle(app: &mut App, key: KeyEvent) {
    // Places mode (task assign): printable keys type into the fuzzy
    // filter, so the scheme's picker bindings below must not fire.
    if app
        .folder_picker
        .as_ref()
        .is_some_and(|p| p.mode == PickerMode::Places)
    {
        match typed(app.keys.maps().filter_picker, &key) {
            Typed::Do(FilterKey::Cancel) => app.close_folder_picker(),
            Typed::Do(FilterKey::Confirm) => {
                // Don't let an empty match list cancel the picker —
                // pick_from_folder_picker closes on no selection.
                if app
                    .folder_picker
                    .as_ref()
                    .is_some_and(|p| p.selected_path().is_some())
                {
                    pick_from_folder_picker(app);
                }
            }
            Typed::Do(FilterKey::Switch) => app.toggle_places_picker_mode(),
            Typed::Do(FilterKey::Down) => {
                if let Some(p) = app.folder_picker.as_mut() {
                    p.move_down();
                }
            }
            Typed::Do(FilterKey::Up) => {
                if let Some(p) = app.folder_picker.as_mut() {
                    p.move_up();
                }
            }
            Typed::Do(FilterKey::Erase) => {
                if let Some(p) = app.folder_picker.as_mut() {
                    p.pop_filter();
                }
            }
            Typed::Char(c) => {
                if let Some(p) = app.folder_picker.as_mut() {
                    p.push_filter(c);
                }
            }
            Typed::Nothing => {}
        }
        return;
    }
    let Some(key) = keymap::resolve(app.keys.maps().folders, &key) else {
        return;
    };
    match key {
        FolderKey::Cancel => app.close_folder_picker(),
        // Browse → back to the places list.
        FolderKey::Places => app.toggle_places_picker_mode(),
        FolderKey::Down => {
            if let Some(p) = app.folder_picker.as_mut() {
                p.move_down();
            }
        }
        FolderKey::Up => {
            if let Some(p) = app.folder_picker.as_mut() {
                p.move_up();
            }
        }
        FolderKey::Bookmark => match app.toggle_selected_bookmark() {
            Some((true, path)) => app.set_status(format!("bookmarked {}", path)),
            Some((false, path)) => app.set_status(format!("unbookmarked {}", path)),
            None => app.set_status("no folder selected".into()),
        },
        FolderKey::Open => {
            if in_bookmarks(app) {
                pick_from_folder_picker(app);
            } else if let Some(p) = app.folder_picker.as_mut() {
                p.descend();
            }
        }
        FolderKey::Parent => {
            if let Some(p) = app.folder_picker.as_mut() {
                p.ascend();
            }
        }
        FolderKey::Pick => pick_from_folder_picker(app),
        FolderKey::PickCwd => {
            // Bookmarks mode has no meaningful "current dir" —
            // the entries are absolute paths from disk — so
            // collapse it into the same action as Pick.
            if in_bookmarks(app) {
                pick_from_folder_picker(app);
            } else {
                let cwd = app
                    .folder_picker
                    .as_ref()
                    .map(|p| p.current_dir.display().to_string());
                if let Some(cwd) = cwd {
                    dispatch_picked_cwd(app, &cwd);
                } else {
                    app.close_folder_picker();
                }
            }
        }
        FolderKey::GhPublic | FolderKey::GhPrivate => {
            if !in_bookmarks(app) {
                app.enter_gh_create_input(key == FolderKey::GhPrivate);
            }
        }
    }
}

pub(super) fn handle_gh_create(app: &mut App, key: KeyEvent, scan_tx: &mpsc::Sender<ScanMsg>) {
    match typed(app.keys.maps().text_input, &key) {
        Typed::Do(TextKey::Cancel) => {
            app.close_gh_create_input();
        }
        Typed::Do(TextKey::Switch) => {
            if let Some(input) = app.gh_create_input.as_mut() {
                input.private = !input.private;
            }
        }
        Typed::Do(TextKey::Erase) => {
            if let Some(input) = app.gh_create_input.as_mut() {
                input.name.pop();
            }
        }
        Typed::Char(c) => {
            if let Some(input) = app.gh_create_input.as_mut() {
                input.name.push(c);
            }
        }
        Typed::Do(TextKey::Submit) => {
            let name_empty = app
                .gh_create_input
                .as_ref()
                .is_none_or(|i| i.name.trim().is_empty());
            if name_empty {
                app.set_status("repo name cannot be empty".into());
                return;
            }
            if let Some((cwd, name, private)) = app.submit_gh_create_input() {
                let trimmed = name.trim().to_string();
                let tx = scan_tx.clone();
                let name_for_msg = trimmed.clone();
                tokio::spawn(async move {
                    let result = tokio::task::spawn_blocking(move || {
                        hal_lib::gh::create_repo(&cwd, &trimmed, private)
                    })
                    .await
                    .unwrap_or_else(|e| Err(format!("task panicked: {}", e)));
                    let _ = tx
                        .send(ScanMsg::GhCreateDone {
                            name: name_for_msg,
                            result,
                        })
                        .await;
                });
                app.set_status(format!(
                    "creating {} repo {}…",
                    if private { "private" } else { "public" },
                    name
                ));
            }
        }
        Typed::Nothing => {}
    }
}

fn in_bookmarks(app: &App) -> bool {
    app.folder_picker
        .as_ref()
        .is_some_and(|p| p.mode == PickerMode::Bookmarks)
}

/// Act on a folder-picker pick at `cwd`: assign the pending board task
/// there, or spawn a session.
fn dispatch_picked_cwd(app: &mut App, cwd: &str) {
    if app.tasks.pending_assign.is_some() {
        let status = app.assign_task_agent(cwd);
        app.set_status(status);
    } else {
        app.close_folder_picker();
        let agent_id = app.default_session_agent_id().to_string();
        let status = match spawn::spawn_agent_session(&agent_id, cwd, None, None, None, false) {
            Ok(name) => {
                let status = format!("started {} [{}]", agent_id, name);
                app.watch_spawn(name, agent_id, cwd.to_string());
                status
            }
            Err(e) => format!("spawn failed: {}", e),
        };
        app.set_status(status);
    }
}

/// Resolve the highlighted picker entry and dispatch it via
/// [`dispatch_picked_cwd`]. Works in both Browse and Bookmarks mode since
/// the lookup goes through `FolderPicker::selected_path`. No-ops by
/// closing the picker when nothing is selected.
fn pick_from_folder_picker(app: &mut App) {
    let cwd = app
        .folder_picker
        .as_ref()
        .and_then(|p| p.selected_path())
        .map(|p| p.display().to_string());
    match cwd {
        Some(cwd) => dispatch_picked_cwd(app, &cwd),
        None => app.close_folder_picker(),
    }
}
