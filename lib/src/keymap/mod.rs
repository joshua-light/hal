//! Key schemes. Each [`Scheme`] is a set of tables, one per place that takes
//! keys, mapping key presses onto what they do there. `bin/` resolves key
//! events against the tables; the status bar and popup footers print their
//! hints from the same rows, so a hint cannot name a key that is not bound.
//!
//! Text inputs, type-to-filter pickers and the pane have tables too
//! ([`shared`]): a key they do not bind is typed. No handler names a key
//! itself, so `[keymap]` reaches every one; a test in `bin/` holds that.
//!
//! - `ide`: arrows and the usual editor chords. The default.
//! - `vim`: `hjkl` and single letters.
//! - `shared`: the places both schemes bind alike.
//! - `layout`: Cyrillic letters read as the key they sit on.
//! - `names`: what each action is called in `config.toml`.
//! - `custom`: `[keymap]`, the user's keys laid over a scheme.

use crate::app::{Command, GlobalCommand, HarnessCommand, TasksCommand};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::Deserialize;

mod custom;
mod ide;
mod layout;
mod names;
mod shared;
mod vim;

pub use custom::{dump, problems, KeyList, Overrides};
pub use layout::{latin, to_latin};
pub use names::Action;

/// Which set of keys the hub answers to: `[ui].keys`, changed in the
/// settings window.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Scheme {
    #[default]
    Ide,
    Vim,
}

impl Scheme {
    pub const ALL: [Scheme; 2] = [Scheme::Ide, Scheme::Vim];

    /// The value `[ui].keys` holds for this scheme.
    pub fn config_value(self) -> &'static str {
        match self {
            Scheme::Ide => "ide",
            Scheme::Vim => "vim",
        }
    }

    /// The tables in force: this scheme with the config's `[keymap]`
    /// applied.
    pub fn maps(self) -> &'static Keymaps {
        custom::maps(self)
    }

    /// The tables as shipped.
    pub fn defaults(self) -> &'static Keymaps {
        match self {
            Scheme::Ide => &ide::MAPS,
            Scheme::Vim => &vim::MAPS,
        }
    }
}

/// One key press a binding answers to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Key {
    code: KeyCode,
    mods: KeyModifiers,
}

const NONE: KeyModifiers = KeyModifiers::NONE;

pub(crate) const fn code(code: KeyCode) -> Key {
    Key { code, mods: NONE }
}

/// A printable key. Uppercase letters carry their own Shift.
pub(crate) const fn ch(c: char) -> Key {
    code(KeyCode::Char(c))
}

pub(crate) const fn ctrl(key: Key) -> Key {
    Key {
        code: key.code,
        mods: key.mods.union(KeyModifiers::CONTROL),
    }
}

pub(crate) const fn shift(key: Key) -> Key {
    Key {
        code: key.code,
        mods: key.mods.union(KeyModifiers::SHIFT),
    }
}

pub(crate) const UP: Key = code(KeyCode::Up);
pub(crate) const DOWN: Key = code(KeyCode::Down);
pub(crate) const LEFT: Key = code(KeyCode::Left);
pub(crate) const RIGHT: Key = code(KeyCode::Right);
pub(crate) const ENTER: Key = code(KeyCode::Enter);
pub(crate) const ESC: Key = code(KeyCode::Esc);
pub(crate) const TAB: Key = code(KeyCode::Tab);
pub(crate) const BACKTAB: Key = code(KeyCode::BackTab);
pub(crate) const SPACE: Key = ch(' ');
pub(crate) const BKSP: Key = code(KeyCode::Backspace);
pub(crate) const DEL: Key = code(KeyCode::Delete);
pub(crate) const INS: Key = code(KeyCode::Insert);
pub(crate) const PGUP: Key = code(KeyCode::PageUp);
pub(crate) const PGDN: Key = code(KeyCode::PageDown);
pub(crate) const END: Key = code(KeyCode::End);
pub(crate) const F1: Key = code(KeyCode::F(1));
pub(crate) const F2: Key = code(KeyCode::F(2));
pub(crate) const F5: Key = code(KeyCode::F(5));

impl Key {
    /// Whether `ev` is this key.
    ///
    /// Ctrl and Alt must match exactly, so `t` and `Ctrl+T` are different
    /// keys. Super counts as Ctrl: a terminal that forwards Cmd lands `⌘P`
    /// on `Ctrl+P`, the chord a Mac user reaches for. Shift is ignored on printable keys, whose character already
    /// says it, and on BackTab, which is Shift+Tab by definition. A Ctrl
    /// chord ignores letter case: only the kitty protocol tells `Ctrl+T`
    /// from `Ctrl+Shift+T`, so `Ctrl+T` answers to both and
    /// `Ctrl+Shift+T` only to a terminal that reports the Shift.
    pub fn matches(&self, ev: &KeyEvent) -> bool {
        let chord = KeyModifiers::CONTROL | KeyModifiers::ALT;
        let mut pressed = ev.modifiers;
        if pressed.contains(KeyModifiers::SUPER) {
            pressed |= KeyModifiers::CONTROL;
        }
        if pressed & chord != self.mods & chord {
            return false;
        }
        match (self.code, ev.code) {
            (KeyCode::Char(a), KeyCode::Char(b)) => {
                if self.mods.contains(KeyModifiers::CONTROL) {
                    let shifted = ev.modifiers.contains(KeyModifiers::SHIFT) || b.is_uppercase();
                    a.eq_ignore_ascii_case(&b)
                        && (shifted || !self.mods.contains(KeyModifiers::SHIFT))
                } else {
                    a == b
                }
            }
            (KeyCode::BackTab, KeyCode::BackTab) => true,
            (a, b) => {
                a == b
                    && ev.modifiers.contains(KeyModifiers::SHIFT)
                        == self.mods.contains(KeyModifiers::SHIFT)
            }
        }
    }

    /// The key as hints print it: `enter`, `ctrl+t`, `shift+←`.
    pub fn label(&self) -> String {
        let name = match self.code {
            KeyCode::Char(' ') => "space".to_string(),
            KeyCode::Char(c) => c.to_string(),
            KeyCode::Enter => "enter".into(),
            KeyCode::Esc => "esc".into(),
            KeyCode::Tab => "tab".into(),
            KeyCode::BackTab => "shift+tab".into(),
            KeyCode::Backspace => "bksp".into(),
            KeyCode::Delete => "del".into(),
            KeyCode::Insert => "ins".into(),
            KeyCode::Up => "↑".into(),
            KeyCode::Down => "↓".into(),
            KeyCode::Left => "←".into(),
            KeyCode::Right => "→".into(),
            KeyCode::PageUp => "PgUp".into(),
            KeyCode::PageDown => "PgDn".into(),
            KeyCode::Home => "home".into(),
            KeyCode::End => "end".into(),
            KeyCode::F(n) => format!("F{}", n),
            other => format!("{:?}", other).to_lowercase(),
        };
        let mut out = String::new();
        for (flag, prefix) in [
            (KeyModifiers::CONTROL, "ctrl+"),
            (KeyModifiers::ALT, "alt+"),
            (KeyModifiers::SHIFT, "shift+"),
        ] {
            if self.mods.contains(flag) {
                out.push_str(prefix);
            }
        }
        out + &name
    }

    fn event(&self) -> KeyEvent {
        KeyEvent::new(self.code, self.mods)
    }

    /// Whether one press would answer to both keys.
    pub(crate) fn same_press(&self, other: &Key) -> bool {
        self.matches(&other.event()) || other.matches(&self.event())
    }

    /// A key as `[keymap]` spells it: what [`Key::label`] prints, in any
    /// case, plus the long names (`backspace`, `up`, `cmd+p`). A lone
    /// character is itself, so `P` is Shift+P.
    pub fn parse(text: &str) -> Result<Key, String> {
        let trimmed = text.trim();
        // `+` is both the separator and a key.
        let (mods_text, name) = match trimmed.strip_suffix('+') {
            Some(rest) if rest.is_empty() || rest.ends_with('+') => {
                (rest.strip_suffix('+').unwrap_or(""), "+")
            }
            _ => trimmed.rsplit_once('+').unwrap_or(("", trimmed)),
        };
        let mut mods = NONE;
        for word in mods_text.split('+').filter(|w| !w.is_empty()) {
            mods |= match word.trim().to_lowercase().as_str() {
                "ctrl" | "control" | "cmd" | "command" | "super" => KeyModifiers::CONTROL,
                "alt" | "opt" | "option" => KeyModifiers::ALT,
                "shift" => KeyModifiers::SHIFT,
                other => return Err(format!("{}: no such modifier: {}", text, other)),
            };
        }
        let mut code = match name.trim().to_lowercase().as_str() {
            "enter" | "return" => KeyCode::Enter,
            "esc" | "escape" => KeyCode::Esc,
            "tab" => KeyCode::Tab,
            "backtab" => KeyCode::BackTab,
            "space" => KeyCode::Char(' '),
            "bksp" | "backspace" => KeyCode::Backspace,
            "del" | "delete" => KeyCode::Delete,
            "ins" | "insert" => KeyCode::Insert,
            "up" | "↑" => KeyCode::Up,
            "down" | "↓" => KeyCode::Down,
            "left" | "←" => KeyCode::Left,
            "right" | "→" => KeyCode::Right,
            "pgup" | "pageup" => KeyCode::PageUp,
            "pgdn" | "pagedown" => KeyCode::PageDown,
            "home" => KeyCode::Home,
            "end" => KeyCode::End,
            lower => {
                let mut chars = name.trim().chars();
                let f_key = lower.strip_prefix('f').and_then(|n| n.parse::<u8>().ok());
                match (chars.next(), chars.next(), f_key) {
                    (Some(c), None, _) => KeyCode::Char(c),
                    (_, _, Some(n @ 1..=12)) => KeyCode::F(n),
                    _ => return Err(format!("{}: no such key: {}", text, name.trim())),
                }
            }
        };
        // Shift is part of what these keys are, not a modifier on them.
        if mods.contains(KeyModifiers::SHIFT) {
            match code {
                KeyCode::Tab | KeyCode::BackTab => code = KeyCode::BackTab,
                // Ctrl+Shift+letter keeps its Shift: see `matches`.
                KeyCode::Char(c) if c.is_alphabetic() && mods.contains(KeyModifiers::CONTROL) => {}
                KeyCode::Char(c) if c.is_alphabetic() => {
                    code = KeyCode::Char(c.to_uppercase().next().unwrap_or(c));
                }
                KeyCode::Char(' ') => {}
                KeyCode::Char(c) => {
                    return Err(format!(
                        "{}: write the character Shift+{} types instead",
                        text, c
                    ));
                }
                _ => {}
            }
            let ctrl_letter = mods.contains(KeyModifiers::CONTROL)
                && matches!(code, KeyCode::Char(c) if c.is_alphabetic());
            if matches!(code, KeyCode::BackTab | KeyCode::Char(_)) && !ctrl_letter {
                mods -= KeyModifiers::SHIFT;
            }
        }
        // A Ctrl chord has no case; lowercase is how hints print it.
        if let (true, KeyCode::Char(c)) = (mods.contains(KeyModifiers::CONTROL), code) {
            code = KeyCode::Char(c.to_lowercase().next().unwrap_or(c));
        }
        Ok(Key { code, mods })
    }
}

/// How a binding shows in a hint line.
#[derive(Clone, Copy, Debug)]
pub enum Bar {
    /// Not shown: a key the line has no room for.
    Hide,
    /// Not shown on its own: the hint of the nearest shown row above names
    /// this binding's key too (`j/k:task`).
    Join,
    /// Every key of the binding, then the label: `a/n:add`.
    Label(&'static str),
    /// The keys spelled by hand, then the label: a pair of bindings under
    /// one hint (`j/k:task`), or the one key worth showing out of several.
    Text(&'static str, &'static str),
}

/// Keys and what they do in one place.
#[derive(Clone, Copy, Debug)]
pub struct Binding<A: 'static> {
    pub keys: &'static [Key],
    pub action: A,
    pub bar: Bar,
}

pub type Keymap<A> = &'static [Binding<A>];

/// `bind!(action, [keys], "label")` shows every key; `"keys" => "label"`
/// spells the keys by hand; no label hides the binding from hint lines;
/// `join` hides it under the hand-spelled hint above.
macro_rules! bind {
    ($action:expr, [$($key:expr),+ $(,)?], join) => {
        $crate::keymap::Binding { keys: &[$($key),+], action: $action, bar: $crate::keymap::Bar::Join }
    };
    ($action:expr, [$($key:expr),+ $(,)?]) => {
        $crate::keymap::Binding { keys: &[$($key),+], action: $action, bar: $crate::keymap::Bar::Hide }
    };
    ($action:expr, [$($key:expr),+ $(,)?], $keys:literal => $label:literal) => {
        $crate::keymap::Binding {
            keys: &[$($key),+],
            action: $action,
            bar: $crate::keymap::Bar::Text($keys, $label),
        }
    };
    ($action:expr, [$($key:expr),+ $(,)?], $label:literal) => {
        $crate::keymap::Binding {
            keys: &[$($key),+],
            action: $action,
            bar: $crate::keymap::Bar::Label($label),
        }
    };
}
pub(crate) use bind;

/// What `key` does in `map`, if anything.
pub fn resolve<A: Copy>(map: Keymap<A>, key: &KeyEvent) -> Option<A> {
    map.iter()
        .find(|b| b.keys.iter().any(|k| k.matches(key)))
        .map(|b| b.action)
}

/// A caller's say over one binding's hint, for places whose keys mean
/// different things by state.
pub enum Hint {
    Keep,
    As(&'static str),
    Skip,
}

/// The hint line for `map`, in table order: `a/n:add  enter/f:focus agent`.
pub fn hints<A: Copy>(map: Keymap<A>) -> String {
    hints_with(map, "  ", |_| Hint::Keep)
}

/// [`hints`] with the separator chosen and each shown binding passed through
/// `relabel`.
pub fn hints_with<A: Copy>(map: Keymap<A>, sep: &str, relabel: impl Fn(A) -> Hint) -> String {
    let mut parts: Vec<String> = Vec::new();
    for b in map {
        // An override can leave a binding with no key: nothing to hint.
        if b.keys.is_empty() {
            continue;
        }
        let (keys, label) = match b.bar {
            Bar::Hide | Bar::Join => continue,
            Bar::Label(label) => (key_names(b.keys), label),
            Bar::Text(keys, label) => (keys.to_string(), label),
        };
        let label = match relabel(b.action) {
            Hint::Keep => label,
            Hint::As(label) => label,
            Hint::Skip => continue,
        };
        parts.push(format!("{}:{}", keys, label));
    }
    parts.join(sep)
}

fn key_names(keys: &[Key]) -> String {
    keys.iter().map(Key::label).collect::<Vec<_>>().join("/")
}

/// The first key bound to `action`, for messages that tell the user what to
/// press.
pub fn key_for<A: Copy + PartialEq>(map: Keymap<A>, action: A) -> String {
    map.iter()
        .find(|b| b.action == action)
        .and_then(|b| b.keys.first())
        .map(Key::label)
        .unwrap_or_default()
}

/// A key on the Tasks board: a command, or the clipboard paste `bin/` runs
/// itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoardKey {
    Do(TasksCommand),
    PasteNote,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskInfoKey {
    Next,
    Prev,
    PageUp,
    PageDown,
    Attach,
    PasteNote,
    CopyPath,
    Open,
    Remove,
    Close,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MetricsKey {
    Down,
    Up,
    Open,
    Refresh,
}

/// Session info and the transcript.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollKey {
    Down,
    Up,
    Bottom,
    Close,
}

/// The agent, respawn and task-kind pickers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListKey {
    Down,
    Up,
    Confirm,
    Cancel,
}

/// The folder picker's Browse and Bookmarks modes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FolderKey {
    Down,
    Up,
    Open,
    Parent,
    Pick,
    PickCwd,
    Bookmark,
    GhPublic,
    GhPrivate,
    Places,
    Cancel,
}

/// A key in the settings window, outside its colour box.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsKey {
    Down,
    Up,
    Next,
    Prev,
    /// Open a colour for typing; step any other row on.
    Activate,
    /// Drop the row from the file, back to its default.
    Reset,
    Close,
}

/// A type-to-filter picker: keys it does not bind go into the filter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FilterKey {
    Down,
    Up,
    Confirm,
    Cancel,
    /// Drop the filter's last character.
    Erase,
    /// The picker's other list: the next agent, or folders for places.
    Switch,
}

/// A one-line text box: keys it does not bind are typed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextKey {
    Submit,
    Cancel,
    /// Drop the last character.
    Erase,
    /// The box's other field or mode, where it has one.
    Switch,
}

/// The close-session confirmation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfirmKey {
    Yes,
    No,
}

/// The keys the hub keeps while the embedded pane has the keyboard.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaneKey {
    /// Give the keyboard back to the hub.
    Release,
    /// Paste the host clipboard into the pane.
    Paste,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnywhereKey {
    /// Repaint the whole screen.
    Redraw,
}

/// One scheme's tables.
///
/// [`PLACES`] lists the fields by name: the `<place>` of `[keymap.<place>]`.
pub struct Keymaps {
    /// Every tab, after the tab's own table.
    pub global: Keymap<GlobalCommand>,
    pub sessions: Keymap<Command>,
    pub board: Keymap<BoardKey>,
    pub task_info: Keymap<TaskInfoKey>,
    pub agents: Keymap<HarnessCommand>,
    pub agent_page: Keymap<HarnessCommand>,
    pub metrics: Keymap<MetricsKey>,
    pub settings: Keymap<SettingsKey>,
    pub popup: Keymap<ScrollKey>,
    pub live_tail: Keymap<ScrollKey>,
    pub list_picker: Keymap<ListKey>,
    pub folders: Keymap<FolderKey>,
    pub filter_picker: Keymap<FilterKey>,
    pub session_finder: Keymap<FilterKey>,
    pub text_input: Keymap<TextKey>,
    pub confirm_close: Keymap<ConfirmKey>,
    pub pane: Keymap<PaneKey>,
    pub everywhere: Keymap<AnywhereKey>,
}

/// Every key bound to `action`, as hints print them: `enter/space`.
pub fn keys_for<A: Copy + PartialEq>(map: Keymap<A>, action: A) -> String {
    map.iter()
        .find(|b| b.action == action)
        .map(|b| key_names(b.keys))
        .unwrap_or_default()
}

pub(crate) const PLACES: [&str; 18] = [
    "global",
    "sessions",
    "board",
    "task_info",
    "agents",
    "agent_page",
    "metrics",
    "settings",
    "popup",
    "live_tail",
    "list_picker",
    "folders",
    "filter_picker",
    "session_finder",
    "text_input",
    "confirm_close",
    "pane",
    "everywhere",
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::SessionsCommand;

    fn names_of<A: Action>(map: Keymap<A>) -> Vec<&'static str> {
        map.iter().map(|b| b.action.name()).collect()
    }

    // `[keymap]` finds a binding by its name, so each needs one of its own.
    #[test]
    fn every_binding_has_a_name_of_its_own_in_its_place() {
        for scheme in Scheme::ALL {
            let m = scheme.defaults();
            let places = [
                names_of(m.global),
                names_of(m.sessions),
                names_of(m.board),
                names_of(m.task_info),
                names_of(m.agents),
                names_of(m.agent_page),
                names_of(m.metrics),
                names_of(m.settings),
                names_of(m.popup),
                names_of(m.live_tail),
                names_of(m.list_picker),
                names_of(m.folders),
                names_of(m.filter_picker),
                names_of(m.session_finder),
                names_of(m.text_input),
                names_of(m.confirm_close),
                names_of(m.pane),
                names_of(m.everywhere),
            ];
            for (place, names) in PLACES.iter().zip(places) {
                for (i, name) in names.iter().enumerate() {
                    assert!(
                        !name.is_empty(),
                        "{:?} {}: row {} has no name",
                        scheme,
                        place,
                        i
                    );
                    assert!(
                        !names[..i].contains(name),
                        "{:?} {}: {} names two rows",
                        scheme,
                        place,
                        name
                    );
                }
            }
        }
    }

    #[test]
    fn a_key_parses_back_from_its_label() {
        for scheme in Scheme::ALL {
            let m = scheme.defaults();
            let keys = [
                keys_of(m.global),
                keys_of(m.sessions),
                keys_of(m.board),
                keys_of(m.filter_picker),
                keys_of(m.pane),
            ];
            for key in keys.iter().flatten().flat_map(|ks| ks.iter()) {
                assert_eq!(Key::parse(&key.label()), Ok(*key), "{}", key.label());
            }
        }
    }

    #[test]
    fn keys_parse_as_written() {
        assert_eq!(Key::parse("Ctrl+P"), Ok(ctrl(ch('p'))));
        assert_eq!(Key::parse("cmd+p"), Ok(ctrl(ch('p'))));
        assert_eq!(Key::parse("shift+p"), Ok(ch('P')));
        assert_eq!(Key::parse("P"), Ok(ch('P')));
        assert_eq!(Key::parse("shift+left"), Ok(shift(LEFT)));
        assert_eq!(Key::parse("Backspace"), Ok(BKSP));
        assert_eq!(Key::parse("f5"), Ok(F5));
        assert_eq!(Key::parse("f"), Ok(ch('f')));
        assert_eq!(Key::parse("+"), Ok(ch('+')));
        assert_eq!(Key::parse("ctrl++"), Ok(ctrl(ch('+'))));
        // Only a Ctrl chord keeps Shift on a letter.
        let paste = ctrl(shift(ch('v')));
        assert_eq!(Key::parse("ctrl+shift+v"), Ok(paste));
        assert_eq!(paste.label(), "ctrl+shift+v");
        assert!(!paste.matches(&KeyEvent::new(KeyCode::Char('v'), KeyModifiers::CONTROL)));
        assert!(paste.matches(&KeyEvent::new(
            KeyCode::Char('V'),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT
        )));
        assert!(Key::parse("f13").is_err());
        assert!(Key::parse("shift+1").is_err());
        assert!(Key::parse("").is_err());
    }

    fn press(c: char) -> KeyEvent {
        KeyEvent::from(KeyCode::Char(c))
    }

    /// Every key of `maps` (each inner slice is one place's tables, checked
    /// together) must resolve to the binding that lists it.
    fn assert_no_key_bound_twice(scheme: Scheme, place: &str, keys: &[&[Key]]) {
        let all: Vec<(usize, Key)> = keys
            .iter()
            .enumerate()
            .flat_map(|(i, ks)| ks.iter().map(move |k| (i, *k)))
            .collect();
        for (i, key) in &all {
            let ev = key.event();
            let hits: Vec<usize> = all
                .iter()
                .filter(|(_, other)| other.matches(&ev))
                .map(|(j, _)| *j)
                .collect();
            assert!(
                hits.iter().all(|j| j == i),
                "{:?} {}: {} is bound twice",
                scheme,
                place,
                key.label()
            );
        }
    }

    fn keys_of<A>(map: Keymap<A>) -> Vec<&'static [Key]> {
        map.iter().map(|b| b.keys).collect()
    }

    // A tab's table runs ahead of the global one, so a key in both would
    // silently lose its global meaning on that tab.
    #[test]
    fn no_key_is_bound_twice_in_one_place() {
        for scheme in Scheme::ALL {
            let m = scheme.defaults();
            let with_global = |mut keys: Vec<&'static [Key]>| {
                keys.extend(keys_of(m.global));
                keys
            };
            let places: Vec<(&str, Vec<&[Key]>)> = vec![
                ("sessions", with_global(keys_of(m.sessions))),
                ("board", with_global(keys_of(m.board))),
                ("agents", with_global(keys_of(m.agents))),
                ("metrics", with_global(keys_of(m.metrics))),
                ("settings", keys_of(m.settings)),
                ("task info", keys_of(m.task_info)),
                ("agent page", keys_of(m.agent_page)),
                ("popup", keys_of(m.popup)),
                ("live tail", keys_of(m.live_tail)),
                ("list picker", keys_of(m.list_picker)),
                ("folders", keys_of(m.folders)),
                ("filter picker", keys_of(m.filter_picker)),
                ("session finder", keys_of(m.session_finder)),
                ("text input", keys_of(m.text_input)),
                ("confirm close", keys_of(m.confirm_close)),
                ("pane", keys_of(m.pane)),
            ];
            for (place, keys) in places {
                assert_no_key_bound_twice(scheme, place, &keys);
            }
        }
    }

    #[test]
    fn a_ctrl_chord_is_not_its_letter() {
        let t = ch('t');
        let ctrl_t = ctrl(ch('t'));
        let ev = KeyEvent::new(KeyCode::Char('t'), KeyModifiers::CONTROL);
        assert!(ctrl_t.matches(&ev));
        assert!(!t.matches(&ev));
        assert!(t.matches(&press('t')));
        assert!(!ctrl_t.matches(&press('t')));
        // kitty reports the shifted letter; legacy terminals cannot.
        let shifted = KeyEvent::new(
            KeyCode::Char('T'),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        );
        assert!(ctrl_t.matches(&shifted));
    }

    #[test]
    fn shift_tells_arrows_apart_but_not_letters_or_backtab() {
        let shifted = KeyEvent::new(KeyCode::Left, KeyModifiers::SHIFT);
        assert!(shift(LEFT).matches(&shifted));
        assert!(!LEFT.matches(&shifted));
        assert!(ch('K').matches(&KeyEvent::new(KeyCode::Char('K'), KeyModifiers::SHIFT)));
        assert!(BACKTAB.matches(&KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT)));
    }

    #[test]
    fn cmd_is_ctrl() {
        let cmd_p = KeyEvent::new(KeyCode::Char('p'), KeyModifiers::SUPER);
        assert!(ctrl(ch('p')).matches(&cmd_p));
        assert!(!ch('p').matches(&cmd_p));
        assert_eq!(
            resolve(Scheme::Ide.defaults().sessions, &cmd_p),
            Some(Command::Sessions(SessionsCommand::OpenPlacesPicker))
        );
    }

    #[test]
    fn vim_keeps_its_letters() {
        let m = Scheme::Vim.defaults();
        assert_eq!(
            resolve(m.sessions, &press('j')),
            Some(Command::Sessions(SessionsCommand::NavDown))
        );
        assert_eq!(
            resolve(m.global, &press('K')),
            Some(GlobalCommand::CycleTab { back: false })
        );
        assert_eq!(
            resolve(m.board, &press('x')),
            Some(BoardKey::Do(TasksCommand::DeleteSelected))
        );
    }

    // `bin/` tells a user who presses j or k under the IDE scheme where the
    // Vim keys went; that only works while no IDE tab binds them.
    #[test]
    fn ide_moves_with_arrows_and_leaves_j_and_k_free() {
        let m = Scheme::Ide.defaults();
        assert_eq!(
            resolve(m.sessions, &KeyEvent::from(KeyCode::Down)),
            Some(Command::Sessions(SessionsCommand::NavDown))
        );
        assert_eq!(
            resolve(m.board, &KeyEvent::from(KeyCode::Delete)),
            Some(BoardKey::Do(TasksCommand::DeleteSelected))
        );
        for c in ['j', 'k'] {
            let ev = press(c);
            assert!(resolve(m.global, &ev).is_none());
            assert!(resolve(m.sessions, &ev).is_none());
            assert!(resolve(m.board, &ev).is_none());
            assert!(resolve(m.agents, &ev).is_none());
            assert!(resolve(m.metrics, &ev).is_none());
            assert!(resolve(m.settings, &ev).is_none());
        }
    }

    #[test]
    fn hints_follow_the_table() {
        let line = hints(Scheme::Vim.defaults().metrics);
        assert_eq!(line, "enter:view transcript  j/k:select  r:refresh");
        let relabelled = hints_with(Scheme::Vim.defaults().list_picker, " · ", |k| match k {
            ListKey::Confirm => Hint::As("respawn"),
            ListKey::Cancel => Hint::Skip,
            _ => Hint::Keep,
        });
        assert_eq!(relabelled, "j/k:move · enter/space:respawn");
    }

    #[test]
    fn key_for_names_the_first_key() {
        let add = BoardKey::Do(TasksCommand::OpenAddInput);
        assert_eq!(key_for(Scheme::Vim.defaults().board, add), "a");
        assert_eq!(key_for(Scheme::Ide.defaults().board, add), "ctrl+n");
    }
}
