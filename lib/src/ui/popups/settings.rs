//! The settings window (`,`): one row per `[ui]` / `[theme]` knob, grouped
//! under headings, with a swatch beside every colour.

use crate::app::{settings_lines, App, SettingKind, SettingsLine};
use crate::config;
use crate::config::theme::ThemeColor;
use crate::ui::common::{centered_fixed, pad, popup_block, CURSOR};
use crate::ui::palette::{selected_row_bg, DIM_TEXT, LABEL_GRAY};
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Clear, Paragraph};
use ratatui::Frame;

const LABEL_W: usize = 28;
const SWATCH: &str = "██ ";

pub(crate) fn render_settings(frame: &mut Frame, area: Rect, app: &App) {
    let lines = settings_lines();
    let cfg = config::get();
    let view = &app.settings;

    // Stop short of the status bar: a rejected colour says why down there.
    let above_status = Rect {
        height: area.height.saturating_sub(1),
        ..area
    };
    let popup = centered_fixed(above_status, 64, lines.len() as u16 + 2);
    frame.render_widget(Clear, popup);
    let hints = if view.editing.is_some() {
        " name, 0–255 or #rrggbb · enter:apply · empty:preset · esc:cancel "
    } else {
        " j/k:move · h/l:change · enter:edit · d:default · esc:close "
    };
    let block = popup_block(Span::styled(
        " Settings · ~/.hal/config.toml ",
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    ))
    .title_bottom(Span::styled(hints, Style::default().fg(DIM_TEXT)));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    // Keep the cursor row in view: scroll only once it would leave the
    // bottom third, so the headings above it stay readable.
    let height = inner.height as usize;
    let offset = view
        .selected
        .saturating_sub(height.saturating_sub(height / 3 + 1))
        .min(lines.len().saturating_sub(height));

    let rows: Vec<Line> = lines
        .iter()
        .enumerate()
        .skip(offset)
        .take(height)
        .map(|(i, line)| {
            let s = match line {
                SettingsLine::Heading(h) => {
                    return Line::from(Span::styled(
                        format!(" {h}"),
                        Style::default().fg(LABEL_GRAY).add_modifier(Modifier::BOLD),
                    ));
                }
                SettingsLine::Setting(s) => s,
            };
            let selected = i == view.selected;
            let label_style = if selected {
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            let mut spans = vec![Span::styled(
                format!(
                    "{} {}",
                    if selected { "▶" } else { " " },
                    pad(s.label, LABEL_W)
                ),
                label_style,
            )];
            match (&view.editing, selected) {
                (Some(text), true) => {
                    // Preview what is typed once it parses.
                    let preview = text.parse::<ThemeColor>().ok().map(|c| c.0);
                    spans.push(Span::styled(
                        SWATCH,
                        Style::default().fg(preview.unwrap_or(Color::Reset)),
                    ));
                    spans.push(Span::styled(
                        format!("{text}{CURSOR}"),
                        Style::default().fg(Color::White),
                    ));
                }
                _ => {
                    if let Some(c) = s.color(cfg) {
                        spans.push(Span::styled(SWATCH, Style::default().fg(c)));
                    }
                    let value = s.value(cfg);
                    let value = match s.kind {
                        SettingKind::Choice(_) | SettingKind::Number { .. } if selected => {
                            format!("‹ {value} ›")
                        }
                        _ => value,
                    };
                    spans.push(Span::styled(value, label_style));
                    if !s.is_set(cfg) {
                        spans.push(Span::styled(" (preset)", Style::default().fg(DIM_TEXT)));
                    }
                }
            }
            let line = Line::from(spans);
            if selected {
                line.style(Style::default().bg(selected_row_bg()))
            } else {
                line
            }
        })
        .collect();
    frame.render_widget(Paragraph::new(rows), inner);
}
