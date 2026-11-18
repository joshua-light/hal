//! A detached session starts in the directory it was asked for, even on a
//! tmux server whose own working directory has been deleted.
//!
//! Such a server ignores `new-session -c` and starts every pane in the dead
//! directory (seen on tmux 3.8). Own test binary: the server must be born in
//! a doomed directory, so it can't share one with the other smoke tests.

#![cfg(unix)]

mod common;

use common::{private_tmux_server, tmux_available};
use hal_lib::platform::mux;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

struct SessionGuard(String);
impl Drop for SessionGuard {
    fn drop(&mut self) {
        let _ = mux::kill_session(&self.0);
    }
}

#[test]
fn session_starts_in_its_cwd_on_a_server_whose_cwd_is_gone() {
    if !tmux_available() {
        eprintln!("session_starts_in_its_cwd: tmux not on PATH, skipping");
        return;
    }
    let _tmux = private_tmux_server();

    // The server takes the working directory of the client that starts it.
    let doomed = tempfile::tempdir().expect("doomed dir");
    let keeper = format!("hal-cwd-keeper-{}", std::process::id());
    let status = Command::new("tmux")
        .current_dir(doomed.path())
        .args(["new-session", "-d", "-s", &keeper, "sleep 60"])
        .status()
        .expect("start tmux server");
    assert!(status.success(), "could not start the tmux server");
    let _keeper_guard = SessionGuard(keeper);
    doomed.close().expect("delete the server's cwd");

    let target = tempfile::tempdir().expect("target dir");
    let want = target.path().canonicalize().expect("canonical target");
    let name = format!("hal-cwd-{}", std::process::id());
    mux::spawn_detached(&name, &want.to_string_lossy(), None).expect("spawn_detached");
    let _guard = SessionGuard(name.clone());

    let pane_cwd = || {
        let out = Command::new("tmux")
            .args(["display-message", "-p", "-t", &name, "#{pane_current_path}"])
            .output()
            .expect("display-message");
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    };
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut got = pane_cwd();
    while got != want.to_string_lossy() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(100));
        got = pane_cwd();
    }
    assert_eq!(
        got,
        want.to_string_lossy(),
        "pane started in the wrong directory"
    );
}
