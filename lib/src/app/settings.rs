//! The settings window (`,`): the `[ui]` and `[theme]` knobs of
//! `config.toml`, changed in place. Every change rewrites the file through
//! [`config::update`] and repaints at once.

use super::{App, View};
use crate::config::theme::{ThemeColor, COLOR_ROLES};
use crate::config::{self, Config};
use ratatui::style::Color;
use toml_edit::value;

/// What one row edits, and how `←`/`→` move it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingKind {
    Toggle,
    /// One of a fixed list of words.
    Choice(&'static [&'static str]),
    Number {
        min: i64,
        max: i64,
        step: i64,
    },
    /// A colour typed as a name, an index or `#rrggbb`; unset falls back to
    /// the preset's.
    Color,
}

/// One row of the window: a key under `[table]` of `config.toml`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Setting {
    pub table: &'static [&'static str],
    pub key: &'static str,
    pub label: &'static str,
    pub kind: SettingKind,
}

/// A line of the window: a section heading or a setting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsLine {
    Heading(&'static str),
    Setting(Setting),
}

const LAYOUTS: &[&str] = &["split", "list", "grid"];
const PRESETS: &[&str] = &["default", "nord", "gruvbox", "catppuccin", "high-contrast"];
const BORDERS: &[&str] = &["rounded", "plain", "thick", "double"];
const SPINNERS: &[&str] = &["braille", "dots", "line", "arc", "static"];
const DOTS: &[&str] = &["circle", "diamond", "square", "ring"];

const UI: &[&str] = &["ui"];
const THEME: &[&str] = &["theme"];
const COLORS: &[&str] = &["theme", "colors"];

const fn setting(
    table: &'static [&'static str],
    key: &'static str,
    label: &'static str,
    kind: SettingKind,
) -> SettingsLine {
    SettingsLine::Setting(Setting {
        table,
        key,
        label,
        kind,
    })
}

/// Every line of the window, in order: layout, then style, then one row
/// per palette role.
pub fn settings_lines() -> Vec<SettingsLine> {
    let mut lines = vec![
        SettingsLine::Heading("Layout"),
        setting(
            UI,
            "sessions_layout",
            "Sessions layout on start",
            SettingKind::Choice(LAYOUTS),
        ),
        setting(
            UI,
            "cell_height",
            "Session card height",
            SettingKind::Number {
                min: 3,
                max: 12,
                step: 1,
            },
        ),
        setting(
            UI,
            "cell_width",
            "Session card min width",
            SettingKind::Number {
                min: 24,
                max: 120,
                step: 2,
            },
        ),
        setting(
            UI,
            "show_planning_column",
            "Planning column",
            SettingKind::Toggle,
        ),
        setting(
            UI,
            "dim_unfocused",
            "Dim the side without keys",
            SettingKind::Toggle,
        ),
        SettingsLine::Heading("Style"),
        setting(THEME, "preset", "Theme", SettingKind::Choice(PRESETS)),
        setting(THEME, "border", "Borders", SettingKind::Choice(BORDERS)),
        setting(THEME, "spinner", "Spinner", SettingKind::Choice(SPINNERS)),
        setting(THEME, "state_dot", "State dot", SettingKind::Choice(DOTS)),
        setting(THEME, "blink", "Blink working dot", SettingKind::Toggle),
        SettingsLine::Heading("Colours"),
    ];
    lines.extend(
        COLOR_ROLES
            .iter()
            .map(|&(key, label)| setting(COLORS, key, label, SettingKind::Color)),
    );
    lines
}

impl Setting {
    /// The live value, as the window shows it and as `config.toml` spells it.
    pub fn value(&self, cfg: &Config) -> String {
        let (ui, t) = (&cfg.ui, &cfg.theme.file);
        match self.key {
            "sessions_layout" => ui.sessions_layout.key().into(),
            "cell_height" => ui.cell_height.to_string(),
            "cell_width" => ui.cell_width.to_string(),
            "show_planning_column" => on_off(ui.show_planning_column),
            "dim_unfocused" => on_off(ui.dim_unfocused),
            "preset" => t.preset.key().into(),
            "border" => t.border.key().into(),
            "spinner" => t.spinner.key().into(),
            "state_dot" => t.state_dot.key().into(),
            "blink" => on_off(t.blink),
            role => cfg
                .theme
                .palette
                .get(role)
                .map(color_text)
                .unwrap_or_default(),
        }
    }

    /// Whether the file sets this row: a colour row left unset takes the
    /// preset's.
    pub fn is_set(&self, cfg: &Config) -> bool {
        self.kind != SettingKind::Color || cfg.theme.file.colors.has(self.key)
    }

    /// The colour a colour row shows as its swatch.
    pub fn color(&self, cfg: &Config) -> Option<Color> {
        match self.kind {
            SettingKind::Color => cfg.theme.palette.get(self.key),
            _ => None,
        }
    }

    /// The value one step along, or `None` where stepping means nothing.
    fn step(&self, cfg: &Config, delta: i64) -> Option<toml_edit::Item> {
        let current = self.value(cfg);
        match self.kind {
            SettingKind::Toggle => Some(value(current != "on")),
            SettingKind::Choice(words) => {
                let at = words.iter().position(|w| *w == current).unwrap_or(0) as i64;
                let n = words.len() as i64;
                Some(value(words[(at + delta).rem_euclid(n) as usize]))
            }
            SettingKind::Number { min, max, step } => {
                let n: i64 = current.parse().unwrap_or(min);
                Some(value((n + delta * step).clamp(min, max)))
            }
            SettingKind::Color => None,
        }
    }
}

fn on_off(b: bool) -> String {
    if b { "on" } else { "off" }.into()
}

/// A colour as a user would type it back: `#rrggbb`, or the name or index.
pub fn color_text(c: Color) -> String {
    match c {
        Color::Rgb(r, g, b) => format!("#{r:02x}{g:02x}{b:02x}"),
        Color::Indexed(i) => i.to_string(),
        other => kebab(&format!("{other:?}")),
    }
}

/// `LightBlue` → `light-blue`, the spelling `[theme.colors]` reads.
fn kebab(name: &str) -> String {
    let mut out = String::new();
    for (i, ch) in name.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            out.push('-');
        }
        out.extend(ch.to_lowercase());
    }
    out
}

/// State behind [`View::Settings`].
#[derive(Clone, Debug, Default)]
pub struct SettingsView {
    /// Index into [`settings_lines`]; always on a setting, never a heading.
    pub selected: usize,
    /// The colour being typed, while a colour row is open for editing.
    pub editing: Option<String>,
}

impl SettingsView {
    pub fn setting(&self) -> Option<Setting> {
        match settings_lines().get(self.selected)? {
            SettingsLine::Setting(s) => Some(*s),
            SettingsLine::Heading(_) => None,
        }
    }
}

impl App {
    pub fn open_settings(&mut self) {
        let first = settings_lines()
            .iter()
            .position(|l| matches!(l, SettingsLine::Setting(_)))
            .unwrap_or(0);
        self.settings = SettingsView {
            selected: first,
            editing: None,
        };
        self.view = View::Settings;
    }

    pub fn close_settings(&mut self) {
        self.settings.editing = None;
        self.view = View::Grid;
    }

    /// Move the cursor `delta` settings, stepping over headings.
    pub fn settings_move(&mut self, delta: isize) {
        let lines = settings_lines();
        let mut at = self.settings.selected as isize;
        loop {
            at += delta.signum();
            match lines.get(at as usize) {
                _ if at < 0 => return,
                None => return,
                Some(SettingsLine::Heading(_)) => continue,
                Some(SettingsLine::Setting(_)) => break,
            }
        }
        self.settings.selected = at as usize;
    }

    /// `←`/`→` on the selected row: flip, cycle or count.
    pub fn settings_step(&mut self, delta: i64) {
        let Some(s) = self.settings.setting() else {
            return;
        };
        if s.kind != SettingKind::Color {
            // Step from the file, not the live value: a hand edit since
            // startup is what the user sees in the file.
            self.write_setting(s, |file| s.step(file, delta));
        }
    }

    /// Enter on the selected row: open a colour for typing, else step it on.
    pub fn settings_activate(&mut self) {
        let Some(s) = self.settings.setting() else {
            return;
        };
        if s.kind == SettingKind::Color {
            self.settings.editing = Some(s.value(config::get()));
        } else {
            self.settings_step(1);
        }
    }

    /// Drop the selected row from the file, back to its default (or, for a
    /// colour, the preset's).
    pub fn settings_reset(&mut self) {
        if let Some(s) = self.settings.setting() {
            self.write_setting(s, |_| None);
        }
    }

    /// Enter in the colour box: apply it when it parses.
    pub fn settings_submit_color(&mut self) {
        let (Some(s), Some(text)) = (self.settings.setting(), self.settings.editing.clone()) else {
            return;
        };
        let text = text.trim().to_string();
        if text.is_empty() {
            self.settings.editing = None;
            self.write_setting(s, |_| None);
            return;
        }
        if let Err(e) = text.parse::<ThemeColor>() {
            self.set_error(e);
            return;
        }
        self.settings.editing = None;
        self.write_setting(s, |_| Some(value(text)));
    }

    /// Write the row's new value — `change` sees the file's config and
    /// gives the value, or `None` to drop the key.
    fn write_setting(
        &mut self,
        s: Setting,
        change: impl FnOnce(&Config) -> Option<toml_edit::Item>,
    ) {
        let planning_was = config::get().ui.show_planning_column;
        let result = config::update(|doc, file| {
            let item = change(file);
            let t = config::table_mut(doc, s.table);
            match item {
                Some(item) => {
                    t.insert(s.key, item);
                }
                None => {
                    t.remove(s.key);
                }
            }
        });
        if let Err(e) = result {
            self.set_error(e);
            return;
        }
        // Hiding or showing Planning renumbers the board's columns.
        if config::get().ui.show_planning_column != planning_was {
            let (col, row) = (self.tasks.col, self.tasks.row);
            self.tasks.focus(col, row);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::theme::{BorderStyle, SpinnerStyle, StateDotStyle, ThemePreset};
    use crate::config::StartLayout;

    /// The window's word lists follow each enum's `ALL`, so a choice it
    /// offers is always one the parser takes.
    #[test]
    fn choice_lists_match_the_parser() {
        fn keys<T: Copy>(all: &[T], key: fn(T) -> &'static str) -> Vec<&'static str> {
            all.iter().map(|&v| key(v)).collect()
        }
        assert_eq!(LAYOUTS, keys(StartLayout::ALL, StartLayout::key));
        assert_eq!(PRESETS, keys(ThemePreset::ALL, ThemePreset::key));
        assert_eq!(BORDERS, keys(BorderStyle::ALL, BorderStyle::key));
        assert_eq!(SPINNERS, keys(SpinnerStyle::ALL, SpinnerStyle::key));
        assert_eq!(DOTS, keys(StateDotStyle::ALL, StateDotStyle::key));
    }

    #[test]
    fn a_colour_reads_back_as_typed() {
        for text in ["#ff8800", "light-blue", "green", "208"] {
            let c = text.parse::<ThemeColor>().unwrap().0;
            assert_eq!(color_text(c), text);
        }
    }

    #[test]
    fn stepping_wraps_choices_and_clamps_numbers() {
        let cfg = Config::default();
        let border = Setting {
            table: THEME,
            key: "border",
            label: "",
            kind: SettingKind::Choice(BORDERS),
        };
        let back = border.step(&cfg, -1).unwrap();
        assert_eq!(back.as_str(), Some("double"));
        let height = Setting {
            table: UI,
            key: "cell_height",
            label: "",
            kind: SettingKind::Number {
                min: 3,
                max: 12,
                step: 1,
            },
        };
        assert_eq!(height.step(&cfg, 100).unwrap().as_integer(), Some(12));
    }
}
