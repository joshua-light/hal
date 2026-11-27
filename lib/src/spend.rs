//! Money Claude Code cost on this machine this week.
//!
//! Nothing here counts tokens: `ccusage` prices the transcripts Claude Code
//! leaves in its default home, and [`read`] keeps its last answer in
//! `~/.hal/spend.json` so the TUI and `hal spend` share one run per TTL. A
//! failed run never erases this week's last good answer.
//!
//! The week is the calendar one, Monday to Sunday in local time, so the number
//! starts from zero every Monday morning. [`reset`] starts it from zero again
//! in the middle of one: what the week had cost by then is kept beside the
//! price and left out of the count until the next Monday.

use crate::config;
use crate::platform::paths;
use crate::usage::now;
use chrono::{Datelike, Duration, Local, NaiveDate};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// What one week of Claude Code cost so far.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Spend {
    /// The Monday the week starts on.
    pub week: NaiveDate,
    /// Everything `ccusage` priced since Monday.
    #[serde(alias = "usd")]
    pub priced_usd: f64,
    /// What the week had cost at its last [`reset`]; zero in a week never reset.
    #[serde(default)]
    pub reset_usd: f64,
    pub observed_at: u64,
}

impl Spend {
    /// What counts against the budget: the week's price since its last reset.
    pub fn usd(&self) -> f64 {
        (self.priced_usd - self.reset_usd).max(0.0)
    }

    /// How much of `budget_usd` is gone, in percent. Not capped: a week over
    /// budget reads above 100.
    pub fn percent_of(&self, budget_usd: f64) -> u32 {
        if budget_usd <= 0.0 {
            return 0;
        }
        (self.usd() / budget_usd * 100.0).round() as u32
    }

    /// The `hal spend` contract: what a script gets on stdout.
    pub fn report(&self) -> serde_json::Value {
        let budget_usd = config::get().spend.weekly_budget_usd;
        serde_json::json!({
            "ok": true,
            "week": self.week,
            "usd": self.usd(),
            "reset_usd": self.reset_usd,
            "budget_usd": budget_usd,
            "percent": self.percent_of(budget_usd),
            "observed_at": self.observed_at,
            "age_s": now().saturating_sub(self.observed_at),
        })
    }
}

/// This week's spend, priced again first when the stored answer is stale.
pub fn read() -> Result<Spend, String> {
    sync(false)
}

/// Like [`read`], but the TTL is ignored.
pub fn refresh() -> Result<Spend, String> {
    sync(true)
}

/// Counts the week from this moment: what it cost so far stops counting.
/// Needs a pricing run of its own, since a stale price would leave the money
/// spent after it on the bar.
pub fn reset() -> Result<Spend, String> {
    let store = Store::open().ok_or("cannot open the hal home")?;
    let mut spend = priced(week_of(Local::now().date_naive()))?;
    spend.reset_usd = spend.priced_usd;
    store.save(&spend);
    Ok(spend)
}

fn sync(force: bool) -> Result<Spend, String> {
    let week = week_of(Local::now().date_naive());
    let store = Store::open();
    let stored = store
        .as_ref()
        .and_then(Store::load)
        .filter(|spend| spend.week == week);
    let ttl = config::get().spend.refresh_interval().as_secs();
    let fresh = stored
        .clone()
        .filter(|spend| !force && now().saturating_sub(spend.observed_at) < ttl);
    if let Some(spend) = fresh {
        return Ok(spend);
    }
    match priced(week) {
        Ok(mut spend) => {
            if let Some(store) = &store {
                // Read again: a reset may have landed while ccusage ran.
                spend.reset_usd = store.reset_usd(week);
                store.save(&spend);
            }
            Ok(spend)
        }
        Err(error) => stored.ok_or(error),
    }
}

fn week_of(day: NaiveDate) -> NaiveDate {
    day - Duration::days(day.weekday().num_days_from_monday() as i64)
}

fn priced(week: NaiveDate) -> Result<Spend, String> {
    let since = week.format("%Y%m%d").to_string();
    let output = Command::new("ccusage")
        .args(["claude", "weekly", "--json", "--since", &since])
        .output()
        .map_err(|e| format!("cannot run ccusage: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let reason = stderr.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
        return Err(format!(
            "ccusage failed (20+ is required): {}",
            reason.trim()
        ));
    }
    let report: Report = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("unexpected ccusage output (20+ is required): {e}"))?;
    Ok(Spend {
        week,
        priced_usd: report.usd(),
        reset_usd: 0.0,
        observed_at: now(),
    })
}

/// `ccusage claude weekly --json`, as far as money goes. Its rows are cut on
/// its own week boundary, so the days since Monday may span two of them.
#[derive(Deserialize)]
struct Report {
    weekly: Vec<Row>,
}

#[derive(Deserialize)]
struct Row {
    #[serde(rename = "totalCost")]
    total_cost: f64,
}

impl Report {
    fn usd(&self) -> f64 {
        self.weekly.iter().map(|row| row.total_cost).sum()
    }
}

struct Store {
    file: PathBuf,
}

impl Store {
    fn open() -> Option<Self> {
        let dir = paths::hal_home()?;
        fs::create_dir_all(&dir).ok()?;
        Some(Self {
            file: dir.join("spend.json"),
        })
    }

    fn load(&self) -> Option<Spend> {
        serde_json::from_str(&fs::read_to_string(&self.file).ok()?).ok()
    }

    fn reset_usd(&self, week: NaiveDate) -> f64 {
        self.load()
            .filter(|spend| spend.week == week)
            .map_or(0.0, |spend| spend.reset_usd)
    }

    fn save(&self, spend: &Spend) {
        let tmp = self
            .file
            .with_extension(format!("tmp.{}", std::process::id()));
        let Ok(body) = serde_json::to_vec_pretty(spend) else {
            return;
        };
        if fs::write(&tmp, body).is_ok() && fs::rename(&tmp, &self.file).is_err() {
            let _ = fs::remove_file(&tmp);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const REPORT: &str = r#"{"totals":{"totalCost":354.58},"weekly":[
        {"week":"2026-11-15","totalCost":272.76,"totalTokens":313729374},
        {"week":"2026-11-22","totalCost":81.82,"totalTokens":449052447}]}"#;

    fn spend(report: &str) -> Spend {
        let report: Report = serde_json::from_str(report).unwrap();
        Spend {
            week: NaiveDate::from_ymd_opt(2026, 11, 16).unwrap(),
            priced_usd: report.usd(),
            reset_usd: 0.0,
            observed_at: 0,
        }
    }

    #[test]
    fn line_keeps_counting_dollars_past_a_full_bar() {
        let line = crate::ui::build_spend_line(&spend(REPORT), 200.0);

        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        assert_eq!(text, "spend ━━━━━━━━━━ $355/200 ");
    }

    #[test]
    fn rows_of_one_week_add_up() {
        assert!((spend(REPORT).usd() - 354.58).abs() < 1e-9);
    }

    #[test]
    fn week_without_usage_costs_nothing() {
        let spend = spend(r#"{"weekly":[]}"#);

        assert_eq!(spend.usd(), 0.0);
        assert_eq!(spend.percent_of(200.0), 0);
    }

    #[test]
    fn percent_is_not_capped_at_the_budget() {
        assert_eq!(spend(REPORT).percent_of(200.0), 177);
    }

    #[test]
    fn reset_week_counts_only_what_came_after() {
        let mut spend = spend(REPORT);
        spend.reset_usd = 300.0;

        assert!((spend.usd() - 54.58).abs() < 1e-9);
        assert_eq!(spend.percent_of(200.0), 27);
    }

    #[test]
    fn week_stored_before_resets_existed_was_never_reset() {
        let stored = r#"{"week":"2026-11-23","usd":22.37,"observed_at":1}"#;

        let spend: Spend = serde_json::from_str(stored).unwrap();

        assert_eq!(spend.usd(), 22.37);
    }

    #[test]
    fn report_without_rows_is_refused() {
        assert!(serde_json::from_str::<Report>(r#"{"error":"no data"}"#).is_err());
    }

    #[test]
    fn week_starts_on_monday() {
        let monday = NaiveDate::from_ymd_opt(2026, 11, 16).unwrap();
        let tuesday = NaiveDate::from_ymd_opt(2026, 11, 17).unwrap();
        let sunday = NaiveDate::from_ymd_opt(2026, 11, 22).unwrap();

        assert_eq!(week_of(monday), monday);
        assert_eq!(week_of(tuesday), monday);
        assert_eq!(week_of(sunday), monday);
    }
}
