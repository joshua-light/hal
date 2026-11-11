//! Tables every scheme shares: the places where printable keys are text (or
//! the pane's), so neither scheme has letters to spend there. Unbound
//! printable keys are typed; `[keymap]` can still move the rest.

use super::*;

/// Type-to-filter pickers: model, task link, and the folder picker's
/// places list. The Ctrl chords move without taking a letter from the
/// filter.
pub(super) const FILTER_PICKER: Keymap<FilterKey> = &[
    bind!(FilterKey::Down, [DOWN, ctrl(ch('j')), ctrl(ch('n'))], "↑/↓" => "move"),
    bind!(FilterKey::Up, [UP, ctrl(ch('k')), ctrl(ch('p'))], join),
    bind!(FilterKey::Switch, [TAB], "switch"),
    bind!(FilterKey::Confirm, [ENTER, SPACE], "select"),
    bind!(FilterKey::Cancel, [ESC], "cancel"),
    bind!(FilterKey::Erase, [BKSP]),
];

/// The session finder. Space is not Confirm here: titles and first
/// messages are prose, so it belongs to the filter.
pub(super) const SESSION_FINDER: Keymap<FilterKey> = &[
    bind!(FilterKey::Down, [DOWN, ctrl(ch('j')), ctrl(ch('n'))], "↑/↓" => "move"),
    bind!(FilterKey::Up, [UP, ctrl(ch('k')), ctrl(ch('p'))], join),
    bind!(FilterKey::Confirm, [ENTER], "open"),
    bind!(FilterKey::Cancel, [ESC], "cancel"),
    bind!(FilterKey::Erase, [BKSP]),
];

/// One-line text boxes: names, tags, notes, filters, typed settings.
pub(super) const TEXT_INPUT: Keymap<TextKey> = &[
    bind!(TextKey::Switch, [TAB], "switch"),
    bind!(TextKey::Submit, [ENTER], "apply"),
    bind!(TextKey::Cancel, [ESC], "cancel"),
    bind!(TextKey::Erase, [BKSP]),
];

pub(super) const CONFIRM_CLOSE: Keymap<ConfirmKey> = &[
    bind!(ConfirmKey::Yes, [ch('y'), ch('Y')], "y" => "confirm"),
    bind!(ConfirmKey::No, [ch('n'), ch('N'), ESC, ch('q')], "n/esc" => "cancel"),
];

/// The embedded pane keeps these; every other key goes to the session.
pub(super) const PANE: Keymap<PaneKey> = &[
    bind!(PaneKey::Release, [F1], "detach"),
    // Plain Ctrl+V is the session's own paste.
    bind!(PaneKey::Paste, [ctrl(shift(ch('v')))]),
];

/// Every view but the pane, text boxes included.
pub(super) const EVERYWHERE: Keymap<AnywhereKey> = &[bind!(AnywhereKey::Redraw, [ctrl(ch('l'))])];
