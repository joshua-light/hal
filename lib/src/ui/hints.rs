//! Hint lines whose wording depends on state, built from the key scheme's
//! tables. The status bar and the popup footers share them, so the two
//! always name the same keys.

use crate::keymap::{self, FolderKey, Hint, ListKey, Scheme};

/// Hints for the agent, respawn and task-kind pickers. `confirm` names what
/// Enter does in this one.
pub(crate) fn list_picker(scheme: Scheme, sep: &str, confirm: &'static str) -> String {
    keymap::hints_with(scheme.maps().list_picker, sep, |key| match key {
        ListKey::Confirm => Hint::As(confirm),
        _ => Hint::Keep,
    })
}

/// What the folder picker is being used for; its keys are the same, but not
/// all of them are worth a hint each time.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Folders {
    /// The bookmark list: every entry is a pick, nothing to descend into.
    Bookmarks,
    /// Browsing for a task's folder.
    Assigning,
    /// Browsing for a new session's folder.
    Browse,
}

pub(crate) fn folders(scheme: Scheme, sep: &str, mode: Folders) -> String {
    use FolderKey as K;
    keymap::hints_with(scheme.maps().folders, sep, |key| match (mode, key) {
        (Folders::Bookmarks, K::Open) => Hint::As("pick"),
        (Folders::Bookmarks, K::Bookmark) => Hint::As("unbookmark"),
        (Folders::Bookmarks, K::Down | K::Cancel) => Hint::Keep,
        (Folders::Bookmarks, _) => Hint::Skip,
        (Folders::Assigning, K::Bookmark | K::GhPublic) => Hint::Skip,
        (Folders::Browse, K::Places) => Hint::Skip,
        _ => Hint::Keep,
    })
}
