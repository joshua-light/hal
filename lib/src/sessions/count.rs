//! Counts of Claude Code session JSONLs whose files were created today or
//! this calendar week (ISO, Monday-anchored). Walks each Claude home's
//! `projects/` and reads each JSONL's `created()` metadata — pure file-system
//! work, no parsing.

use chrono::{Datelike, Local, NaiveDate, TimeZone};
use std::fs;
use std::time::SystemTime;

#[derive(Debug, Clone, Copy, Default)]
pub struct SessionCounts {
    pub today: u32,
    pub week: u32,
}

/// Counts across the default Claude home and every Claude account home, the
/// roots the live scan walks.
pub fn count_recent_sessions() -> SessionCounts {
    let now = Local::now();
    let today: NaiveDate = now.date_naive();
    let week_start: NaiveDate =
        today - chrono::Duration::days(today.weekday().num_days_from_monday() as i64);

    // Skip the titler's scratch project dir: its JSONLs are one-shot
    // `claude -p` runs hal itself spawns, not real sessions — the same
    // exclusion the Claude scanner applies. Sharing the predicate keeps both
    // aligned.
    let scratch_proj_dir = crate::sessions::scanner::scratch_project_dir_name();

    let mut counts = SessionCounts::default();
    let projects = crate::resources::claude_roots()
        .into_iter()
        .filter_map(|home| fs::read_dir(home.join("projects")).ok())
        .flatten();
    for proj in projects.flatten() {
        if let Some(skip) = scratch_proj_dir.as_deref() {
            if proj.file_name().to_str() == Some(skip) {
                continue;
            }
        }
        let Ok(files) = fs::read_dir(proj.path()) else {
            continue;
        };
        for f in files.flatten() {
            let path = f.path();
            if path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
                continue;
            }
            let Ok(meta) = f.metadata() else { continue };
            // `created()` returns birthtime on macOS / creationTime on
            // Windows; on Linux it may be `Err(Unsupported)`, in which
            // case we fall back to mtime.
            let created: SystemTime = meta
                .created()
                .or_else(|_| meta.modified())
                .unwrap_or(now.into());
            let Ok(dur) = created.duration_since(SystemTime::UNIX_EPOCH) else {
                continue;
            };
            let Some(local_dt) = Local.timestamp_opt(dur.as_secs() as i64, 0).single() else {
                continue;
            };
            let date = local_dt.date_naive();
            if date == today {
                counts.today += 1;
            }
            if date >= week_start {
                counts.week += 1;
            }
        }
    }
    counts
}

// `$HOME` redirection only isolates `dirs::home_dir()` on unix; on Windows it
// resolves via the profile API and ignores `$HOME`, so gate the test there.
#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::platform::paths;
    use crate::test_util::with_temp_home;

    #[test]
    fn sessions_under_an_extra_account_home_are_counted() {
        with_temp_home(|| {
            let home = dirs::home_dir().unwrap();
            fs::create_dir_all(home.join(".hal")).unwrap();
            fs::write(
                home.join(".hal/resources.toml"),
                "[accounts.work]\nprovider = \"claude\"\nhome = \"~/.claude-work\"\n",
            )
            .unwrap();
            for projects in [".claude/projects/-p", ".claude-work/projects/-p"] {
                fs::create_dir_all(home.join(projects)).unwrap();
                fs::write(home.join(projects).join("a.jsonl"), "{}\n").unwrap();
            }

            let counts = count_recent_sessions();

            assert_eq!(counts.today, 2);
            assert_eq!(counts.week, 2);
        });
    }

    #[test]
    fn scratch_summary_sessions_are_not_counted() {
        with_temp_home(|| {
            let projects = paths::claude_home().expect("claude_home").join("projects");
            let real = projects.join("-home-me-proj");
            // The scratch project dir is exactly what the Claude scanner skips.
            let scratch = projects.join(
                crate::sessions::scanner::scratch_project_dir_name()
                    .expect("scratch project dir name"),
            );
            fs::create_dir_all(&real).unwrap();
            fs::create_dir_all(&scratch).unwrap();
            // Both JSONLs are created now, so both land in today's window.
            fs::write(real.join("a.jsonl"), "{}\n").unwrap();
            fs::write(scratch.join("b.jsonl"), "{}\n").unwrap();

            let counts = count_recent_sessions();

            // Only the real session is counted; the scratch one is excluded.
            assert_eq!(counts.today, 1);
            assert_eq!(counts.week, 1);
        });
    }
}
