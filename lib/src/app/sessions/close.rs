//! `x`, then `y`: close the selected session.
//!
//! The close itself lands in milliseconds, but the agent behind it takes its
//! time to exit, and every scan until then still reports it alive.
//! [`Closing`] covers that gap: the card leaves the grid when `y` is
//! pressed, not when the process finally goes.

use crate::app::App;
use crate::models::{SessionInfo, SessionState};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// How long hal keeps hiding a closed session that scans still report as
/// alive. An agent alive after this long survived its close, so its card
/// comes back.
const GRACE: Duration = Duration::from_secs(30);

/// Pids hal has closed whose processes a scan may still see.
#[derive(Default)]
pub(crate) struct Closing(HashMap<u32, Instant>);

impl Closing {
    fn insert(&mut self, pid: u32) {
        self.0.insert(pid, Instant::now());
    }

    /// Show each closed session in `sessions` the way a scan will report
    /// it once its process has exited: Inactive, with no tmux session.
    pub(super) fn apply(&mut self, sessions: &mut [SessionInfo]) {
        self.0.retain(|_, closed_at| closed_at.elapsed() < GRACE);
        if self.0.is_empty() {
            return;
        }
        for s in sessions.iter_mut().filter(|s| self.0.contains_key(&s.pid)) {
            s.state = SessionState::Inactive;
            s.tmux_session = None;
        }
    }
}

impl App {
    /// `y` in the close confirmation: close the staged session and drop its
    /// card at once. The cursor keeps its slot, so the next card slides
    /// under it, as with a hold.
    pub fn confirm_close(&mut self) {
        let Some(pending) = self.take_pending_close() else {
            return;
        };
        if !self.runtime.close_window(pending.pid) {
            self.set_status(format!("failed to close {}", pending.display));
            return;
        }
        let sessions = &mut self.sessions;
        sessions.closing.insert(pending.pid);
        sessions.closing.apply(&mut sessions.last_sessions);

        let slot = (self.sessions.sel_group, self.sessions.sel_in_group);
        self.rebuild_groups();
        (self.sessions.sel_group, self.sessions.sel_in_group) = slot;
        self.sessions.clamp_cursor();

        self.set_status(format!("closed {}", pending.display));
    }
}

#[cfg(all(test, unix))]
mod tests {
    use crate::app::test_support::{app_with, session, status};
    use crate::app::{Command, SessionsCommand, View};
    use crate::models::SessionState;

    fn ids(app: &crate::app::App) -> Vec<&str> {
        app.sessions
            .groups
            .iter()
            .flat_map(|g| g.sessions.iter().map(|s| s.session_id.as_str()))
            .collect()
    }

    fn close_selected(app: &mut crate::app::App) {
        app.execute(Command::Sessions(SessionsCommand::StageConfirmClose));
        assert_eq!(app.view, View::ConfirmClose);
        app.execute(Command::Sessions(SessionsCommand::ConfirmClose));
    }

    #[test]
    fn a_closed_card_leaves_the_grid_before_its_process_exits() {
        crate::test_util::with_temp_home(|| {
            let mut closed = session("a-1", SessionState::Idle, Some("hal-a-1"));
            closed.pid = 101;
            let mut kept = session("a-2", SessionState::Idle, Some("hal-a-2"));
            kept.pid = 102;
            let (mut app, runtime) = app_with(vec![closed.clone(), kept.clone()]);
            app.sessions.show_inactive = false;
            app.rebuild_groups();
            assert_eq!(app.selected_session_id().as_deref(), Some("a-1"));

            close_selected(&mut app);
            assert_eq!(*runtime.closes.lock().unwrap(), vec![101]);
            assert_eq!(ids(&app), vec!["a-2"]);
            assert_eq!(app.selected_session_id().as_deref(), Some("a-2"));
            assert_eq!(app.view, View::Grid);
            assert!(status(&app).starts_with("closed "));

            // The agent is still shutting down: the next scan reports it
            // alive, and it must not bring the card back.
            app.update_sessions(vec![closed, kept]);
            assert_eq!(ids(&app), vec!["a-2"]);
        });
    }

    #[test]
    fn a_closed_session_reads_as_inactive_with_no_tmux() {
        crate::test_util::with_temp_home(|| {
            let mut closed = session("a-1", SessionState::Processing, Some("hal-a-1"));
            closed.pid = 101;
            let (mut app, _runtime) = app_with(vec![closed.clone()]);

            close_selected(&mut app);
            app.update_sessions(vec![closed]);
            let card = app.selected_session_info().expect("shown as inactive");
            assert_eq!(card.state, SessionState::Inactive);
            assert_eq!(card.tmux_session, None);
            assert!(app.sessions_by_tmux().is_empty());
        });
    }
}
