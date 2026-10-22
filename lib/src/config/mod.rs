//! User config at `~/.hal/config.toml`, loaded once and exposed via
//! [`get`]. Missing file, missing section, and missing field all fall back
//! to [`Default`], so this is a pure knob layer — removing the file yields
//! the same behaviour as shipped defaults.
//!
//! The settings window rewrites the file through [`update`], and the new
//! config replaces the live one at once.
//!
//! - `agents`: `[spawn]`, `[agents.<id>]` and the resolved agent list.
//! - `theme`: `[theme]`, the preset palette and its overrides.
//! - `sections`: every other `[section]`.

use serde::de::IgnoredAny;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{OnceLock, PoisonError, RwLock};

mod agents;
mod sections;
pub mod theme;

pub use agents::*;
pub use sections::*;
pub use theme::ThemeConfig;

pub fn config_path() -> Option<PathBuf> {
    crate::platform::paths::hal_home().map(|h| h.join("config.toml"))
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub spawn: SpawnConfig,
    pub agents: BTreeMap<String, ConfiguredAgent>,
    pub projects: ProjectsConfig,
    pub tasks: TasksConfig,
    pub title: TitleConfig,
    pub inactive: InactiveConfig,
    pub scan: ScanConfig,
    pub ui: UiConfig,
    pub metrics: MetricsConfig,
    pub harness: HarnessConfig,
    pub theme: ThemeConfig,
    /// Retired with the Projects layer. Accepted and ignored so an old
    /// config keeps loading: `deny_unknown_fields` would otherwise fail the
    /// parse and drop every other setting back to defaults.
    #[serde(rename = "backlog")]
    _backlog: Option<IgnoredAny>,
    #[serde(rename = "auto_review")]
    _auto_review: Option<IgnoredAny>,
    /// Retired with the Builds tab.
    #[serde(rename = "builds")]
    _builds: Option<IgnoredAny>,
}

/// The live config. Each one is leaked, so a `&'static` handed out before
/// a settings change stays valid; the window changes it a handful of times
/// a session.
fn live() -> &'static RwLock<&'static Config> {
    static CFG: OnceLock<RwLock<&'static Config>> = OnceLock::new();
    CFG.get_or_init(|| RwLock::new(Box::leak(Box::new(load()))))
}

pub fn get() -> &'static Config {
    *live().read().unwrap_or_else(PoisonError::into_inner)
}

/// Replace the live config: every later [`get`] sees `cfg`.
pub fn set(cfg: Config) {
    *live().write().unwrap_or_else(PoisonError::into_inner) = Box::leak(Box::new(cfg));
}

/// Edit `config.toml` with `edit`, keeping its comments and layout, and make
/// the result the live config. `edit` also gets the config the file holds
/// now, which a hand edit since startup may have moved past [`get`]. The
/// edited file must parse, or nothing is written: a file hal would reject is
/// worse than the change not landing.
pub fn update(edit: impl FnOnce(&mut toml_edit::DocumentMut, &Config)) -> Result<(), String> {
    let path = config_path().ok_or("no home directory")?;
    let raw = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(format!("{}: {}", path.display(), e)),
    };
    let (updated, cfg) = edit_text(&raw, edit)?;
    crate::persist::write_atomic(&path, updated.as_bytes())
        .map_err(|e| format!("{}: {}", path.display(), e))?;
    set(cfg);
    Ok(())
}

/// Pure core of [`update`]: the edited text and the config it loads as.
fn edit_text(
    raw: &str,
    edit: impl FnOnce(&mut toml_edit::DocumentMut, &Config),
) -> Result<(String, Config), String> {
    let before = toml::from_str::<Config>(raw)
        .map_err(|e| format!("config.toml doesn't load, fix it first: {}", e.message()))?;
    let mut doc: toml_edit::DocumentMut = raw
        .parse()
        .map_err(|e: toml_edit::TomlError| format!("config.toml: {}", e.message()))?;
    edit(&mut doc, &before);
    let updated = doc.to_string();
    let cfg = toml::from_str::<Config>(&updated).map_err(|e| e.message().to_string())?;
    Ok((updated, cfg))
}

/// The table at `path` (`["theme", "colors"]`), created when missing. A
/// table the file already has keeps its form, inline or not.
pub fn table_mut<'a>(
    doc: &'a mut toml_edit::DocumentMut,
    path: &[&str],
) -> &'a mut dyn toml_edit::TableLike {
    let mut t: &mut dyn toml_edit::TableLike = doc.as_table_mut();
    for name in path {
        let item = t.entry(name).or_insert_with(|| {
            let mut new = toml_edit::Table::new();
            // A table that only holds sub-tables prints no header of its
            // own: `[theme.colors]` without an empty `[theme]` above it.
            new.set_implicit(true);
            toml_edit::Item::Table(new)
        });
        t = item
            .as_table_like_mut()
            .expect("config.toml loaded, so its sections are tables");
    }
    t
}

fn load() -> Config {
    let Some(path) = config_path() else {
        log::debug!("config: no home dir, using defaults");
        return Config::default();
    };
    let raw = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            log::debug!("config: {} not found, using defaults", path.display());
            return Config::default();
        }
        Err(e) => {
            log::warn!(
                "config: read error at {}: {} — using defaults",
                path.display(),
                e
            );
            return Config::default();
        }
    };
    match toml::from_str::<Config>(&raw) {
        Ok(cfg) => {
            log::info!("config: loaded {}", path.display());
            cfg
        }
        Err(e) => {
            log::warn!(
                "config: parse error in {}: {} — using defaults",
                path.display(),
                e
            );
            Config::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_edit_keeps_comments_and_adds_nested_tables() {
        let raw = "# mine\n[ui]\ncell_width = 50 # wide\n";
        let (text, cfg) = edit_text(raw, |doc, _| {
            table_mut(doc, &["ui"]).insert("cell_height", toml_edit::value(8));
            table_mut(doc, &["theme", "colors"]).insert("p1", toml_edit::value("#ff0000"));
        })
        .unwrap();
        assert!(text.starts_with("# mine\n[ui]\ncell_width = 50 # wide\n"));
        assert!(text.contains("[theme.colors]\np1 = \"#ff0000\""));
        assert!(!text.contains("[theme]\n"));
        assert_eq!((cfg.ui.cell_width, cfg.ui.cell_height), (50, 8));
    }

    #[test]
    fn an_edit_that_breaks_the_file_is_refused() {
        let bad = edit_text("", |doc, _| {
            table_mut(doc, &["theme"]).insert("border", toml_edit::value("wavy"));
        });
        assert!(bad.is_err());
        let broken = edit_text("[ui]\nnope = 1\n", |_, _| {});
        assert!(broken.unwrap_err().contains("fix it first"));
    }

    #[test]
    fn empty_toml_yields_defaults() {
        let cfg: Config = toml::from_str("").unwrap();
        let def = Config::default();
        assert_eq!(cfg.spawn.command, def.spawn.command);
        assert_eq!(cfg.title.model, def.title.model);
        assert_eq!(cfg.inactive.window_secs, def.inactive.window_secs);
        assert_eq!(cfg.inactive.orphan_relist_secs, 30);
        assert!(!cfg.ui.show_planning_column);
    }

    /// Keys the Projects layer owned must not fail the parse: a parse error
    /// drops the whole config back to defaults.
    #[test]
    fn retired_projects_keys_are_ignored() {
        let src = r#"
            [backlog]
            enabled = true

            [auto_review]
            enabled = true
            interval_secs = 30

            [ui]
            show_projects_tab = true
            cell_width = 50

            [projects]
            default_orchestrator_agent = "claude"
        "#;
        let cfg: Config = toml::from_str(src).unwrap();
        assert_eq!(cfg.ui.cell_width, 50);
    }

    /// Nor may a recipe written for the retired Builds tab.
    #[test]
    fn retired_builds_recipes_are_ignored() {
        let src = r#"
            [builds.recipes.server]
            resource = "build-box"
            run = [["build-server", "{ref}"]]

            [ui]
            cell_width = 50
        "#;
        let cfg: Config = toml::from_str(src).unwrap();
        assert_eq!(cfg.ui.cell_width, 50);
    }

    #[test]
    fn partial_section_merges_with_defaults() {
        let src = r#"
            [title]
            model = "sonnet"
        "#;
        let cfg: Config = toml::from_str(src).unwrap();
        assert_eq!(cfg.title.model, "sonnet");
        assert!(cfg.title.enabled);
        assert_eq!(cfg.title.max_length, 40);
    }

    #[test]
    fn unknown_field_rejected() {
        let src = r#"
            [title]
            mdoel = "sonnet"
        "#;
        let err = toml::from_str::<Config>(src).unwrap_err();
        assert!(err.to_string().contains("unknown field"));
    }
}
