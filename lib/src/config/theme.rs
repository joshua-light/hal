//! `[theme]`: how hal looks — a preset palette, per-role colour overrides
//! (`[theme.colors]`), the border style, the spinner and the state dot.
//!
//! The palette resolves once per load: [`ThemeConfig`] deserializes through
//! [`ThemeFile`], so the renderer reads finished colours from
//! [`ThemeConfig::palette`] instead of parsing strings every frame.

use ratatui::style::Color;
use ratatui::widgets::BorderType;
use serde::{Deserialize, Deserializer};
use std::str::FromStr;

/// A colour as the config spells it: a name (`green`, `light-blue`), an
/// ANSI index (`0`–`255`) or `#rrggbb`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemeColor(pub Color);

impl FromStr for ThemeColor {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Color::from_str(s.trim())
            .map(ThemeColor)
            .map_err(|_| format!("{s:?} is not a colour — use a name, 0–255 or #rrggbb"))
    }
}

impl<'de> Deserialize<'de> for ThemeColor {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(d)?;
        raw.parse().map_err(serde::de::Error::custom)
    }
}

/// Declares the palette once: the resolved [`Palette`], its
/// `[theme.colors]` overrides, and the role list the settings window walks.
macro_rules! palette {
    ($($role:ident: $label:literal),* $(,)?) => {
        /// Every colour a theme sets, by role.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct Palette {
            $(pub $role: Color,)*
        }

        /// `[theme.colors]`: a colour per role that replaces the preset's.
        #[derive(Debug, Clone, Default, Deserialize)]
        #[serde(default, deny_unknown_fields)]
        pub struct ColorOverrides {
            $(pub $role: Option<ThemeColor>,)*
        }

        /// `(key, label)` of every role, in settings-window order.
        pub const COLOR_ROLES: &[(&str, &str)] = &[$((stringify!($role), $label)),*];

        impl Palette {
            fn apply(&mut self, o: &ColorOverrides) {
                $(if let Some(c) = o.$role {
                    self.$role = c.0;
                })*
            }

            /// The colour of the role named `key`.
            pub fn get(&self, key: &str) -> Option<Color> {
                match key {
                    $(stringify!($role) => Some(self.$role),)*
                    _ => None,
                }
            }
        }

        impl ColorOverrides {
            /// Whether `[theme.colors]` sets the role named `key`.
            pub fn has(&self, key: &str) -> bool {
                match key {
                    $(stringify!($role) => self.$role.is_some(),)*
                    _ => false,
                }
            }
        }
    };
}

palette! {
    working: "Session: working",
    waiting: "Session: waiting for you",
    asking: "Session: asking",
    failed: "Session: failed",
    resting: "Session: idle",
    backlog: "Board: backlog",
    planning: "Board: planning",
    running: "Board: in progress",
    review: "Board: review",
    done: "Board: done",
    p1: "Priority P1",
    p2: "Priority P2",
    p3: "Priority P3",
    p4: "Priority P4",
    usage_low: "Usage bar: low",
    usage_mid: "Usage bar: 50%+",
    usage_high: "Usage bar: 80%+",
    accent: "Accent (paths, notes)",
    separator: "Separators, idle borders",
    tag: "Task #tags",
    kind: "Task kind chip",
    handoff: "Handoff mark",
    cold: "Cold-cache snowflake",
    selected_row: "Selected row",
    unfocused_row: "Selected row, unfocused",
    handoff_row: "Handoff row",
    selected_handoff_row: "Handoff row, selected",
    unfocused_handoff_row: "Handoff row, unfocused",
}

const fn rgb(hex: u32) -> Color {
    Color::Rgb((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
}

/// A ready-made palette; `[theme.colors]` overrides its roles one by one.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ThemePreset {
    #[default]
    Default,
    Nord,
    Gruvbox,
    Catppuccin,
    HighContrast,
}

impl ThemePreset {
    pub const ALL: &[Self] = &[
        Self::Default,
        Self::Nord,
        Self::Gruvbox,
        Self::Catppuccin,
        Self::HighContrast,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Nord => "nord",
            Self::Gruvbox => "gruvbox",
            Self::Catppuccin => "catppuccin",
            Self::HighContrast => "high-contrast",
        }
    }

    pub fn palette(self) -> Palette {
        match self {
            Self::Default => Palette {
                working: Color::White,
                waiting: Color::Green,
                asking: Color::LightBlue,
                failed: Color::Red,
                resting: rgb(0x50505a),
                backlog: rgb(0x788cc8),
                planning: rgb(0xaa8cd2),
                running: Color::LightYellow,
                review: Color::LightCyan,
                done: Color::LightGreen,
                p1: Color::LightRed,
                p2: Color::LightYellow,
                p3: Color::LightGreen,
                p4: Color::LightBlue,
                usage_low: Color::Green,
                usage_mid: Color::Yellow,
                usage_high: Color::Red,
                accent: rgb(0xb4c8e6),
                separator: rgb(0x3c3c46),
                tag: rgb(0x96aac8),
                kind: rgb(0x78c8be),
                handoff: rgb(0x6ebeff),
                cold: rgb(0x82bedc),
                selected_row: rgb(0x282834),
                unfocused_row: rgb(0x1e1e28),
                handoff_row: rgb(0x162c44),
                selected_handoff_row: rgb(0x224062),
                unfocused_handoff_row: rgb(0x1c3653),
            },
            Self::Nord => Palette {
                working: rgb(0xeceff4),
                waiting: rgb(0xa3be8c),
                asking: rgb(0x81a1c1),
                failed: rgb(0xbf616a),
                resting: rgb(0x4c566a),
                backlog: rgb(0x5e81ac),
                planning: rgb(0xb48ead),
                running: rgb(0xebcb8b),
                review: rgb(0x88c0d0),
                done: rgb(0xa3be8c),
                p1: rgb(0xbf616a),
                p2: rgb(0xebcb8b),
                p3: rgb(0xa3be8c),
                p4: rgb(0x81a1c1),
                usage_low: rgb(0xa3be8c),
                usage_mid: rgb(0xebcb8b),
                usage_high: rgb(0xbf616a),
                accent: rgb(0x88c0d0),
                separator: rgb(0x434c5e),
                tag: rgb(0x81a1c1),
                kind: rgb(0x8fbcbb),
                handoff: rgb(0x5e81ac),
                cold: rgb(0x88c0d0),
                selected_row: rgb(0x3b4252),
                unfocused_row: rgb(0x2e3440),
                handoff_row: rgb(0x29374a),
                selected_handoff_row: rgb(0x3a4f6a),
                unfocused_handoff_row: rgb(0x31435a),
            },
            Self::Gruvbox => Palette {
                working: rgb(0xebdbb2),
                waiting: rgb(0xb8bb26),
                asking: rgb(0x83a598),
                failed: rgb(0xfb4934),
                resting: rgb(0x665c54),
                backlog: rgb(0x83a598),
                planning: rgb(0xd3869b),
                running: rgb(0xfabd2f),
                review: rgb(0x8ec07c),
                done: rgb(0xb8bb26),
                p1: rgb(0xfb4934),
                p2: rgb(0xfabd2f),
                p3: rgb(0xb8bb26),
                p4: rgb(0x83a598),
                usage_low: rgb(0xb8bb26),
                usage_mid: rgb(0xfabd2f),
                usage_high: rgb(0xfb4934),
                accent: rgb(0xfe8019),
                separator: rgb(0x504945),
                tag: rgb(0xa89984),
                kind: rgb(0x8ec07c),
                handoff: rgb(0x458588),
                cold: rgb(0x83a598),
                selected_row: rgb(0x3c3836),
                unfocused_row: rgb(0x32302f),
                handoff_row: rgb(0x1d3638),
                selected_handoff_row: rgb(0x2a4c4f),
                unfocused_handoff_row: rgb(0x234143),
            },
            Self::Catppuccin => Palette {
                working: rgb(0xcdd6f4),
                waiting: rgb(0xa6e3a1),
                asking: rgb(0x89b4fa),
                failed: rgb(0xf38ba8),
                resting: rgb(0x6c7086),
                backlog: rgb(0x74c7ec),
                planning: rgb(0xcba6f7),
                running: rgb(0xf9e2af),
                review: rgb(0x94e2d5),
                done: rgb(0xa6e3a1),
                p1: rgb(0xf38ba8),
                p2: rgb(0xfab387),
                p3: rgb(0xa6e3a1),
                p4: rgb(0x89b4fa),
                usage_low: rgb(0xa6e3a1),
                usage_mid: rgb(0xf9e2af),
                usage_high: rgb(0xf38ba8),
                accent: rgb(0xb4befe),
                separator: rgb(0x45475a),
                tag: rgb(0x9399b2),
                kind: rgb(0x94e2d5),
                handoff: rgb(0x89dceb),
                cold: rgb(0x89dceb),
                selected_row: rgb(0x313244),
                unfocused_row: rgb(0x262637),
                handoff_row: rgb(0x1f3346),
                selected_handoff_row: rgb(0x2b4a63),
                unfocused_handoff_row: rgb(0x253f55),
            },
            Self::HighContrast => Palette {
                working: Color::White,
                waiting: Color::LightGreen,
                asking: Color::LightCyan,
                failed: Color::LightRed,
                resting: Color::Gray,
                backlog: Color::LightBlue,
                planning: Color::LightMagenta,
                running: Color::LightYellow,
                review: Color::LightCyan,
                done: Color::LightGreen,
                p1: Color::LightRed,
                p2: Color::LightYellow,
                p3: Color::LightGreen,
                p4: Color::LightBlue,
                usage_low: Color::LightGreen,
                usage_mid: Color::LightYellow,
                usage_high: Color::LightRed,
                accent: Color::LightCyan,
                separator: Color::Gray,
                tag: Color::White,
                kind: Color::LightCyan,
                handoff: Color::LightBlue,
                cold: Color::LightCyan,
                selected_row: rgb(0x3c3c64),
                unfocused_row: rgb(0x28283c),
                handoff_row: rgb(0x003060),
                selected_handoff_row: rgb(0x00509a),
                unfocused_handoff_row: rgb(0x00407c),
            },
        }
    }
}

/// The line every card, column and popup frame is drawn with.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BorderStyle {
    #[default]
    Rounded,
    Plain,
    Thick,
    Double,
}

impl BorderStyle {
    pub const ALL: &[Self] = &[Self::Rounded, Self::Plain, Self::Thick, Self::Double];

    pub fn key(self) -> &'static str {
        match self {
            Self::Rounded => "rounded",
            Self::Plain => "plain",
            Self::Thick => "thick",
            Self::Double => "double",
        }
    }

    pub fn border_type(self) -> BorderType {
        match self {
            Self::Rounded => BorderType::Rounded,
            Self::Plain => BorderType::Plain,
            Self::Thick => BorderType::Thick,
            Self::Double => BorderType::Double,
        }
    }
}

/// The glyph that turns while work is in flight.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SpinnerStyle {
    #[default]
    Braille,
    Dots,
    Line,
    Arc,
    /// One still glyph: work in flight without the motion.
    Static,
}

impl SpinnerStyle {
    pub const ALL: &[Self] = &[
        Self::Braille,
        Self::Dots,
        Self::Line,
        Self::Arc,
        Self::Static,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Self::Braille => "braille",
            Self::Dots => "dots",
            Self::Line => "line",
            Self::Arc => "arc",
            Self::Static => "static",
        }
    }

    pub fn frames(self) -> &'static [&'static str] {
        match self {
            Self::Braille => &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            Self::Dots => &["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"],
            Self::Line => &["-", "\\", "|", "/"],
            Self::Arc => &["◜", "◠", "◝", "◞", "◡", "◟"],
            Self::Static => &["⠿"],
        }
    }
}

/// The glyph every session and agent draws for its state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StateDotStyle {
    #[default]
    Circle,
    Diamond,
    Square,
    Ring,
}

impl StateDotStyle {
    pub const ALL: &[Self] = &[Self::Circle, Self::Diamond, Self::Square, Self::Ring];

    pub fn key(self) -> &'static str {
        match self {
            Self::Circle => "circle",
            Self::Diamond => "diamond",
            Self::Square => "square",
            Self::Ring => "ring",
        }
    }

    pub fn glyph(self) -> &'static str {
        match self {
            Self::Circle => "●",
            Self::Diamond => "◆",
            Self::Square => "■",
            Self::Ring => "◉",
        }
    }
}

/// `[theme]` as written in the file.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ThemeFile {
    pub preset: ThemePreset,
    pub border: BorderStyle,
    pub spinner: SpinnerStyle,
    pub state_dot: StateDotStyle,
    /// A working session's dot blinks; off, it stays the working colour.
    pub blink: bool,
    pub colors: ColorOverrides,
}

impl Default for ThemeFile {
    fn default() -> Self {
        Self {
            preset: ThemePreset::default(),
            border: BorderStyle::default(),
            spinner: SpinnerStyle::default(),
            state_dot: StateDotStyle::default(),
            blink: true,
            colors: ColorOverrides::default(),
        }
    }
}

/// `[theme]` with its palette resolved: the preset, then every
/// `[theme.colors]` override on top.
#[derive(Debug, Clone, Deserialize)]
#[serde(from = "ThemeFile")]
pub struct ThemeConfig {
    pub file: ThemeFile,
    pub palette: Palette,
}

impl From<ThemeFile> for ThemeConfig {
    fn from(file: ThemeFile) -> Self {
        let mut palette = file.preset.palette();
        palette.apply(&file.colors);
        Self { file, palette }
    }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        ThemeFile::default().into()
    }
}

#[cfg(test)]
mod tests {
    use crate::config::Config;
    use ratatui::style::Color;

    #[test]
    fn an_override_replaces_one_role_of_the_preset() {
        let src = r##"
            [theme]
            preset = "nord"
            [theme.colors]
            waiting = "#ff0000"
        "##;
        let cfg: Config = toml::from_str(src).unwrap();
        let nord = super::ThemePreset::Nord.palette();
        assert_eq!(cfg.theme.palette.waiting, Color::Rgb(255, 0, 0));
        assert_eq!(cfg.theme.palette.asking, nord.asking);
    }

    #[test]
    fn a_colour_may_be_a_name_or_an_index() {
        let cfg: Config =
            toml::from_str("[theme.colors]\np1 = \"light-magenta\"\np2 = \"208\"").unwrap();
        assert_eq!(cfg.theme.palette.p1, Color::LightMagenta);
        assert_eq!(cfg.theme.palette.p2, Color::Indexed(208));
    }

    #[test]
    fn a_bad_colour_fails_the_parse() {
        assert!(toml::from_str::<Config>("[theme.colors]\np1 = \"blurple\"").is_err());
    }

    #[test]
    fn an_unknown_role_fails_the_parse() {
        assert!(toml::from_str::<Config>("[theme.colors]\nnope = \"red\"").is_err());
    }
}
