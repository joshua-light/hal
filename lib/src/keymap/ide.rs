//! The IDE scheme: arrows move, and the actions an editor has a usual chord
//! for take it (`Ctrl+N` new, `Ctrl+T` find, `Ctrl+P` open a folder, `F2`
//! rename, `Del` delete, `Ctrl+Z` undo). The rest keep their letter. `j` and `k` stay unbound on
//! every tab: `bin/` answers them with where the Vim keys went.
//!
//! Chords stick to what a terminal without the kitty protocol can report:
//! no `Ctrl+Shift+letter`, and none of `Ctrl+I`/`M`/`H`/`J`, which arrive as
//! Tab, Enter and Backspace. Keys a laptop may lack have a second key: `Ins`
//! follows `Ctrl+N`, `Bksp` follows `Del`, a letter follows `F2` and `F5`.
//!
//! Rows are in hint order, as in [`super::vim`].

use super::*;
use crate::app::{HarnessCommand as H, Section, SessionsCommand as S, Tab, TasksCommand as T};
use crate::tasks::store::TaskPriority;
use BoardKey::Do;

const fn s(cmd: S) -> Command {
    Command::Sessions(cmd)
}

const NEW: Key = ctrl(ch('n'));

pub(super) static MAPS: Keymaps = Keymaps {
    global: &[
        // Ctrl+Tab reaches the hub only under the kitty protocol.
        bind!(GlobalCommand::CycleTab { back: false }, [TAB, ctrl(TAB)], "tab" => "next"),
        bind!(
            GlobalCommand::CycleTab { back: true },
            [BACKTAB, ctrl(BACKTAB)]
        ),
        bind!(GlobalCommand::OpenSettings, [ch(',')], "settings"),
        bind!(GlobalCommand::Quit, [ch('q'), ctrl(ch('q'))], "q" => "quit"),
    ],
    sessions: &[
        bind!(s(S::FocusSelected), [ENTER], "type in/resume"),
        bind!(s(S::OpenSessionFinder), [ctrl(ch('t')), ch('/')], "ctrl+t" => "find any"),
        bind!(s(S::SpawnAgentHere), [NEW, INS], "ctrl+n" => "new"),
        bind!(s(S::ToggleHandoff), [ch('h')], "handoff"),
        bind!(s(S::OpenRespawnPicker), [ch('R')], "respawn"),
        bind!(s(S::OpenAgentPicker), [ch('A')], "default agent"),
        bind!(s(S::OpenModelPicker), [ch('N')], "new+model"),
        bind!(s(S::OpenPlacesPicker), [ctrl(ch('p'))], "new in…"),
        bind!(s(S::OpenDetailPopup), [ch('i')], "info"),
        bind!(s(S::AckSelected), [ch('z')], "ack"),
        bind!(s(S::OpenRenameSession), [F2, ch('r')], "F2" => "rename"),
        bind!(s(S::OpenTaskLinkPicker), [ch('L')], "link task"),
        bind!(s(S::OpenTerminalHere), [ch('t')], "shell"),
        bind!(s(S::OpenBookmarksPicker), [ch('M')], "bookmarks"),
        bind!(s(S::NavLeft), [LEFT], "arrows" => "nav"),
        bind!(s(S::NavDown), [DOWN], join),
        bind!(s(S::NavUp), [UP], join),
        bind!(s(S::NavRight), [RIGHT], join),
        bind!(s(S::ToggleLayout), [ch('v')], "layout"),
        bind!(s(S::StageConfirmClose), [DEL, BKSP], "del" => "close"),
        bind!(s(S::ToggleShowInactive), [ch('H')], "inactive"),
        bind!(s(S::ToggleHold), [SPACE]),
        bind!(
            Command::Global(GlobalCommand::SetTab(Tab::Metrics)),
            [ch('m')]
        ),
    ],
    board: &[
        bind!(Do(T::OpenAddInput), [NEW, INS], "ctrl+n" => "add"),
        bind!(Do(T::FocusAgent), [ENTER], "focus agent"),
        bind!(Do(T::OpenTaskInfo), [ch('v')], "info"),
        bind!(Do(T::OpenAssignPicker), [ch('s')], "assign agent"),
        bind!(Do(T::AssignAtHome), [ch('S')], "agent in ~"),
        bind!(Do(T::NavLeft), [LEFT], "←/→" => "col"),
        bind!(Do(T::NavRight), [RIGHT], join),
        bind!(Do(T::NavDown), [DOWN], "↑/↓" => "task"),
        bind!(Do(T::NavUp), [UP], join),
        bind!(Do(T::MoveTaskLeft), [shift(LEFT)], "shift+←/→" => "move"),
        bind!(Do(T::MoveTaskRight), [shift(RIGHT)], join),
        bind!(Do(T::OpenFilter), [ctrl(ch('f')), ch('/')], "ctrl+f" => "filter"),
        bind!(Do(T::ClearFilter), [ESC]),
        bind!(Do(T::SetPriority(TaskPriority::P1)), [ch('1')], "1-4" => "priority"),
        bind!(Do(T::SetPriority(TaskPriority::P2)), [ch('2')], join),
        bind!(Do(T::SetPriority(TaskPriority::P3)), [ch('3')], join),
        bind!(Do(T::SetPriority(TaskPriority::P4)), [ch('4')], join),
        bind!(Do(T::OpenTags), [ch('t')], "tags"),
        bind!(Do(T::OpenKindPicker), [ch('T')], "kind"),
        bind!(Do(T::OpenRename), [F2, ch('r')], "F2" => "rename"),
        bind!(
            Do(T::OpenAttachInput { from_info: false }),
            [ch('A')],
            "attach"
        ),
        // Some terminals keep Ctrl+V for their own paste; `p` always
        // arrives.
        bind!(BoardKey::PasteNote, [ctrl(ch('v')), ch('p')], "paste note"),
        bind!(Do(T::DeleteSelected), [DEL, BKSP], "del" => "delete"),
        bind!(Do(T::UndoDelete), [ctrl(ch('z'))], "undo"),
        bind!(Do(T::ClearDone), [ch('c')], "clear done"),
        bind!(Do(T::SpaceAction), [SPACE]),
    ],
    task_info: &[
        bind!(TaskInfoKey::Next, [DOWN], "↑/↓" => "attachment"),
        bind!(TaskInfoKey::Prev, [UP], join),
        bind!(TaskInfoKey::Attach, [ch('a')], "attach"),
        bind!(
            TaskInfoKey::PasteNote,
            [ctrl(ch('v')), ch('p')],
            "paste note"
        ),
        bind!(TaskInfoKey::CopyPath, [ctrl(ch('c')), ch('c')], "ctrl+c" => "copy path"),
        bind!(TaskInfoKey::Open, [ENTER, ch('o')], "enter" => "open"),
        bind!(TaskInfoKey::Remove, [DEL, BKSP], "del" => "remove"),
        bind!(TaskInfoKey::PageUp, [PGUP], "PgUp/PgDn" => "scroll"),
        bind!(TaskInfoKey::PageDown, [PGDN], join),
        bind!(TaskInfoKey::Close, [ESC, ch('v'), ch('q')], "esc" => "close"),
    ],
    agents: &[
        bind!(H::OpenDetail, [ENTER, ch('i')], "enter" => "page"),
        bind!(H::ToggleOn, [SPACE], "on/off"),
        bind!(H::RunNow, [ch('p')], "run now"),
        bind!(H::NewSession, [NEW, INS], "ctrl+n" => "claude here"),
        bind!(H::NavDown, [DOWN], "↑/↓" => "agent"),
        bind!(H::NavUp, [UP], join),
    ],
    agent_page: &[
        bind!(H::Activate, [ENTER, ch('o')], "enter" => "open"),
        bind!(H::EditSetting, [F2, ch('e')], "F2" => "type value"),
        bind!(H::DetailDown, [DOWN], "↑/↓" => "select"),
        bind!(H::DetailUp, [UP], join),
        bind!(H::NextSection, [TAB, RIGHT], "←/→" => "section"),
        bind!(H::PrevSection, [BACKTAB, LEFT], join),
        bind!(H::ShowSection(Section::Runs), [ch('1')]),
        bind!(H::ShowSection(Section::Artifacts), [ch('2')]),
        bind!(H::ShowSection(Section::Log), [ch('3')]),
        bind!(H::ShowSection(Section::Settings), [ch('4')]),
        bind!(H::ToggleOn, [SPACE], "on/off"),
        bind!(H::RunNow, [ch('p')], "run now"),
        bind!(H::NewSession, [NEW, INS], "ctrl+n" => "claude here"),
        bind!(H::Reset, [ch('R')], "reset"),
        bind!(H::CloseDetail, [ESC, F1, ch('q')], "esc/F1" => "list"),
    ],
    metrics: &[
        bind!(MetricsKey::Open, [ENTER], "view transcript"),
        bind!(MetricsKey::Down, [DOWN], "↑/↓" => "select"),
        bind!(MetricsKey::Up, [UP], join),
        bind!(MetricsKey::Refresh, [F5, ch('r')], "F5" => "refresh"),
    ],
    settings: &[
        bind!(SettingsKey::Down, [DOWN], "↑/↓" => "move"),
        bind!(SettingsKey::Up, [UP], join),
        bind!(SettingsKey::Next, [RIGHT], "←/→" => "change"),
        bind!(SettingsKey::Prev, [LEFT], join),
        bind!(SettingsKey::Activate, [ENTER, SPACE], "enter" => "edit"),
        bind!(SettingsKey::Reset, [DEL, BKSP, ch('d')], "del" => "default"),
        bind!(SettingsKey::Close, [ESC, ch('q'), ch(',')], "esc" => "close"),
    ],
    popup: &[
        bind!(ScrollKey::Down, [DOWN], "↑/↓" => "scroll"),
        bind!(ScrollKey::Up, [UP], join),
        bind!(ScrollKey::Close, [ESC, ch('q')], "esc" => "close"),
    ],
    live_tail: &[
        bind!(ScrollKey::Down, [DOWN], "↑/↓" => "scroll"),
        bind!(ScrollKey::Up, [UP], join),
        bind!(ScrollKey::Bottom, [END], "bottom"),
        bind!(ScrollKey::Close, [ESC, ch('q')], "esc" => "close"),
    ],
    list_picker: &[
        bind!(ListKey::Down, [DOWN], "↑/↓" => "move"),
        bind!(ListKey::Up, [UP], join),
        bind!(ListKey::Confirm, [ENTER, SPACE], "select"),
        bind!(ListKey::Cancel, [ESC], "cancel"),
    ],
    folders: &[
        bind!(FolderKey::Down, [DOWN], "↑/↓" => "move"),
        bind!(FolderKey::Up, [UP], join),
        bind!(FolderKey::Open, [ENTER, RIGHT], "enter" => "descend"),
        bind!(FolderKey::Parent, [BKSP, LEFT], "bksp" => "parent"),
        bind!(FolderKey::Pick, [SPACE], "pick"),
        bind!(FolderKey::PickCwd, [ch('.')], "pick cwd"),
        bind!(FolderKey::Places, [TAB], "places"),
        bind!(FolderKey::Bookmark, [ch('m')], "bookmark"),
        bind!(FolderKey::GhPublic, [ch('c')], "c/C" => "gh new (pub/priv)"),
        bind!(FolderKey::GhPrivate, [ch('C')], join),
        bind!(FolderKey::Cancel, [ESC, ch('q')], "esc" => "cancel"),
    ],
};
