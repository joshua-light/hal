use crate::persist::save_json;
use log::warn;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TickRecord {
    pub at: i64,
    pub event: Option<String>,
    /// The session this tick ran in, so its transcript can be reopened.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    pub ok: bool,
    pub subtype: Option<String>,
    pub turns: u32,
    pub compactions: u32,
    pub cost_usd: f64,
    pub context_start: u64,
    pub context_end: u64,
    pub duration_s: u64,
    /// The result, for the timeline: ~200 chars on success, more on
    /// failure, where it is the reason.
    pub result: String,
    /// The CLI's stderr tail and exit code, kept for failed runs only —
    /// often the one line that says why.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// Supervisor-owned bookkeeping. The agent's own working state lives in its
/// workdir and is none of the harness's business.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct AgentState {
    pub session_id: Option<String>,
    pub ticks: u64,
    pub compactions: u64,
    pub cost_usd: f64,
    pub failures_in_a_row: u32,
    /// Set when the loop halted itself (budget, usage limit, failures).
    /// Cleared by `resume` / `reset`, or by the supervisor at
    /// `stopped_until`.
    pub stopped_reason: Option<String>,
    /// When the halt lifts itself (unix): a daily budget at the next UTC
    /// midnight, a usage limit at its reset. `None` waits for `resume`.
    pub stopped_until: Option<i64>,
    /// User-requested pause. The supervisor skips a paused agent.
    pub paused: bool,
    pub last_tick_at: Option<i64>,
    pub last_result: String,
    /// `Some` while a tick is in flight: (unix start, event id).
    pub ticking: Option<Ticking>,
    /// Spend for `today` (UTC date); rolls over when the date changes.
    pub today: String,
    pub today_cost_usd: f64,
    pub today_ticks: u64,
    pub history: Vec<TickRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Ticking {
    pub since: i64,
    pub event: Option<String>,
    /// Filled the moment the CLI announces it, so the tab can tail a tick
    /// while it is still running.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

pub(super) fn today_utc() -> String {
    chrono::Utc::now().format("%Y-%m-%d").to_string()
}

pub fn now_unix() -> i64 {
    chrono::Utc::now().timestamp()
}

/// The first second of the UTC day after `now`: when daily spend rolls over.
pub(super) fn next_utc_midnight(now: i64) -> i64 {
    const DAY_S: i64 = 86_400;
    (now.div_euclid(DAY_S) + 1) * DAY_S
}

/// `at` on the local wall clock, with the weekday when that isn't today.
fn local_clock(at: i64, now: i64) -> String {
    use chrono::{Local, TimeZone};
    let local = |t| Local.timestamp_opt(t, 0).single();
    match (local(at), local(now)) {
        (Some(at), Some(now)) if at.date_naive() == now.date_naive() => {
            at.format("%H:%M").to_string()
        }
        (Some(at), _) => at.format("%a %H:%M").to_string(),
        _ => "later".into(),
    }
}

/// Why the loop must stop, and when that stops being true by itself.
#[derive(Debug, Clone, PartialEq)]
pub struct Halt {
    pub reason: String,
    /// See [`AgentState::stopped_until`].
    pub until: Option<i64>,
}

impl AgentState {
    pub(super) fn halt(&mut self, halt: Halt) {
        self.stopped_reason = Some(halt.reason);
        self.stopped_until = halt.until;
    }

    pub(super) fn clear_halt(&mut self) {
        self.stopped_reason = None;
        self.stopped_until = None;
    }

    /// True for a halt whose reason has run out: the day rolled over, the
    /// usage limit reset.
    pub fn halt_is_over(&self, now: i64) -> bool {
        self.stopped_reason.is_some() && self.stopped_until.is_some_and(|t| now >= t)
    }

    /// When the current halt lifts itself, if it does.
    pub fn resumes_at(&self) -> Option<i64> {
        self.stopped_reason.as_ref().and(self.stopped_until)
    }

    /// The halt in one line, for the log and the status bar: its reason,
    /// and when it lifts itself if it does.
    pub fn halt_label(&self) -> Option<String> {
        let reason = self.stopped_reason.as_ref()?;
        Some(match self.stopped_until {
            Some(t) => format!("{} · resumes itself {}", reason, local_clock(t, now_unix())),
            None => reason.clone(),
        })
    }

    pub(super) fn roll_day(&mut self) {
        let t = today_utc();
        if self.today != t {
            self.today = t;
            self.today_cost_usd = 0.0;
            self.today_ticks = 0;
        }
    }

    /// Spend today, correct even if the date rolled since the last write.
    pub fn today_cost(&self) -> f64 {
        if self.today == today_utc() {
            self.today_cost_usd
        } else {
            0.0
        }
    }

    pub fn today_ticks(&self) -> u64 {
        if self.today == today_utc() {
            self.today_ticks
        } else {
            0
        }
    }
}

pub fn state_path(dir: &Path) -> PathBuf {
    dir.join("state.json")
}

pub fn load_state(dir: &Path) -> AgentState {
    let path = state_path(dir);
    match fs::read_to_string(&path) {
        Ok(raw) => serde_json::from_str(&raw).unwrap_or_else(|e| {
            warn!(
                "harness: {} unreadable ({}), starting fresh",
                path.display(),
                e
            );
            AgentState::default()
        }),
        Err(_) => AgentState::default(),
    }
}

/// Read-mutate-write `state.json` under the agent's `state.lock`, so the
/// TUI supervisor and CLI verbs can't clobber each other.
pub fn update_state<F: FnOnce(&mut AgentState)>(dir: &Path, f: F) -> io::Result<AgentState> {
    fs::create_dir_all(dir)?;
    let lock = crate::persist::lock_exclusive(&dir.join("state.lock"))?;
    let mut state = load_state(dir);
    f(&mut state);
    save_json(&state_path(dir), &state)?;
    let _ = lock.unlock();
    Ok(state)
}

pub fn set_paused(dir: &Path, paused: bool) -> io::Result<AgentState> {
    update_state(dir, |s| {
        s.paused = paused;
        if !paused {
            // Resume also clears a halt so the user has one verb for "go".
            s.clear_halt();
            s.failures_in_a_row = 0;
        }
    })
}

/// Clear harness bookkeeping. The workdir is untouched.
pub fn reset(dir: &Path) -> io::Result<()> {
    let _ = fs::remove_file(state_path(dir));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_roundtrip_under_lock() {
        let tmp = tempfile::tempdir().unwrap();
        let s = update_state(tmp.path(), |s| {
            s.ticks = 3;
            s.paused = true;
        })
        .unwrap();
        assert_eq!(s.ticks, 3);
        assert_eq!(load_state(tmp.path()), s);
        let s2 = set_paused(tmp.path(), false).unwrap();
        assert!(!s2.paused);
    }

    #[test]
    fn a_halt_is_over_only_past_its_own_deadline() {
        let mut s = AgentState::default();
        assert!(!s.halt_is_over(100), "not halted");

        s.halt(Halt {
            reason: "5 consecutive failed ticks".into(),
            until: None,
        });
        assert!(!s.halt_is_over(i64::MAX), "waits for resume");
        assert_eq!(s.resumes_at(), None);

        s.halt(Halt {
            reason: "usage limit reached".into(),
            until: Some(100),
        });
        assert!(!s.halt_is_over(99));
        assert!(s.halt_is_over(100));
        assert_eq!(s.resumes_at(), Some(100));
    }

    #[test]
    fn resume_clears_the_deadline_with_the_halt() {
        let tmp = tempfile::tempdir().unwrap();
        update_state(tmp.path(), |s| {
            s.halt(Halt {
                reason: "daily budget $1 reached".into(),
                until: Some(100),
            })
        })
        .unwrap();
        let s = set_paused(tmp.path(), false).unwrap();
        assert_eq!((s.stopped_reason, s.stopped_until), (None, None));
    }

    #[test]
    fn the_day_rolls_at_utc_midnight() {
        // 2026-11-12T00:00:00Z
        const MIDNIGHT: i64 = 1_794_441_600;
        assert_eq!(next_utc_midnight(MIDNIGHT - 1), MIDNIGHT);
        assert_eq!(next_utc_midnight(MIDNIGHT), MIDNIGHT + 86_400);
        assert_eq!(next_utc_midnight(MIDNIGHT + 5), MIDNIGHT + 86_400);
    }
}
