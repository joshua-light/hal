//! The one-row status bar: status message or per-view key hints, the pending
//! dispatch indicator, and the refresh age.

use crate::app::{App, Tab, View};
use crate::config;
use crate::folder_picker::PickerMode;
use crate::keymap;
use crate::models;
use crate::ui::agents;
use crate::ui::hints::{self, Folders};
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;
use std::time::Duration;

const ERROR_STATUS_TTL: Duration = Duration::from_secs(30);

/// Hints for the views where printable keys are text, or the pane's.
fn typing_hints(app: &App) -> String {
    use crate::keymap::PaneKey;
    let keys = app.keys;
    let maps = keys.maps();
    let text = |what: &str, submit, cancel, switch| {
        format!(
            "{}  {}",
            what,
            hints::text(keys, "  ", submit, cancel, switch)
        )
    };
    let release = keymap::key_for(maps.pane, PaneKey::Release);
    match app.view {
        View::ConfirmClose => keymap::hints(maps.confirm_close),
        View::ModelPicker
            if app
                .model_picker
                .as_ref()
                .is_some_and(|picker| picker.has_multiple_agents()) =>
        {
            hints::filter(keys, "  ", "start", Some("agent"))
        }
        View::ModelPicker => hints::filter(keys, "  ", "start", None),
        View::TaskLinkPicker => hints::filter(keys, "  ", "link", None),
        View::SessionFinder => {
            format!("type:filter  {}", keymap::hints(maps.session_finder))
        }
        View::RenameSession => text("edit title", "rename", "cancel", None),
        View::TmuxPane => format!("forwarding keys to tmux · {}: detach & close", release),
        View::Dock => format!("typing into the session · {}: back to the list", release),
        View::GhCreateInput => text(
            "type name",
            "create",
            "cancel",
            Some("toggle public/private"),
        ),
        View::TaskInput => {
            if app.tasks.renaming.is_some() {
                text("edit task", "rename", "cancel", None)
            } else {
                text(
                    "type task (paste lands in context)",
                    "add",
                    "cancel",
                    Some("context"),
                )
            }
        }
        View::TaskTags => text("edit tags  space/comma separates", "save", "cancel", None),
        View::TaskAttachInput => {
            if app.tasks.attach_note {
                text("type note", "attach", "cancel", Some("file/URL"))
            } else {
                text("paste path or URL", "attach", "cancel", Some("note"))
            }
        }
        View::TaskFilter => text("type to filter (text or #tag)", "apply", "clear", None),
        // Every other view prints its hints from its own table.
        _ => String::new(),
    }
}

pub(super) fn render_status_bar(frame: &mut Frame, area: Rect, app: &App) {
    let elapsed = app.last_refresh.elapsed().as_secs();
    let refresh_text = if elapsed < 2 {
        "just now".to_string()
    } else {
        models::relative_age(elapsed)
    };

    // An error tells the user to fix something, so it outlives the usual
    // flash long enough to be read and acted on.
    let (ttl, color) = if app.status_is_error {
        (
            config::get().ui.status_msg_ttl().max(ERROR_STATUS_TTL),
            Color::Red,
        )
    } else {
        (config::get().ui.status_msg_ttl(), Color::Green)
    };
    let fresh_status = app
        .status_msg
        .as_ref()
        .filter(|(_, ts)| ts.elapsed() < ttl)
        .map(|(msg, _)| msg.as_str());

    let mut spans: Vec<Span> = Vec::new();

    if let Some(msg) = fresh_status {
        spans.push(Span::styled(
            format!(" {} ", msg),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ));
    } else {
        let maps = app.keys.maps();
        // A tab's own keys lead; the ones every tab shares trail.
        let on_tab = |tab: String| format!("{}  {}", tab, keymap::hints(maps.global));
        let keybinds: String = match app.view {
            // Tables are in hint order: high-value verbs lead so they
            // survive the right-edge truncation at narrow widths (the
            // status bar is one row, no wrap). The Space chip is rendered
            // separately *ahead* of this string below so it is never the
            // first thing clipped.
            View::Grid => on_tab(match app.current_tab {
                Tab::Tasks => keymap::hints(maps.board),
                Tab::Sessions => keymap::hints(maps.sessions),
                Tab::Metrics => keymap::hints(maps.metrics),
                Tab::Agents => agents::hints(app),
            }),
            View::AgentDetail => agents::hints(app),
            View::Popup => keymap::hints(maps.popup),
            View::LiveTail => keymap::hints(maps.live_tail),
            View::Settings if app.settings.editing.is_some() => format!(
                "type a colour  {}",
                hints::text(app.keys, "  ", "apply", "cancel", None)
            ),
            View::Settings => keymap::hints(maps.settings),
            View::AgentPicker => hints::list_picker(app.keys, "  ", "select default"),
            View::RespawnPicker => hints::list_picker(app.keys, "  ", "respawn"),
            View::TaskKindPicker => hints::list_picker(app.keys, "  ", "set kind"),
            View::TaskInfo => keymap::hints(maps.task_info),
            View::FolderPicker => match app.folder_picker.as_ref().map(|p| p.mode) {
                Some(PickerMode::Bookmarks) => hints::folders(app.keys, "  ", Folders::Bookmarks),
                Some(PickerMode::Places) => {
                    hints::filter(app.keys, "  ", "pick", Some("browse folders"))
                }
                _ if app.tasks.pending_assign.is_some() => {
                    hints::folders(app.keys, "  ", Folders::Assigning)
                }
                _ => hints::folders(app.keys, "  ", Folders::Browse),
            },
            _ => typing_hints(app),
        };
        // Render the Space chip *first* so the single highest-value verb
        // (proceed/done on Tasks, hold on Sessions) is never the first thing
        // clipped off the right edge of this one-row, no-wrap status bar.
        let space_verb = match (&app.view, app.current_tab) {
            // Space is status-aware on the Tasks board: it approves a
            // focused Planning card's plan, and toggles Done elsewhere.
            (View::Grid, Tab::Tasks) => Some(match app.selected_board_task().map(|t| t.status) {
                Some(crate::tasks::store::TaskStatus::Planning) => "proceed ",
                _ => "done ",
            }),
            (View::Grid, Tab::Sessions) => Some(match app.selected_session_info() {
                Some(s) if app.sessions.holds.contains(&s.session_id) => "release ",
                _ => "hold ",
            }),
            _ => None,
        };
        if let Some(verb) = space_verb {
            spans.push(Span::styled(
                " Space ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::styled(
                verb,
                Style::default()
                    .fg(Color::LightCyan)
                    .add_modifier(Modifier::BOLD),
            ));
        }
        spans.push(Span::styled(
            format!(" {} ", keybinds),
            Style::default().fg(Color::DarkGray),
        ));
    }

    // Pending dispatch indicator — visible when a freshly-spawned session
    // has a queued prompt that hasn't fired yet. Without this the user has
    // no way to tell that "session sitting there empty" actually has a
    // dispatch in flight, or that it's about to time out.
    if let Some(target) = app.pending_dispatch_target() {
        let age = app.pending_dispatch_age().map(|d| d.as_secs()).unwrap_or(0);
        let queued = app.pending_dispatch_count();
        let suffix = if queued > 1 {
            format!(" +{}", queued - 1)
        } else {
            String::new()
        };
        spans.push(Span::styled(
            format!(" ↻ dispatch waiting [{}{}] {}s ", target, suffix, age),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
    }

    spans.push(Span::styled(
        format!("refreshed {} ", refresh_text),
        Style::default().fg(Color::DarkGray),
    ));

    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}
