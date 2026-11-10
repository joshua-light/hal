//! Named colours shared across the UI, named by role where one is
//! obvious, else by value. The constants are fixed; the functions read the
//! live theme (`[theme]`, [`crate::config::theme`]).

use crate::config;
use crate::config::theme::Palette;
use ratatui::style::Color;
use ratatui::widgets::BorderType;

/// The live theme's palette.
pub(crate) fn theme() -> &'static Palette {
    &config::get().theme.palette
}

/// The line every card, column and popup frame is drawn with.
pub(crate) fn border_type() -> BorderType {
    config::get().theme.file.border.border_type()
}

/// Separator for `│` dividers and unfocused card borders.
pub(crate) fn separator_fg() -> Color {
    theme().separator
}

/// Soft accent (cwd path, note icon).
pub(crate) fn accent_fg() -> Color {
    theme().accent
}

/// Planning/thinking accent: the Planning column's hue.
pub(crate) fn planning_fg() -> Color {
    theme().planning
}

/// Backlog accent (chip backlog count, backlog popup arrows/title).
pub(crate) fn backlog_fg() -> Color {
    theme().backlog
}

/// Task `#tag` badges — distinct from the priority hues and column accents,
/// reads as secondary metadata on the card border.
pub(crate) fn tag_fg() -> Color {
    theme().tag
}

/// The task-kind chip — a filled chip like the priority badge it mirrors on
/// the other corner, in a hue no column accent or priority uses.
pub(crate) fn kind_fg() -> Color {
    theme().kind
}

/// The session marked for a handoff (`h`) — brighter and cooler than the
/// Question state's colour, so the mark reads as the user's own rather than
/// as a card asking something.
pub(crate) fn handoff_fg() -> Color {
    theme().handoff
}

/// The cold-cache snowflake (session quiet past the prompt-cache TTL — see
/// [`crate::models::SessionInfo::cache_cold`]).
pub(crate) fn cold_fg() -> Color {
    theme().cold
}

/// Background of the selected row in one-line lists (sessions list, agents
/// table and detail lists).
pub(crate) fn selected_row_bg() -> Color {
    theme().selected_row
}

/// Background of the cursor row in a sidebar without the keyboard —
/// [`selected_row_bg`] sunk, so the list's cursor fades as the page's frame
/// goes gray. The sidebar with the keyboard keeps the plain one.
pub(crate) fn unfocused_row_bg() -> Color {
    theme().unfocused_row
}

/// Background of a row marked for a handoff (`h`) — [`handoff_fg`] sunk
/// into the dark, so the mark tints the whole row without outshouting it.
pub(crate) fn handoff_row_bg() -> Color {
    theme().handoff_row
}

/// Background of the handoff row under the cursor: the same hue, lifted as
/// far as [`selected_row_bg`] lifts a plain row.
pub(crate) fn selected_handoff_row_bg() -> Color {
    theme().selected_handoff_row
}

/// The handoff row under the cursor of a sidebar without the keyboard,
/// halfway between [`handoff_row_bg`] and [`selected_handoff_row_bg`].
pub(crate) fn unfocused_handoff_row_bg() -> Color {
    theme().unfocused_handoff_row
}

/// Low-priority/dim body text and footers.
pub(crate) const DIM_TEXT: Color = Color::Rgb(110, 110, 130);

/// Faint secondary text — captions, prompt previews, queued-card body.
pub(crate) const FAINT_TEXT: Color = Color::Rgb(180, 180, 200);

/// Muted icon/prefix gray used ahead of labels in popup/card bodies.
pub(crate) const MUTED_TEXT: Color = Color::Rgb(100, 100, 120);

/// Mid gray for inline labels (`agents`, `merges`, chip prefix).
pub(crate) const LABEL_GRAY: Color = Color::Rgb(150, 150, 170);

/// Metadata gray for card footers / queued-merge glyphs.
pub(crate) const META_GRAY: Color = Color::Rgb(110, 120, 135);

/// Diff/context and tail-entry gray.
pub(crate) const CONTEXT_GRAY: Color = Color::Rgb(160, 160, 170);

/// Idle/overflow dot and narrow-board hint gray.
pub(crate) const DOT_IDLE: Color = Color::Rgb(140, 140, 160);
