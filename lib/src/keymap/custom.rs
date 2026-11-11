//! `[keymap.<place>]`: the user's own keys, laid over the scheme in force.
//!
//! ```toml
//! [keymap.sessions]
//! new_in = "ctrl+p"
//! close = ["del", "x"]
//! shell = []          # unbound
//! ```
//!
//! An override replaces every key of its action. A key it takes leaves
//! whatever else held it in that place, so the user's key always wins; a
//! `[keymap.global]` key also leaves the tabs, which are asked first.
//! Entries that name no place, no action or no key are skipped and reported
//! by [`problems`], never fatal: one typo must not cost the other keys.

use super::*;
use crate::config::{self, Config};
use std::collections::BTreeMap;
use std::sync::{PoisonError, RwLock};

/// One action's keys in the file: `"ctrl+p"` or `["del", "x"]`.
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum KeyList {
    One(String),
    Many(Vec<String>),
}

impl KeyList {
    fn names(&self) -> &[String] {
        match self {
            KeyList::One(key) => std::slice::from_ref(key),
            KeyList::Many(keys) => keys,
        }
    }
}

/// `[keymap]` as the file holds it: place, then action, then keys.
pub type Overrides = BTreeMap<String, BTreeMap<String, KeyList>>;

/// Both schemes with one config's overrides applied.
struct Live {
    /// Address of the config these were built from. Configs are leaked, so
    /// an address names one config for good.
    config: usize,
    maps: [&'static Keymaps; 2],
    problems: &'static [String],
}

static LIVE: RwLock<Option<Live>> = RwLock::new(None);

fn with_live<T>(read: impl Fn(&Live) -> T) -> T {
    let cfg = config::get();
    let id = cfg as *const Config as usize;
    if let Some(live) = LIVE
        .read()
        .unwrap_or_else(PoisonError::into_inner)
        .as_ref()
        .filter(|live| live.config == id)
    {
        return read(live);
    }
    let live = build_live(id, &cfg.keymap);
    let out = read(&live);
    *LIVE.write().unwrap_or_else(PoisonError::into_inner) = Some(live);
    out
}

fn build_live(config: usize, overrides: &Overrides) -> Live {
    if overrides.is_empty() {
        return Live {
            config,
            maps: Scheme::ALL.map(Scheme::defaults),
            problems: &[],
        };
    }
    // Both schemes report the same typos; the active one is as good as any.
    let mut problems = Vec::new();
    let maps = Scheme::ALL.map(|scheme| {
        let (maps, found) = customize(scheme.defaults(), overrides);
        problems = found;
        &*Box::leak(Box::new(maps))
    });
    for problem in &problems {
        log::warn!("config: [keymap] {}", problem);
    }
    Live {
        config,
        maps,
        problems: Box::leak(problems.into_boxed_slice()),
    }
}

/// The scheme's tables with the live config's `[keymap]` applied.
pub(super) fn maps(scheme: Scheme) -> &'static Keymaps {
    with_live(|live| live.maps[scheme as usize])
}

/// What is wrong with the live config's `[keymap]`, one line per entry.
pub fn problems() -> &'static [String] {
    with_live(|live| live.problems)
}

/// `base` with `overrides` applied, and what was wrong with them.
pub(super) fn customize(base: &Keymaps, overrides: &Overrides) -> (Keymaps, Vec<String>) {
    let mut problems = Vec::new();
    for place in overrides.keys() {
        if !PLACES.contains(&place.as_str()) {
            problems.push(format!("{}: no such place", place));
        }
    }
    let global_keys = parsed(overrides.get("global"));
    macro_rules! place {
        ($field:ident, $shared:expr) => {
            build(
                stringify!($field),
                base.$field,
                overrides.get(stringify!($field)),
                $shared,
                &mut problems,
            )
        };
    }
    let maps = Keymaps {
        global: place!(global, &[]),
        sessions: place!(sessions, &global_keys),
        board: place!(board, &global_keys),
        agents: place!(agents, &global_keys),
        metrics: place!(metrics, &global_keys),
        task_info: place!(task_info, &[]),
        agent_page: place!(agent_page, &[]),
        settings: place!(settings, &[]),
        popup: place!(popup, &[]),
        live_tail: place!(live_tail, &[]),
        list_picker: place!(list_picker, &[]),
        folders: place!(folders, &[]),
        filter_picker: place!(filter_picker, &[]),
        session_finder: place!(session_finder, &[]),
        text_input: place!(text_input, &[]),
        confirm_close: place!(confirm_close, &[]),
        pane: place!(pane, &[]),
        everywhere: place!(everywhere, &[]),
    };
    (maps, problems)
}

/// Every key a place's overrides name, bad ones dropped.
fn parsed(over: Option<&BTreeMap<String, KeyList>>) -> Vec<Key> {
    over.into_iter()
        .flat_map(BTreeMap::values)
        .flat_map(|list| list.names())
        .filter_map(|name| Key::parse(name).ok())
        .collect()
}

/// One place's table. `shared` are keys the user gave to the global table,
/// which this place must let go of.
fn build<A: Action>(
    place: &str,
    base: Keymap<A>,
    over: Option<&BTreeMap<String, KeyList>>,
    shared: &[Key],
    problems: &mut Vec<String>,
) -> Keymap<A> {
    let mut keys: Vec<Vec<Key>> = base.iter().map(|b| b.keys.to_vec()).collect();
    let mut mine = vec![false; base.len()];
    for (name, list) in over.into_iter().flatten() {
        let Some(row) = base.iter().position(|b| b.action.name() == name) else {
            problems.push(format!("{}.{}: no such action", place, name));
            continue;
        };
        let mut new = Vec::new();
        for text in list.names() {
            match Key::parse(text) {
                Ok(key) => new.push(key),
                Err(why) => problems.push(format!("{}.{}: {}", place, name, why)),
            }
        }
        // Every key misspelt: keep the default rather than unbind.
        if new.is_empty() && !list.names().is_empty() {
            continue;
        }
        keys[row] = new;
        mine[row] = true;
    }

    let taken: Vec<(usize, Key)> = (0..base.len())
        .filter(|row| mine[*row])
        .flat_map(|row| keys[row].iter().map(move |key| (row, *key)))
        .collect();
    for (row, held) in keys.iter_mut().enumerate() {
        if mine[row] {
            for (other, key) in &taken {
                if *other > row && held.iter().any(|k| k.same_press(key)) {
                    problems.push(format!(
                        "{}: {} is bound to both {} and {}",
                        place,
                        key.label(),
                        base[row].action.name(),
                        base[*other].action.name()
                    ));
                }
            }
        } else {
            held.retain(|k| {
                !taken.iter().any(|(_, key)| k.same_press(key))
                    && !shared.iter().any(|key| k.same_press(key))
            });
        }
    }

    if keys.iter().zip(base).all(|(new, b)| new == b.keys) {
        return base;
    }
    let built: Vec<Binding<A>> = (0..base.len())
        .map(|row| Binding {
            keys: Vec::leak(keys[row].clone()),
            action: base[row].action,
            bar: bar(base, &keys, row),
        })
        .collect();
    Vec::leak(built)
}

/// The hint for `row` once the keys are `keys`. Hand-spelled text stays
/// while it is still true: until the row, or a row joined to it, changes.
fn bar<A>(base: Keymap<A>, keys: &[Vec<Key>], row: usize) -> Bar {
    let Bar::Text(_, label) = base[row].bar else {
        return base[row].bar;
    };
    let joined = base[row + 1..]
        .iter()
        .take_while(|b| matches!(b.bar, Bar::Join))
        .count();
    let group = row..=row + joined;
    if group.clone().all(|r| keys[r] == base[r].keys) {
        return base[row].bar;
    }
    let names: Vec<String> = if joined == 0 {
        keys[row].iter().map(Key::label).collect()
    } else {
        group
            .filter_map(|r| keys[r].first().map(Key::label))
            .collect()
    };
    if names.is_empty() {
        return Bar::Hide;
    }
    Bar::Text(String::leak(names.join("/")), label)
}

/// Every place and action with the keys it has now, as the TOML that would
/// bind them: the list of what `[keymap]` can name.
pub fn dump(scheme: Scheme) -> String {
    let maps = scheme.maps();
    let mut out = format!(
        "# Keys in force: the {} scheme and [keymap] in config.toml.\n\
         # Copy a line into config.toml under the same header to change it.\n",
        scheme.config_value()
    );
    macro_rules! place {
        ($($field:ident),+) => {
            $(dump_place(&mut out, stringify!($field), maps.$field);)+
        };
    }
    place!(
        global,
        sessions,
        board,
        task_info,
        agents,
        agent_page,
        metrics,
        settings,
        popup,
        live_tail,
        list_picker,
        folders,
        filter_picker,
        session_finder,
        text_input,
        confirm_close,
        pane,
        everywhere
    );
    out
}

fn dump_place<A: Action>(out: &mut String, place: &str, map: Keymap<A>) {
    out.push_str(&format!("\n[keymap.{}]\n", place));
    for b in map {
        let keys: Vec<String> = b
            .keys
            .iter()
            .map(|key| toml_edit::Value::from(key.label()).to_string())
            .collect();
        out.push_str(&format!("{} = [{}]\n", b.action.name(), keys.join(", ")));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::SessionsCommand as S;

    fn overrides(toml: &str) -> Overrides {
        toml::from_str(toml).unwrap()
    }

    fn custom(scheme: Scheme, toml: &str) -> (Keymaps, Vec<String>) {
        customize(scheme.defaults(), &overrides(toml))
    }

    fn press(text: &str) -> KeyEvent {
        Key::parse(text).unwrap().event()
    }

    #[test]
    fn an_override_replaces_the_keys_of_its_action() {
        let (maps, problems) = custom(Scheme::Ide, "[sessions]\nnew_in = ['alt+o', 'O']\n");
        assert!(problems.is_empty(), "{:?}", problems);
        let open = Some(Command::Sessions(S::OpenPlacesPicker));
        assert_eq!(resolve(maps.sessions, &press("alt+o")), open);
        assert_eq!(resolve(maps.sessions, &press("O")), open);
        assert_eq!(resolve(maps.sessions, &press("ctrl+p")), None);
        assert!(hints(maps.sessions).contains("alt+o/O:new in…"));
    }

    #[test]
    fn an_override_takes_its_key_from_whoever_held_it() {
        let (maps, problems) = custom(Scheme::Vim, "[sessions]\nnew_in = 'x'\n");
        assert!(problems.is_empty());
        assert_eq!(
            resolve(maps.sessions, &press("x")),
            Some(Command::Sessions(S::OpenPlacesPicker))
        );
        // Close lost its only key, and its hint with it.
        assert!(!hints(maps.sessions).contains("close"));
    }

    #[test]
    fn a_global_key_leaves_the_tabs() {
        let (maps, _) = custom(Scheme::Vim, "[global]\nquit = 'x'\n");
        assert_eq!(resolve(maps.global, &press("x")), Some(GlobalCommand::Quit));
        assert_eq!(resolve(maps.sessions, &press("x")), None);
        assert_eq!(resolve(maps.board, &press("x")), None);
        assert_eq!(resolve(maps.global, &press("q")), None);
    }

    #[test]
    fn an_empty_list_unbinds() {
        let (maps, problems) = custom(Scheme::Ide, "[sessions]\nshell = []\n");
        assert!(problems.is_empty());
        assert_eq!(resolve(maps.sessions, &press("t")), None);
        assert!(!hints(maps.sessions).contains("shell"));
    }

    #[test]
    fn a_paired_hint_follows_either_half() {
        let (maps, _) = custom(Scheme::Vim, "[board]\nup = 'w'\n");
        let line = hints(maps.board);
        assert!(line.contains("↓/w:task"), "{}", line);
        assert!(line.contains("h/l:col"), "{}", line);
    }

    #[test]
    fn typos_are_reported_and_skipped() {
        let (maps, problems) = custom(
            Scheme::Ide,
            "[sesions]\nnew = 'n'\n[sessions]\nnwe = 'n'\nnew = 'hyper+n'\nack = 'i'\ninfo = 'i'\n",
        );
        assert_eq!(
            problems,
            [
                "sesions: no such place",
                "sessions.new: hyper+n: no such modifier: hyper",
                "sessions.nwe: no such action",
                "sessions: i is bound to both info and ack",
            ]
        );
        // The misspelt key left `new` on its default.
        assert_eq!(
            resolve(maps.sessions, &press("ctrl+n")),
            Some(Command::Sessions(S::SpawnAgentHere))
        );
    }

    #[test]
    fn text_boxes_and_the_pane_rebind_too() {
        let (maps, problems) = custom(
            Scheme::Ide,
            "[text_input]\nsubmit = 'ctrl+s'\n[pane]\nrelease = 'f12'\n[everywhere]\nredraw = 'f5'\n",
        );
        assert!(problems.is_empty(), "{:?}", problems);
        assert_eq!(
            resolve(maps.text_input, &press("ctrl+s")),
            Some(TextKey::Submit)
        );
        assert_eq!(resolve(maps.text_input, &press("enter")), None);
        assert_eq!(resolve(maps.pane, &press("f12")), Some(PaneKey::Release));
        assert_eq!(resolve(maps.pane, &press("f1")), None);
        assert_eq!(
            resolve(maps.everywhere, &press("f5")),
            Some(AnywhereKey::Redraw)
        );
    }

    #[test]
    fn untouched_places_keep_the_shipped_table() {
        let (maps, _) = custom(Scheme::Ide, "[sessions]\nnew_in = 'o'\n");
        assert!(std::ptr::eq(maps.board, Scheme::Ide.defaults().board));
    }

    #[test]
    fn the_dump_is_a_keymap_that_loads() {
        let text = dump(Scheme::Ide);
        let cfg: Config = toml::from_str(&text).unwrap();
        let (_, problems) = customize(Scheme::Ide.defaults(), &cfg.keymap);
        assert!(problems.is_empty(), "{:?}", problems);
        assert!(text.contains("[keymap.sessions]"));
    }
}
