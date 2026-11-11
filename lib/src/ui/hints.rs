//! Hint lines whose wording depends on state, built from the key scheme's
//! tables. The status bar and the popup footers share them, so the two
//! always name the same keys.

use crate::keymap::{self, FilterKey, FolderKey, Hint, ListKey, Scheme, TextKey};

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

/// Hints for a one-line text box. `submit` and `cancel` say what those keys
/// do in this box, `switch` what its second field or mode is, if it has one.
pub(crate) fn text(
    scheme: Scheme,
    sep: &str,
    submit: &'static str,
    cancel: &'static str,
    switch: Option<&'static str>,
) -> String {
    keymap::hints_with(scheme.maps().text_input, sep, |key| match key {
        TextKey::Submit => Hint::As(submit),
        TextKey::Cancel => Hint::As(cancel),
        TextKey::Switch => switch.map_or(Hint::Skip, Hint::As),
        TextKey::Erase => Hint::Keep,
    })
}

/// Hints for a type-to-filter picker, named like [`text`]'s.
pub(crate) fn filter(
    scheme: Scheme,
    sep: &str,
    confirm: &'static str,
    switch: Option<&'static str>,
) -> String {
    let keys = keymap::hints_with(scheme.maps().filter_picker, sep, |key| match key {
        FilterKey::Confirm => Hint::As(confirm),
        FilterKey::Switch => switch.map_or(Hint::Skip, Hint::As),
        _ => Hint::Keep,
    });
    format!("type:filter{}{}", sep, keys)
}
