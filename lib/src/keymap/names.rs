//! The name each bound action goes by in `[keymap.<place>]`. One name per
//! action, shared by every scheme, so an override written under one scheme
//! still means the same thing under the other.

use super::*;
use crate::app::{Section, SessionsCommand, Tab};
use crate::tasks::store::TaskPriority;

/// Something a key can be bound to.
pub trait Action: Copy + PartialEq + 'static {
    /// The name `[keymap.<place>]` knows it by; empty for a value no table
    /// binds.
    fn name(self) -> &'static str;
}

impl Action for GlobalCommand {
    fn name(self) -> &'static str {
        match self {
            GlobalCommand::CycleTab { back: false } => "next_tab",
            GlobalCommand::CycleTab { back: true } => "prev_tab",
            GlobalCommand::OpenSettings => "settings",
            GlobalCommand::Quit => "quit",
            GlobalCommand::SetTab(Tab::Metrics) => "metrics_tab",
            GlobalCommand::SetTab(_) => "",
        }
    }
}

impl Action for Command {
    fn name(self) -> &'static str {
        use SessionsCommand as S;
        match self {
            Command::Global(cmd) => cmd.name(),
            Command::Harness(cmd) => cmd.name(),
            Command::Tasks(cmd) => BoardKey::Do(cmd).name(),
            Command::Sessions(cmd) => match cmd {
                S::FocusSelected => "focus",
                S::OpenSessionFinder => "find",
                S::SpawnAgentHere => "new",
                S::ToggleHandoff => "handoff",
                S::OpenRespawnPicker => "respawn",
                S::OpenAgentPicker => "default_agent",
                S::OpenModelPicker => "new_with_model",
                S::OpenPlacesPicker => "new_in",
                S::OpenDetailPopup => "info",
                S::AckSelected => "ack",
                S::OpenRenameSession => "rename",
                S::OpenTaskLinkPicker => "link_task",
                S::OpenTerminalHere => "shell",
                S::OpenBookmarksPicker => "bookmarks",
                S::NavLeft => "left",
                S::NavDown => "down",
                S::NavUp => "up",
                S::NavRight => "right",
                S::ToggleLayout => "layout",
                S::StageConfirmClose => "close",
                S::ToggleShowInactive => "inactive",
                S::ToggleHold => "hold",
                _ => "",
            },
        }
    }
}

impl Action for BoardKey {
    fn name(self) -> &'static str {
        use TasksCommand as T;
        let BoardKey::Do(cmd) = self else {
            return "paste_note";
        };
        match cmd {
            T::OpenAddInput => "add",
            T::FocusAgent => "focus_agent",
            T::OpenTaskInfo => "info",
            T::OpenAssignPicker => "assign",
            T::AssignAtHome => "assign_home",
            T::NavLeft => "left",
            T::NavRight => "right",
            T::NavDown => "down",
            T::NavUp => "up",
            T::MoveTaskLeft => "move_left",
            T::MoveTaskRight => "move_right",
            T::OpenFilter => "filter",
            T::ClearFilter => "clear_filter",
            T::SetPriority(TaskPriority::P1) => "priority_1",
            T::SetPriority(TaskPriority::P2) => "priority_2",
            T::SetPriority(TaskPriority::P3) => "priority_3",
            T::SetPriority(TaskPriority::P4) => "priority_4",
            T::OpenTags => "tags",
            T::OpenKindPicker => "kind",
            T::OpenRename => "rename",
            T::OpenAttachInput { from_info: false } => "attach",
            T::DeleteSelected => "delete",
            T::UndoDelete => "undo",
            T::ClearDone => "clear_done",
            T::SpaceAction => "proceed",
            _ => "",
        }
    }
}

impl Action for HarnessCommand {
    fn name(self) -> &'static str {
        use HarnessCommand as H;
        match self {
            H::OpenDetail => "page",
            H::ToggleOn => "toggle",
            H::RunNow => "run_now",
            H::NewSession => "new_session",
            H::NavDown | H::DetailDown => "down",
            H::NavUp | H::DetailUp => "up",
            H::Activate => "open",
            H::EditSetting => "edit",
            H::NextSection => "next_section",
            H::PrevSection => "prev_section",
            H::ShowSection(Section::Runs) => "runs",
            H::ShowSection(Section::Artifacts) => "artifacts",
            H::ShowSection(Section::Log) => "log",
            H::ShowSection(Section::Settings) => "settings",
            H::Reset => "reset",
            H::CloseDetail => "close",
            _ => "",
        }
    }
}

impl Action for TaskInfoKey {
    fn name(self) -> &'static str {
        match self {
            TaskInfoKey::Next => "next",
            TaskInfoKey::Prev => "prev",
            TaskInfoKey::PageUp => "page_up",
            TaskInfoKey::PageDown => "page_down",
            TaskInfoKey::Attach => "attach",
            TaskInfoKey::PasteNote => "paste_note",
            TaskInfoKey::CopyPath => "copy_path",
            TaskInfoKey::Open => "open",
            TaskInfoKey::Remove => "remove",
            TaskInfoKey::Close => "close",
        }
    }
}

impl Action for MetricsKey {
    fn name(self) -> &'static str {
        match self {
            MetricsKey::Down => "down",
            MetricsKey::Up => "up",
            MetricsKey::Open => "open",
            MetricsKey::Refresh => "refresh",
        }
    }
}

impl Action for ScrollKey {
    fn name(self) -> &'static str {
        match self {
            ScrollKey::Down => "down",
            ScrollKey::Up => "up",
            ScrollKey::Bottom => "bottom",
            ScrollKey::Close => "close",
        }
    }
}

impl Action for ListKey {
    fn name(self) -> &'static str {
        match self {
            ListKey::Down => "down",
            ListKey::Up => "up",
            ListKey::Confirm => "confirm",
            ListKey::Cancel => "cancel",
        }
    }
}

impl Action for FolderKey {
    fn name(self) -> &'static str {
        match self {
            FolderKey::Down => "down",
            FolderKey::Up => "up",
            FolderKey::Open => "open",
            FolderKey::Parent => "parent",
            FolderKey::Pick => "pick",
            FolderKey::PickCwd => "pick_cwd",
            FolderKey::Bookmark => "bookmark",
            FolderKey::GhPublic => "gh_public",
            FolderKey::GhPrivate => "gh_private",
            FolderKey::Places => "places",
            FolderKey::Cancel => "cancel",
        }
    }
}

impl Action for SettingsKey {
    fn name(self) -> &'static str {
        match self {
            SettingsKey::Down => "down",
            SettingsKey::Up => "up",
            SettingsKey::Next => "next",
            SettingsKey::Prev => "prev",
            SettingsKey::Activate => "edit",
            SettingsKey::Reset => "reset",
            SettingsKey::Close => "close",
        }
    }
}

impl Action for FilterKey {
    fn name(self) -> &'static str {
        match self {
            FilterKey::Down => "down",
            FilterKey::Up => "up",
            FilterKey::Confirm => "confirm",
            FilterKey::Cancel => "cancel",
            FilterKey::Erase => "erase",
            FilterKey::Switch => "switch",
        }
    }
}

impl Action for TextKey {
    fn name(self) -> &'static str {
        match self {
            TextKey::Submit => "submit",
            TextKey::Cancel => "cancel",
            TextKey::Erase => "erase",
            TextKey::Switch => "switch",
        }
    }
}

impl Action for ConfirmKey {
    fn name(self) -> &'static str {
        match self {
            ConfirmKey::Yes => "yes",
            ConfirmKey::No => "no",
        }
    }
}

impl Action for PaneKey {
    fn name(self) -> &'static str {
        match self {
            PaneKey::Release => "release",
            PaneKey::Paste => "paste",
        }
    }
}

impl Action for AnywhereKey {
    fn name(self) -> &'static str {
        match self {
            AnywhereKey::Redraw => "redraw",
        }
    }
}
