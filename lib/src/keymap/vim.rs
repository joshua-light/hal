//! The Vim scheme: `hjkl` and single letters. Rows are in hint order, the
//! verbs worth the most first: the status bar is one row and clips on the
//! right.

use super::*;
use crate::app::{HarnessCommand as H, Section, SessionsCommand as S, Tab, TasksCommand as T};
use crate::tasks::store::TaskPriority;
use BoardKey::Do;

const fn s(cmd: S) -> Command {
    Command::Sessions(cmd)
}

pub(super) static MAPS: Keymaps = Keymaps {
    global: &[
        bind!(GlobalCommand::CycleTab { back: false }, [TAB, ch('K')], "tab" => "next"),
        bind!(GlobalCommand::CycleTab { back: true }, [BACKTAB, ch('J')]),
        bind!(GlobalCommand::OpenSettings, [ch(',')], "settings"),
        bind!(GlobalCommand::Quit, [ch('q')], "quit"),
    ],
    sessions: &[
        bind!(s(S::FocusSelected), [ENTER, ch('f')], "type in/resume"),
        bind!(s(S::OpenSessionFinder), [ch('/')], "find any"),
        bind!(s(S::SpawnAgentHere), [ch('n')], "new"),
        bind!(s(S::ToggleHandoff), [ch('h')], "handoff"),
        bind!(s(S::OpenRespawnPicker), [ch('R')], "respawn"),
        bind!(s(S::OpenAgentPicker), [ch('A')], "default agent"),
        bind!(s(S::OpenModelPicker), [ch('N')], "new+model"),
        bind!(s(S::OpenPlacesPicker), [ch('p')], "new in…"),
        bind!(s(S::OpenDetailPopup), [ch('i')], "info"),
        bind!(s(S::AckSelected), [ch('z')], "ack"),
        bind!(s(S::OpenRenameSession), [ch('r')], "rename"),
        bind!(s(S::OpenTaskLinkPicker), [ch('L')], "link task"),
        bind!(s(S::OpenTerminalHere), [ch('t')], "shell"),
        bind!(s(S::OpenBookmarksPicker), [ch('M')], "bookmarks"),
        // `h` is handoff, so only the arrow moves left.
        bind!(s(S::NavLeft), [LEFT], "←/j/k/l" => "nav"),
        bind!(s(S::NavDown), [DOWN, ch('j')], join),
        bind!(s(S::NavUp), [UP, ch('k')], join),
        bind!(s(S::NavRight), [RIGHT, ch('l')], join),
        bind!(s(S::ToggleLayout), [ch('v')], "layout"),
        bind!(s(S::StageConfirmClose), [ch('x')], "close"),
        bind!(s(S::ToggleShowInactive), [ch('H')], "inactive"),
        // The status bar leads with its own Space chip.
        bind!(s(S::ToggleHold), [SPACE]),
        bind!(
            Command::Global(GlobalCommand::SetTab(Tab::Metrics)),
            [ch('m')]
        ),
    ],
    board: &[
        bind!(Do(T::OpenAddInput), [ch('a'), ch('n')], "add"),
        bind!(Do(T::FocusAgent), [ENTER, ch('f')], "focus agent"),
        bind!(Do(T::OpenTaskInfo), [ch('v')], "info"),
        bind!(Do(T::OpenAssignPicker), [ch('s')], "assign agent"),
        bind!(Do(T::AssignAtHome), [ch('S')], "agent in ~"),
        bind!(Do(T::NavLeft), [LEFT, ch('h')], "h/l" => "col"),
        bind!(Do(T::NavRight), [RIGHT, ch('l')], join),
        bind!(Do(T::NavDown), [DOWN, ch('j')], "j/k" => "task"),
        bind!(Do(T::NavUp), [UP, ch('k')], join),
        bind!(Do(T::MoveTaskLeft), [ch('H')], "H/L" => "move"),
        bind!(Do(T::MoveTaskRight), [ch('L')], join),
        bind!(Do(T::OpenFilter), [ch('/')], "filter"),
        bind!(Do(T::ClearFilter), [ESC]),
        bind!(Do(T::SetPriority(TaskPriority::P1)), [ch('1')], "1-4" => "priority"),
        bind!(Do(T::SetPriority(TaskPriority::P2)), [ch('2')], join),
        bind!(Do(T::SetPriority(TaskPriority::P3)), [ch('3')], join),
        bind!(Do(T::SetPriority(TaskPriority::P4)), [ch('4')], join),
        bind!(Do(T::OpenTags), [ch('t')], "tags"),
        bind!(Do(T::OpenKindPicker), [ch('T')], "kind"),
        bind!(Do(T::OpenRename), [ch('r')], "rename"),
        bind!(
            Do(T::OpenAttachInput { from_info: false }),
            [ch('A')],
            "attach"
        ),
        bind!(BoardKey::PasteNote, [ch('p')], "paste note"),
        bind!(Do(T::DeleteSelected), [ch('x')], "delete"),
        bind!(Do(T::UndoDelete), [ch('u')], "undo"),
        bind!(Do(T::ClearDone), [ch('c')], "clear done"),
        bind!(Do(T::SpaceAction), [SPACE]),
    ],
    task_info: &[
        bind!(TaskInfoKey::Next, [DOWN, ch('j')], "j/k" => "attachment"),
        bind!(TaskInfoKey::Prev, [UP, ch('k')], join),
        bind!(TaskInfoKey::Attach, [ch('a')], "attach"),
        bind!(TaskInfoKey::PasteNote, [ch('p')], "paste note"),
        bind!(TaskInfoKey::CopyPath, [ch('c')], "copy path"),
        bind!(TaskInfoKey::Open, [ch('o')], "open"),
        bind!(TaskInfoKey::Remove, [ch('x')], "remove"),
        bind!(TaskInfoKey::PageUp, [PGUP], "PgUp/PgDn" => "scroll"),
        bind!(TaskInfoKey::PageDown, [PGDN], join),
        bind!(TaskInfoKey::Close, [ESC, ch('v'), ch('q')], "esc/v" => "close"),
    ],
    agents: &[
        bind!(H::OpenDetail, [ch('f'), ENTER, ch('i')], "f/enter" => "page"),
        bind!(H::ToggleOn, [SPACE], "on/off"),
        bind!(H::RunNow, [ch('p')], "run now"),
        bind!(H::NewSession, [ch('n')], "claude here"),
        bind!(H::NavDown, [DOWN, ch('j')], "j/k" => "agent"),
        bind!(H::NavUp, [UP, ch('k')], join),
    ],
    agent_page: &[
        bind!(H::Activate, [ch('f'), ENTER, ch('o')], "f/enter" => "open"),
        bind!(H::EditSetting, [ch('e')], "type value"),
        bind!(H::DetailDown, [DOWN, ch('j')], "j/k" => "select"),
        bind!(H::DetailUp, [UP, ch('k')], join),
        bind!(H::NextSection, [TAB, RIGHT, ch('l')], "h/l" => "section"),
        bind!(H::PrevSection, [BACKTAB, LEFT, ch('h')], join),
        bind!(H::ShowSection(Section::Runs), [ch('1')]),
        bind!(H::ShowSection(Section::Artifacts), [ch('2')]),
        bind!(H::ShowSection(Section::Log), [ch('3')]),
        bind!(H::ShowSection(Section::Settings), [ch('4')]),
        bind!(H::ToggleOn, [SPACE], "on/off"),
        bind!(H::RunNow, [ch('p')], "run now"),
        bind!(H::NewSession, [ch('n')], "claude here"),
        bind!(H::Reset, [ch('R')], "reset"),
        bind!(H::CloseDetail, [ESC, F1, ch('q')], "esc/F1" => "list"),
    ],
    metrics: &[
        bind!(MetricsKey::Open, [ENTER], "view transcript"),
        bind!(MetricsKey::Down, [DOWN, ch('j')], "j/k" => "select"),
        bind!(MetricsKey::Up, [UP, ch('k')], join),
        bind!(MetricsKey::Refresh, [ch('r')], "refresh"),
    ],
    settings: &[
        bind!(SettingsKey::Down, [DOWN, ch('j')], "j/k" => "move"),
        bind!(SettingsKey::Up, [UP, ch('k')], join),
        bind!(SettingsKey::Next, [RIGHT, ch('l')], "h/l" => "change"),
        bind!(SettingsKey::Prev, [LEFT, ch('h')], join),
        bind!(SettingsKey::Activate, [ENTER, SPACE], "enter" => "edit"),
        bind!(SettingsKey::Reset, [ch('d'), DEL, BKSP], "d" => "default"),
        bind!(SettingsKey::Close, [ESC, ch('q'), ch(',')], "esc" => "close"),
    ],
    popup: &[
        bind!(ScrollKey::Down, [DOWN, ch('j')], "j/k" => "scroll"),
        bind!(ScrollKey::Up, [UP, ch('k')], join),
        bind!(ScrollKey::Close, [ESC, ch('q')], "close"),
    ],
    live_tail: &[
        bind!(ScrollKey::Down, [DOWN, ch('j')], "j/k" => "scroll"),
        bind!(ScrollKey::Up, [UP, ch('k')], join),
        bind!(ScrollKey::Bottom, [ch('G')], "bottom"),
        bind!(ScrollKey::Close, [ESC, ch('q')], "close"),
    ],
    list_picker: &[
        bind!(ListKey::Down, [DOWN, ch('j')], "j/k" => "move"),
        bind!(ListKey::Up, [UP, ch('k')], join),
        bind!(ListKey::Confirm, [ENTER, SPACE], "select"),
        bind!(ListKey::Cancel, [ESC], "cancel"),
    ],
    folders: &[
        bind!(FolderKey::Down, [DOWN, ch('j')], "j/k" => "move"),
        bind!(FolderKey::Up, [UP, ch('k')], join),
        bind!(FolderKey::Open, [ENTER, RIGHT, ch('l')], "enter" => "descend"),
        bind!(FolderKey::Parent, [BKSP, LEFT, ch('h')], "bksp" => "parent"),
        bind!(FolderKey::Pick, [SPACE], "pick"),
        bind!(FolderKey::PickCwd, [ch('.')], "pick cwd"),
        bind!(FolderKey::Places, [TAB], "places"),
        bind!(FolderKey::Bookmark, [ch('m')], "bookmark"),
        bind!(FolderKey::GhPublic, [ch('c')], "c/C" => "gh new (pub/priv)"),
        bind!(FolderKey::GhPrivate, [ch('C')], join),
        bind!(FolderKey::Cancel, [ESC, ch('q')], "esc" => "cancel"),
    ],
};
