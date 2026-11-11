use super::{typed, Typed};
use crossterm::event::KeyEvent;
#[cfg(test)]
use crossterm::event::{KeyCode, KeyModifiers};
use hal_lib::app::{App, Command, Tab, TasksCommand, View};
use hal_lib::keymap::{self, BoardKey, TaskInfoKey, TextKey};

/// Map a Tasks-board or Tasks-modal-submit key onto a [`Command`]. The
/// Esc-clears-filter binding claims Esc only while a filter is set, so an
/// unfiltered board's Esc falls through. Modal character/backspace editing
/// stays in [`handle`]; only the actions and the input/tags/filter submits
/// are commands.
pub(super) fn map_tasks_command(app: &App, key: &KeyEvent) -> Option<Command> {
    use TasksCommand as T;
    let maps = app.keys.maps();
    let submit = || keymap::resolve(maps.text_input, key) == Some(TextKey::Submit);
    let cmd = match app.view {
        View::Grid if app.current_tab == Tab::Tasks => match keymap::resolve(maps.board, key)? {
            BoardKey::Do(T::ClearFilter) if app.tasks.filter.is_empty() => return None,
            BoardKey::Do(cmd) => cmd,
            // Reads the clipboard: a bin arm, see `handle`.
            BoardKey::PasteNote => return None,
        },
        View::TaskInfo => match keymap::resolve(maps.task_info, key)? {
            TaskInfoKey::Attach => T::OpenAttachInput { from_info: true },
            TaskInfoKey::Remove => T::RemoveAttachment,
            _ => return None,
        },
        View::TaskInput if submit() => T::SubmitInput,
        View::TaskTags if submit() => T::SubmitTags,
        View::TaskAttachInput if submit() => T::SubmitAttach,
        View::TaskFilter if submit() => T::ApplyFilter,
        _ => return None,
    };
    Some(Command::Tasks(cmd))
}

/// Handle the Tasks modal buffer-edit keys that stay in `bin` (typing into the
/// add/rename, tag, and filter buffers, plus their Esc/Backspace). Board
/// actions and modal submits are commands (see [`map_tasks_command`]); this
/// runs only after `map_tasks_command` returns `None`. Returns true when it
/// consumed the key.
pub(super) fn handle(app: &mut App, key: KeyEvent) -> bool {
    let maps = app.keys.maps();
    match app.view {
        View::Grid if app.current_tab == Tab::Tasks => {
            // `p`/Ctrl+V: attach the clipboard's text as a note. The
            // clipboard read stays bin-side like the copy arm below, so App
            // (and its tests) never touch the real clipboard.
            if keymap::resolve(maps.board, &key) != Some(BoardKey::PasteNote) {
                return false;
            }
            paste_note(app);
            true
        }
        View::TaskInfo => match keymap::resolve(maps.task_info, &key) {
            Some(action) => {
                task_info(app, action);
                true
            }
            None => false,
        },
        _ => edit_buffer(app, key),
    }
}

/// Task Info popup: nav + per-attachment actions. Attach and remove are
/// commands (see `map_tasks_command`); copy/open stay here because they
/// need the clipboard / OS opener.
fn task_info(app: &mut App, action: TaskInfoKey) {
    match action {
        TaskInfoKey::Close => app.close_task_info(),
        TaskInfoKey::Next => app.task_info_next(),
        TaskInfoKey::Prev => app.task_info_prev(),
        TaskInfoKey::PageDown => app.task_info_scroll_by(10),
        TaskInfoKey::PageUp => app.task_info_scroll_by(-10),
        TaskInfoKey::CopyPath => match app.selected_task_attachment().map(|a| a.path.clone()) {
            None => app.set_status("no attachment to copy".into()),
            Some(path) => match hal_lib::clipboard::copy(&path) {
                Ok(()) => app.set_status(format!("copied: {}", path)),
                Err(e) => app.set_status(format!("copy failed: {}", e)),
            },
        },
        TaskInfoKey::Open => match app.selected_task_attachment().map(|a| a.path.clone()) {
            None => app.set_status("no attachment to open".into()),
            Some(path) => match crate::effects::open_path_detached(&path) {
                Ok(()) => app.set_status(format!("opening {}", path)),
                Err(e) => app.set_status(format!("open failed: {}", e)),
            },
        },
        TaskInfoKey::PasteNote => paste_note(app),
        TaskInfoKey::Attach | TaskInfoKey::Remove => {}
    }
}

/// Typing into the add/rename, tag, attach and filter buffers.
fn edit_buffer(app: &mut App, key: KeyEvent) -> bool {
    use TextKey as K;
    let key = typed(app.keys.maps().text_input, &key);
    match (&app.view, key) {
        (View::TaskInput, Typed::Do(K::Cancel)) => app.close_task_input(),
        (View::TaskInput, Typed::Do(K::Switch)) => app.toggle_task_input_field(),
        (View::TaskInput, Typed::Do(K::Erase)) => {
            app.task_input_buffer().pop();
        }
        (View::TaskInput, Typed::Char(c)) => app.task_input_buffer().push(c),
        (View::TaskTags, Typed::Do(K::Cancel)) => app.close_task_tags(),
        (View::TaskTags, Typed::Do(K::Erase)) => {
            app.tasks.input.pop();
        }
        (View::TaskTags, Typed::Char(c)) => app.tasks.input.push(c),
        (View::TaskAttachInput, Typed::Do(K::Cancel)) => app.close_task_attach(),
        (View::TaskAttachInput, Typed::Do(K::Switch)) => app.toggle_task_attach_mode(),
        (View::TaskAttachInput, Typed::Do(K::Erase)) => {
            app.tasks.input.pop();
        }
        (View::TaskAttachInput, Typed::Char(c)) => app.tasks.input.push(c),
        (View::TaskFilter, Typed::Do(K::Cancel)) => app.clear_task_filter(),
        (View::TaskFilter, Typed::Do(K::Erase)) => app.task_filter_pop(),
        (View::TaskFilter, Typed::Char(c)) => app.task_filter_push(c),
        _ => return false,
    }
    true
}

/// Read the host clipboard and attach its text to the focused card as a
/// `note` attachment. All failure modes land in the status bar.
fn paste_note(app: &mut App) {
    match hal_lib::clipboard::paste() {
        Ok(text) => match app.attach_text_to_selected(&text) {
            Some(msg) => app.set_status(msg),
            None => app.set_status("no task focused".into()),
        },
        Err(e) => app.set_status(format!("clipboard read failed: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys::map_command;
    use hal_lib::app::GlobalCommand;
    use hal_lib::keymap::Scheme;

    fn press(app: &App, key: KeyCode) -> Option<Command> {
        map_command(app, &KeyEvent::from(key))
    }

    fn on_board(scheme: Scheme) -> App {
        let mut app = App::new();
        app.keys = scheme;
        app.current_tab = Tab::Tasks;
        app
    }

    // The board's keys run ahead of the global ones; `J`/`K` cycle tabs
    // everywhere and must come through.
    #[test]
    fn vim_board_keeps_the_tab_cycle_and_its_letters() {
        let app = on_board(Scheme::Vim);
        for c in ['J', 'K'] {
            assert!(matches!(
                press(&app, KeyCode::Char(c)),
                Some(Command::Global(GlobalCommand::CycleTab { .. }))
            ));
        }
        assert_eq!(
            press(&app, KeyCode::Char('T')),
            Some(Command::Tasks(TasksCommand::OpenKindPicker))
        );
        assert_eq!(
            press(&app, KeyCode::Char('j')),
            Some(Command::Tasks(TasksCommand::NavDown))
        );
    }

    #[test]
    fn ide_board_moves_with_arrows_and_ignores_hjkl() {
        let app = on_board(Scheme::Ide);
        assert_eq!(
            press(&app, KeyCode::Down),
            Some(Command::Tasks(TasksCommand::NavDown))
        );
        assert_eq!(
            map_command(&app, &KeyEvent::new(KeyCode::Right, KeyModifiers::SHIFT)),
            Some(Command::Tasks(TasksCommand::MoveTaskRight))
        );
        assert_eq!(
            map_command(
                &app,
                &KeyEvent::new(KeyCode::Char('n'), KeyModifiers::CONTROL)
            ),
            Some(Command::Tasks(TasksCommand::OpenAddInput))
        );
        for c in ['h', 'j', 'k', 'l', 'J', 'K'] {
            assert_eq!(press(&app, KeyCode::Char(c)), None, "{} is a Vim key", c);
        }
    }

    // Esc belongs to the board only while there is a filter to clear.
    #[test]
    fn esc_clears_a_filter_and_is_otherwise_free() {
        let mut app = on_board(Scheme::Ide);
        assert_eq!(press(&app, KeyCode::Esc), None);
        app.tasks.filter = "bug".into();
        assert_eq!(
            press(&app, KeyCode::Esc),
            Some(Command::Tasks(TasksCommand::ClearFilter))
        );
    }

    #[test]
    fn stray_j_under_ide_is_told_where_vim_keys_went() {
        let mut app = on_board(Scheme::Ide);
        assert!(crate::keys::is_stray_vim_key(
            &app,
            &KeyEvent::from(KeyCode::Char('j'))
        ));
        app.keys = Scheme::Vim;
        assert!(!crate::keys::is_stray_vim_key(
            &app,
            &KeyEvent::from(KeyCode::Char('j'))
        ));
    }
}
