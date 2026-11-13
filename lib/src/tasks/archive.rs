use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::store::TaskState;
use crate::persist::lock_exclusive;
use crate::platform::paths::hal_home;

fn archive_path() -> Option<PathBuf> {
    hal_home().map(|h| h.join("tasks-archive-v2.json"))
}

/// Append removed tasks to `~/.hal/tasks-archive-v2.json` (a bare JSON
/// array of unified [`TaskState`]s), so `x` and `c` are recoverable beyond
/// the in-session undo slot. The archive is a log, not a ledger: an undone
/// delete leaves its copy behind. A corrupt archive is moved aside, not
/// overwritten, and a fresh one started: a bad log must not block deletes.
pub(super) fn archive_tasks(items: &[TaskState]) -> io::Result<()> {
    if items.is_empty() {
        return Ok(());
    }
    let Some(path) = archive_path() else {
        return Ok(());
    };
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::other("task archive path has no parent"))?;
    fs::create_dir_all(parent)?;
    let _lock = lock_exclusive(&parent.join("tasks-archive.lock"))?;
    let mut archived: Vec<TaskState> = match fs::read_to_string(&path) {
        Ok(raw) => match serde_json::from_str(&raw) {
            Ok(archived) => archived,
            Err(e) => {
                set_aside(&path, &e)?;
                Vec::new()
            }
        },
        Err(e) if e.kind() == io::ErrorKind::NotFound => Vec::new(),
        Err(e) => return Err(e),
    };
    archived.extend(items.iter().cloned());
    crate::persist::save_json(&path, &archived)
}

/// Rename the unparseable archive at `path` to `<name>.corrupt-<unix-ts>`,
/// keeping its bytes for a manual rescue.
fn set_aside(path: &Path, why: &serde_json::Error) -> io::Result<()> {
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    let aside = path.with_file_name(format!(
        "{}.corrupt-{}",
        name,
        crate::persist::now_unix_secs()
    ));
    fs::rename(path, &aside).map_err(|e| {
        io::Error::new(
            e.kind(),
            format!(
                "task archive {} is corrupt ({}) and could not be moved aside: {}",
                path.display(),
                why,
                e
            ),
        )
    })?;
    log::warn!(
        "task archive {} is corrupt ({}); moved it to {} and started a new one",
        path.display(),
        why,
        aside.display()
    );
    Ok(())
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::tasks::store::TaskStatus;
    use crate::tasks::PersonalBoard;
    use crate::test_util::with_temp_home;

    #[test]
    fn removals_land_in_archive_and_restore_reinserts() {
        with_temp_home(|| {
            let mut b = PersonalBoard::load();
            let a = b.add("deleted one").unwrap().unwrap();
            let d = b.add("done one").unwrap().unwrap();
            b.set_status(&d, TaskStatus::Done).unwrap();
            let removed = b.remove(&a).unwrap().unwrap();
            b.clear_done().unwrap();
            // Both removal paths append to the archive file.
            let raw = fs::read_to_string(archive_path().unwrap()).unwrap();
            let archived: Vec<TaskState> = serde_json::from_str(&raw).unwrap();
            assert_eq!(
                archived
                    .iter()
                    .map(|t| t.task_id.as_str())
                    .collect::<Vec<_>>(),
                vec![a.as_str(), d.as_str()]
            );
            // Undo restores the task (with its id); a second restore of the
            // same id is refused.
            assert!(b.restore(removed.clone()).unwrap());
            assert!(!b.restore(removed).unwrap());
            assert_eq!(PersonalBoard::load().get(&a).unwrap().prompt, "deleted one");
        });
    }

    #[test]
    fn a_corrupt_archive_is_moved_aside_and_the_delete_goes_through() {
        with_temp_home(|| {
            let path = archive_path().unwrap();
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, "not json").unwrap();

            let mut b = PersonalBoard::load();
            let a = b.add("doomed").unwrap().unwrap();
            b.remove(&a).unwrap().unwrap();
            assert!(PersonalBoard::load().get(&a).is_none());

            let raw = fs::read_to_string(&path).unwrap();
            let archived: Vec<TaskState> = serde_json::from_str(&raw).unwrap();
            assert_eq!(archived.len(), 1);
            assert_eq!(archived[0].task_id, a);

            let asides: Vec<_> = fs::read_dir(path.parent().unwrap())
                .unwrap()
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.file_name()
                        .to_string_lossy()
                        .starts_with("tasks-archive-v2.json.corrupt-")
                })
                .collect();
            assert_eq!(asides.len(), 1);
            assert_eq!(fs::read_to_string(asides[0].path()).unwrap(), "not json");
        });
    }
}
