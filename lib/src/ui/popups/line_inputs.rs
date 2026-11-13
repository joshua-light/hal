//! Single-line editors: task attachment and task tags.

use super::widgets::{input_line, input_rows, key_footer, render_line_input};
use crate::app::App;
use crate::ui::common::{centered_fixed, CURSOR};
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Span;
use ratatui::Frame;

/// Centered single-line input for attaching to the focused board card. Same
/// shape as the task input popup; the buffer accepts a paste (bracketed paste
/// routes here via `App::paste_into_input`), enter attaches, esc cancels.
/// Opens in note mode — the buffer becomes a typed `note` attachment; Tab
/// flips to file-path/URL mode and back, keeping the buffer.
pub(crate) fn render_task_attach_input(frame: &mut Frame, area: Rect, app: &App) {
    let note_mode = app.tasks.attach_note;
    let mut text = app.tasks.input.clone();
    text.push(CURSOR);
    let prefix = Span::styled(
        if note_mode { "  ✎ " } else { "  📎 " },
        Style::default().fg(Color::Green),
    );
    let input = input_line(prefix, text, Color::White);

    let desired_w = 70u16.min(area.width);
    let desired_h = 5u16
        .saturating_add(input_rows(&input, desired_w))
        .max(9)
        .min(area.height);
    let popup = centered_fixed(area, desired_w, desired_h);

    let (title, hint) = if note_mode {
        (" Attach note ", " saved as a .md note inside the task ")
    } else {
        (
            " Attach file or URL ",
            " files are copied into the task — safe to clean the original ",
        )
    };
    render_line_input(
        frame,
        popup,
        title,
        hint,
        input,
        key_footer("attach", Some(if note_mode { "file/URL" } else { "note" })),
    );
}

/// Centered single-line editor for the focused task's tags. Same shape as the
/// task input popup: the buffer is prefilled with the current tags, typing
/// edits the whole set (space/comma separated), enter saves, esc cancels.
pub(crate) fn render_task_tags(frame: &mut Frame, area: Rect, app: &App) {
    let mut text = app.tasks.input.clone();
    text.push(CURSOR);
    let prefix = Span::styled("  # ", Style::default().fg(Color::Cyan));
    let input = input_line(prefix, text, Color::White);

    let desired_w = 70u16.min(area.width);
    let desired_h = 5u16
        .saturating_add(input_rows(&input, desired_w))
        .max(9)
        .min(area.height);
    let popup = centered_fixed(area, desired_w, desired_h);

    render_line_input(
        frame,
        popup,
        " Edit tags ",
        " space or comma separates tags ",
        input,
        key_footer("save", None),
    );
}
