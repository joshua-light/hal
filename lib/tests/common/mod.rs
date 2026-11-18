//! Shared setup for the tmux smoke tests.

use std::process::Command;
use std::sync::{Arc, Mutex, Weak};
use tempfile::TempDir;

/// The mux shim is driven through the `tmux` binary (psmux's `tmux.exe` shim
/// on Windows). Tests skip silently when it's not on `PATH` so the suite
/// stays passable on bare CI images.
pub fn tmux_available() -> bool {
    Command::new("tmux")
        .arg("-V")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Point every `tmux` this test binary runs — the library's and the test's
/// own — at a private server. Hold the returned handle for the whole test.
///
/// On the default socket, a test that runs while no server is up starts the
/// developer's real server. That server keeps `cargo test`'s environment and
/// working directory for life, and every session hal opens afterwards
/// inherits both.
///
/// `TMUX` is cleared too: inside tmux it names the outer server's socket and
/// wins over `TMUX_TMPDIR`.
///
/// The environment changes only while no test holds a handle, so parallel
/// tests never see it move under them.
pub fn private_tmux_server() -> Arc<TempDir> {
    static CURRENT: Mutex<Weak<TempDir>> = Mutex::new(Weak::new());
    let mut current = CURRENT.lock().unwrap();
    if let Some(dir) = current.upgrade() {
        return dir;
    }
    let dir = Arc::new(
        tempfile::Builder::new()
            .prefix("hal-tmux-")
            .tempdir()
            .expect("temp dir for the private tmux socket"),
    );
    std::env::set_var("TMUX_TMPDIR", dir.path());
    std::env::remove_var("TMUX");
    std::env::remove_var("TMUX_PANE");
    *current = Arc::downgrade(&dir);
    dir
}
