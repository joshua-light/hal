//! Helpers shared by the Claude, Pi and Codex session scanners.

use crate::agent::{AgentConfig, AgentKind};
use crate::models::SessionState;
use crate::sessions::dir_cache::FileEntry;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// The card's project label: the cwd's last component, or `"unknown"`.
pub(super) fn project_name(cwd: &str) -> String {
    Path::new(cwd)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string()
}

/// The first configured agent of `kind`; `None` disables that backend's scan.
pub(super) fn default_agent(agents: &[AgentConfig], kind: AgentKind) -> Option<AgentConfig> {
    agents.iter().find(|a| a.kind == kind).cloned()
}

/// Combine the transcript-derived state with the scanner's own verdict. A dead
/// process forces Inactive; a caller that already knows the turn is running
/// forces Processing. For every other hint the transcript state stands.
pub(super) fn apply_state_hint(parsed: SessionState, hint: SessionState) -> SessionState {
    match hint {
        SessionState::Inactive | SessionState::Processing => hint,
        SessionState::Idle
        | SessionState::WaitingForInput
        | SessionState::Question
        | SessionState::Starting => parsed,
    }
}

/// The transcripts in `files` not in `claimed` and modified within
/// `window_secs`, newest first. A live transcript's mtime can lead the wall
/// clock, so a future mtime counts as age 0, as in the Codex scan.
pub(super) fn recent_unclaimed(
    files: &[FileEntry],
    claimed: &HashSet<PathBuf>,
    window_secs: u64,
) -> Vec<PathBuf> {
    let mut candidates: Vec<&FileEntry> = Vec::new();
    for entry @ (path, mtime) in files {
        if claimed.contains(path) {
            continue;
        }
        let age = mtime.elapsed().map_or(0, |d| d.as_secs());
        if age > window_secs {
            continue;
        }
        candidates.push(entry);
    }
    candidates.sort_by_key(|b| std::cmp::Reverse(b.1));
    candidates
        .into_iter()
        .map(|(path, _)| path.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, SystemTime};

    #[test]
    fn a_future_mtime_counts_as_fresh() {
        let now = SystemTime::now();
        let future = PathBuf::from("future.jsonl");
        let old = PathBuf::from("old.jsonl");
        let files = vec![
            (old.clone(), now - Duration::from_secs(7_200)),
            (future.clone(), now + Duration::from_secs(60)),
        ];
        let got = recent_unclaimed(&files, &HashSet::new(), 3_600);
        assert_eq!(got, vec![future]);
    }
}
