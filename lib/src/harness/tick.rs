use super::state::next_utc_midnight;
use super::{
    log_event, log_path, now_unix, runner, trigger, update_state, AgentState, Event, Halt, Spec,
    Tick, TickRecord, Ticking,
};
use log::info;
use std::io;

pub const MAX_HISTORY: usize = 50;
pub const MAX_FAILURES_IN_A_ROW: u32 = 5;
/// How long a usage-limit halt lasts when the CLI didn't say when the limit
/// resets. A limited run costs nothing, so trying again is cheap.
const LIMIT_RETRY_S: i64 = 15 * 60;
/// Slack past the reset time, so the retry doesn't race the reset itself.
const LIMIT_RESET_SLACK_S: i64 = 60;

/// Run one tick and fold its outcome into `state.json`. Synchronous; the
/// supervisor calls it from `spawn_blocking`, the CLI directly.
pub fn tick_once(spec: &Spec, event: Option<&Event>) -> io::Result<(Tick, AgentState)> {
    let started = now_unix();
    let event_id = event.map(|e| e.id.clone());
    let state = update_state(&spec.dir, |s| {
        s.roll_day();
        // A persistent agent resumes a session we already know; a fresh one
        // gets its id from the CLI a moment later (see `session_started`).
        let resumed = spec.run.persistent_session.then(|| s.session_id.clone());
        s.ticking = Some(Ticking {
            since: started,
            event: event_id.clone(),
            session_id: resumed.flatten(),
        });
    })?;
    let resume = if spec.run.persistent_session {
        state.session_id.clone()
    } else {
        None
    };
    let number = state.ticks + 1;
    log_event(
        &spec.dir,
        "info",
        format!(
            "run #{} started · {}",
            number,
            event_id.as_deref().unwrap_or("manual")
        ),
    );
    let prompt = trigger::render_prompt(spec, event);
    let header = serde_json::json!({
        "type": "hal-tick",
        "agent": spec.name,
        "tick": state.ticks + 1,
        "event": event_id,
        "at": started,
    })
    .to_string();
    let tick = runner::run(
        spec,
        &prompt,
        resume.as_deref(),
        Some(&log_path(&spec.dir)),
        &header,
        &|sid| {
            let _ = update_state(&spec.dir, |s| {
                if let Some(t) = &mut s.ticking {
                    t.session_id = Some(sid.to_string());
                }
            });
        },
    );

    let state = update_state(&spec.dir, |s| {
        s.roll_day();
        s.ticking = None;
        s.ticks += 1;
        s.today_ticks += 1;
        s.compactions += tick.compactions as u64;
        s.cost_usd += tick.cost_usd;
        s.today_cost_usd += tick.cost_usd;
        s.last_tick_at = Some(now_unix());
        s.last_result = truncate(&tick.result, 800);
        if spec.run.persistent_session {
            if let Some(id) = &tick.session_id {
                s.session_id = Some(id.clone());
            }
        }
        // A thrashing session can never recover: drop it so the next tick
        // starts clean.
        if tick.thrashed() {
            s.session_id = None;
            s.last_result = "session dropped: autocompact thrashing (window too small)".into();
        } else if tick.compaction_starved() {
            s.last_result = format!(
                "budget spent on {} compactions, no work done — raise run.window_pct (now {}) or max_budget_usd",
                tick.compactions, spec.run.window_pct
            );
        }
        s.history.push(TickRecord {
            at: started,
            event: event_id.clone(),
            ok: tick.ok,
            subtype: tick.subtype.clone(),
            turns: tick.turns,
            compactions: tick.compactions,
            cost_usd: (tick.cost_usd * 10_000.0).round() / 10_000.0,
            context_start: tick.context_start,
            context_end: tick.context_end,
            duration_s: tick.duration_s,
            session_id: tick.session_id.clone(),
            result: truncate(&tick.result, if tick.ok { 200 } else { 800 }),
            detail: failure_detail(&tick),
        });
        if s.history.len() > MAX_HISTORY {
            let drop = s.history.len() - MAX_HISTORY;
            s.history.drain(..drop);
        }
        judge(s, &tick, now_unix());
    })?;
    if tick.ok {
        log_event(
            &spec.dir,
            "info",
            format!(
                "run #{} ok · {} turns · ${:.2} · {}s",
                number, tick.turns, tick.cost_usd, tick.duration_s
            ),
        );
    } else {
        log_event(
            &spec.dir,
            "warn",
            format!(
                "run #{} failed ({}): {}",
                number,
                tick.subtype.as_deref().unwrap_or("?"),
                truncate(&tick.result, 200)
            ),
        );
    }
    if let Some(halt) = state.halt_label() {
        // A halt that lifts itself is a wait, not a fault.
        let level = if state.resumes_at().is_some() {
            "warn"
        } else {
            "error"
        };
        log_event(&spec.dir, level, format!("halted: {}", halt));
    }
    info!(
        "harness[{}]: tick={} ok={} turns={} compact={} ${:.3} total=${:.2} {}",
        spec.name,
        state.ticks,
        tick.ok,
        tick.turns,
        tick.compactions,
        tick.cost_usd,
        state.cost_usd,
        tick.subtype.as_deref().unwrap_or("")
    );
    Ok((tick, state))
}

/// What a failed tick left on stderr, with its exit code: the part of a
/// failure the result text often lacks (a crash, a missing binary).
fn failure_detail(tick: &Tick) -> Option<String> {
    if tick.ok {
        return None;
    }
    let stderr = tick.stderr.trim();
    if stderr.is_empty() && tick.returncode == 0 {
        return None;
    }
    let tail = runner::tail(stderr, 1000);
    Some(if tail.is_empty() {
        format!("exit {}", tick.returncode)
    } else {
        format!("exit {} · {}", tick.returncode, tail)
    })
}

/// Fold a tick's outcome into the failure count, and halt if it calls for
/// one.
fn judge(s: &mut AgentState, tick: &Tick, now: i64) {
    if tick.limited {
        // A usage limit is the account's failure, not the agent's: it
        // neither counts toward the failure halt nor clears the count.
        // `once` runs a halted agent too, and a halt that waits for the
        // user must not turn into one that lifts itself.
        if s.stopped_reason.is_none() {
            s.halt(usage_limit_halt(tick.limit_resets_at, now));
        }
        return;
    }
    s.failures_in_a_row = if tick.ok { 0 } else { s.failures_in_a_row + 1 };
    if s.failures_in_a_row >= MAX_FAILURES_IN_A_ROW {
        s.halt(Halt {
            reason: format!("{} consecutive failed ticks", MAX_FAILURES_IN_A_ROW),
            until: None,
        });
    }
}

/// The halt for a tick the account's usage limit rejected. It lifts when
/// the limit resets, or after [`LIMIT_RETRY_S`] when the reset time is
/// unknown or already past.
fn usage_limit_halt(resets_at: Option<i64>, now: i64) -> Halt {
    let until = match resets_at {
        Some(t) if t > now => t + LIMIT_RESET_SLACK_S,
        _ => now + LIMIT_RETRY_S,
    };
    Halt {
        reason: "usage limit reached".into(),
        until: Some(until),
    }
}

/// Why a tick must not start now, if any. Checked by the supervisor before
/// every tick and by `once` unless forced. A spent total budget waits for
/// the user; a spent daily budget lifts when the UTC day rolls over.
pub fn budget_block(spec: &Spec, state: &AgentState) -> Option<Halt> {
    if let Some(total) = spec.run.budget_usd_total {
        if state.cost_usd >= total {
            return Some(Halt {
                reason: format!("total budget ${} reached", total),
                until: None,
            });
        }
    }
    if let Some(daily) = spec.run.daily_budget_usd {
        if state.today_cost() >= daily {
            return Some(Halt {
                reason: format!("daily budget ${} reached", daily),
                until: Some(next_utc_midnight(now_unix())),
            });
        }
    }
    None
}

pub(crate) fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        return s.to_string();
    }
    let mut out: String = s.chars().take(n.saturating_sub(1)).collect();
    out.push('…');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::harness::{spec, today_utc};
    use std::path::Path;

    #[test]
    fn budget_gates() {
        let spec = spec::parse(
            Path::new("/tmp/x"),
            "[run]\nbudget_usd_total = 1.0\ndaily_budget_usd = 0.5\n[prompt]\ninstruction=\"x\"",
        )
        .unwrap();
        let mut st = AgentState::default();
        assert!(budget_block(&spec, &st).is_none());
        st.today = today_utc();
        st.today_cost_usd = 0.6;
        let daily = budget_block(&spec, &st).unwrap();
        assert!(daily.reason.contains("daily"));
        assert!(
            daily.until.is_some_and(|t| t > now_unix()),
            "until tomorrow"
        );
        st.cost_usd = 1.2;
        let total = budget_block(&spec, &st).unwrap();
        assert!(total.reason.contains("total"));
        assert_eq!(total.until, None, "for good");
    }

    #[test]
    fn a_limited_tick_halts_until_the_reset_without_counting_as_a_failure() {
        let limited = Tick {
            limited: true,
            limit_resets_at: Some(5_000),
            ..Default::default()
        };
        let mut st = AgentState {
            failures_in_a_row: MAX_FAILURES_IN_A_ROW - 1,
            ..Default::default()
        };
        judge(&mut st, &limited, 1_000);
        assert_eq!(st.failures_in_a_row, MAX_FAILURES_IN_A_ROW - 1);
        assert_eq!(st.stopped_reason.as_deref(), Some("usage limit reached"));
        assert_eq!(st.stopped_until, Some(5_000 + LIMIT_RESET_SLACK_S));
    }

    #[test]
    fn a_limited_tick_leaves_a_halt_that_waits_for_the_user() {
        let limited = Tick {
            limited: true,
            limit_resets_at: Some(5_000),
            ..Default::default()
        };
        let mut st = AgentState::default();
        for _ in 0..MAX_FAILURES_IN_A_ROW {
            judge(&mut st, &Tick::default(), 1_000);
        }
        assert_eq!(st.stopped_until, None, "a failure halt is for good");
        judge(&mut st, &limited, 1_000);
        assert_eq!(
            st.stopped_reason.as_deref(),
            Some("5 consecutive failed ticks")
        );
        assert_eq!(st.stopped_until, None);
    }

    #[test]
    fn a_usage_limit_halt_lifts_at_the_reset_or_retries_soon() {
        let known = usage_limit_halt(Some(5_000), 1_000);
        assert_eq!(known.until, Some(5_000 + LIMIT_RESET_SLACK_S));
        // No reset time, or one already behind us: try again shortly.
        assert_eq!(
            usage_limit_halt(None, 1_000).until,
            Some(1_000 + LIMIT_RETRY_S)
        );
        assert_eq!(
            usage_limit_halt(Some(900), 1_000).until,
            Some(1_000 + LIMIT_RETRY_S)
        );
    }
}
