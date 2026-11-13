//! CLI subcommands. When argv starts with a known verb, [`dispatch`] runs it
//! before the TUI starts and returns the exit code. Parsing is hand-rolled
//! to avoid a clap dep. Each verb prints one JSON line on stdout so a calling
//! agent can parse the outcome.
//!
//! - `error`: [`CliError`] and the JSON error contract.
//! - `flags`: the shared `--flag` parser and the help-request scan.
//! - `board`, `link` (`open`), `agent`, `resource`, `usage`, `wake`,
//!   `keys`: one file per verb.
//! - `help`: `--help` text.

mod agent;
mod board;
mod error;
mod flags;
mod help;
mod keys;
mod link;
mod resource;
mod usage;
mod wake;

use error::{handle, CliError};
use flags::{args_request_help, parse_flags, require_task};
use hal_lib::ops;

pub fn dispatch(args: &[String]) -> Option<i32> {
    let (verb, rest) = args.split_first()?;
    if matches!(verb.as_str(), "help" | "--help" | "-h") {
        return Some(handle(help::print_cli_help(rest)));
    }
    if args_request_help(rest) {
        return Some(handle(help::print_cli_help(args)));
    }
    let code = match verb.as_str() {
        "open" => handle(link::open(rest)),
        "resource" => handle(resource::resource(rest)),
        "agent" => handle(agent::agent_subcommand(rest)),
        "board" => handle(board::board_subcommand(rest)),
        "usage" => handle(usage::usage(rest)),
        "keys" => handle(keys::keys(rest)),
        "wake" => handle(wake::wake(rest)),
        _ => return None,
    };
    // A verb runs before the logger starts, so stderr is the only place a
    // config the verb half-ignored can be reported.
    if let Some(problem) = hal_lib::config::load_problem() {
        eprintln!("hal: config.toml: {}", problem);
    }
    Some(code)
}

fn print_json(value: &serde_json::Value) {
    // One line per call so callers can split on \n. Pretty-print would
    // make Bash piping awkward.
    match serde_json::to_string(value) {
        Ok(s) => println!("{}", s),
        Err(e) => eprintln!("(failed to serialise output: {})", e),
    }
}

/// Map a [`PromptStatus`](ops::prompt::PromptStatus) to its JSON string,
/// printing the human warning to stderr for `Deferred`. Presentation stays
/// in the CLI, not in `ops`.
fn report_prompt_status(status: &ops::prompt::PromptStatus) -> &'static str {
    if let ops::prompt::PromptStatus::Deferred(warning) = status {
        eprintln!("warning: {}", warning);
    }
    status.as_str()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispatch_handles_help() {
        assert_eq!(dispatch(&["--help".to_string()]), Some(0));
        assert_eq!(
            dispatch(&["board".to_string(), "--help".to_string()]),
            Some(0)
        );
    }
}
