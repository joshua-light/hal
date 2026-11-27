//! A session as `hal --no-tui` prints it: one text line, or one JSON object.
//!
//! The title is the name the user sees in the TUI, so a script that reads
//! this listing and a person who reads the grid call a session the same
//! thing. Titles are not unique: two sessions can share one, and the session
//! id is the only key.

use crate::models::{short_sid, SessionInfo};
use serde_json::{json, Value};

pub struct Listing<'a>(pub &'a SessionInfo);

impl Listing<'_> {
    /// `pid:sid [state] agent project title last-message`. The title is
    /// padded, not cut, so a long name stays whole.
    pub fn text(&self) -> String {
        let s = self.0;
        format!(
            "{:>7}:{} [{:<17}] {:<10} {:<24} {:<24} {}",
            s.pid,
            short_sid(&s.session_id),
            s.state,
            s.agent_badge(),
            s.project_name,
            s.title.as_deref().unwrap_or(""),
            s.last_user_message.as_deref().unwrap_or(""),
        )
    }

    pub fn json(&self) -> Value {
        let s = self.0;
        json!({
            "session_id": s.session_id,
            "title": s.title.as_deref().filter(|t| !t.is_empty()),
            "pid": s.pid,
            "state": s.state.to_string(),
            "agent": s.agent_badge(),
            "project": s.project_name,
            "cwd": s.cwd,
            "tmux": s.tmux_session,
            "last_user_message": s.last_user_message,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::session_info;

    fn named(title: &str) -> SessionInfo {
        SessionInfo {
            session_id: "c301846a-a2da-404c-b870-9bd105098814".into(),
            title: Some(title.into()),
            ..session_info()
        }
    }

    #[test]
    fn the_line_carries_the_title_between_project_and_message() {
        let s = SessionInfo {
            last_user_message: Some("go".into()),
            ..named("Overruns")
        };

        let line = Listing(&s).text();

        let project_at = line.find("tmp").unwrap();
        let title_at = line.find("Overruns").unwrap();
        let message_at = line.rfind("go").unwrap();
        assert!(project_at < title_at && title_at < message_at);
    }

    #[test]
    fn a_long_title_is_not_cut() {
        let long = "A name that is longer than the column is wide";

        assert!(Listing(&named(long)).text().contains(long));
    }

    #[test]
    fn json_keys_the_title_by_the_full_session_id() {
        let json = Listing(&named("Overruns")).json();

        assert_eq!(json["session_id"], "c301846a-a2da-404c-b870-9bd105098814");
        assert_eq!(json["title"], "Overruns");
    }

    #[test]
    fn json_has_a_null_title_for_a_nameless_session() {
        let nameless = SessionInfo {
            title: None,
            ..named("")
        };
        let empty = named("");

        assert!(Listing(&nameless).json()["title"].is_null());
        assert!(Listing(&empty).json()["title"].is_null());
    }
}
