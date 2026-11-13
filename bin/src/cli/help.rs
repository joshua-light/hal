//! Help text for `hal help [topic]` and `--help`.

use super::CliError;

pub(crate) fn print_cli_help(topic: &[String]) -> Result<(), CliError> {
    let topic = topic.first().map(String::as_str);
    let text = help_text(topic).ok_or_else(|| {
        CliError::Usage(format!(
            "unknown help topic: {} (try `hal help`)",
            topic.unwrap_or_default()
        ))
    })?;
    print!("{}", text);
    Ok(())
}

fn help_text(topic: Option<&str>) -> Option<&'static str> {
    Some(match topic {
        None => GENERAL_HELP,
        Some("open") => OPEN_HELP,
        Some("agent") => AGENT_HELP,
        Some("board") => BOARD_HELP,
        Some("resource") => RESOURCE_HELP,
        Some("usage") => USAGE_HELP,
        Some("keys") => KEYS_HELP,
        Some("wake") => WAKE_HELP,
        Some(_) => return None,
    })
}

const GENERAL_HELP: &str = r#"hal

Usage:
  hal                         Start the TUI
  hal --no-tui                Print discovered sessions
  hal help [topic]            Show CLI help
  hal keys [ide|vim]          List every key and the name to rebind it by

Desktop-facing topics:
  open              Act on a hal:// deep link (a PR review or fix, a board task)

Persistent agents (Agents tab):
  agent             Scaffold, run, poke, pause and inspect persistent agents
  wake              Say a named thing happened, so agents watching it poll now

Tasks board:
  board             Add a card to the personal Tasks board

Accounts and task sessions:
  resource          Account capacity and the managed session working each task
  usage             Quota of a Claude account, from the shared store

Examples:
  hal board add --text "Fix the flaky test" --priority p1
  hal open "hal://task?id=tk-123&dir=$PWD"
"#;

const KEYS_HELP: &str = r#"hal keys [ide|vim]

Print every place that takes keys, the actions in it and the keys each has
now, as TOML. The scheme defaults to [ui].keys; [keymap] overrides from
~/.hal/config.toml are applied, and its mistakes are listed on stderr.

To rebind an action, copy its line and header into config.toml and change
the keys:

  [keymap.sessions]
  new_in = ["ctrl+p", "o"]
  shell = []                  # no key

Keys: a character (P is Shift+P), enter, esc, tab, space, bksp, del, ins,
up, down, left, right, pgup, pgdn, home, end, f1-f12, with ctrl+, alt+ or
shift+ in front. cmd+ means ctrl+. Restart hal to load the change.
"#;

const USAGE_HELP: &str = r#"hal usage [--home DIR] [--refresh]

Print one JSON line with the account's quota windows and the store's health.
Every consumer (statusline, TUI, resource broker) reads through this store, so
the usage endpoint sees one request per account per minute from this machine.

  --home DIR     Claude config dir of the account; default CLAUDE_CONFIG_DIR
                 or ~/.claude
  --refresh      Ask again even if the last reading is younger than the TTL.
                 A pending retry window is still honoured.

Fields: health (ready|throttled|login_required|unavailable), five_hour and
seven_day (utilization, resets_at) from the last good reading or null, age_s
of that reading, retry_at when the endpoint may be asked again, error, and
token_expires_at: when the stored access token lapses. A lapsed token reads
as login_required until any claude run for that home refreshes it.
"#;

const RESOURCE_HELP: &str = r#"hal resource

  accounts [--refresh]                      Show account health and quota windows
  list                                      Resources, who holds each, and the queue
  claim NAME... [--wait SECONDS]            (in a session) the complete set the work needs
  release [NAME...] [--as NAME]             (in a session) hand back, or stop waiting
                                            for, some or all of it
  select --kind KIND --role ROLE            Preview a capacity-aware allocation
  start --task ID --kind KIND --role ROLE --cwd DIR --prompt TEXT [--title TEXT]
                                            Start the session working a task; a task
                                            with a live session in another role is
                                            handed over (new one starts, old one stops).
                                            --title names the session; default: the
                                            card's title
  status [--worker ID]                      Workers, their accounts and generations
  stop --worker ID [--reason TEXT]          End a worker's session
  retry --worker ID                         Requeue a blocked worker
  supervise                                 Refresh quota and reconcile workers once
  hook                                      PreToolUse: records the transcript, notes quota

Configuration: ~/.hal/resources.toml. Python 3.11+ and tmux required.
One worker holds a task at a time. When its account reaches stop_percent the
supervisor stops it and continues it on another account from its transcript.
Default warning/start/stop ceilings are 80%/85%/95%.

A resource is a thing only one session may use at a time: a checkout, a phone.
[hosts.*] says where commands run, [resources.*] what lives on each host, and
routing says nothing about them — a session claims what its work needs while
it runs and is queued, first to ask first served, when it is taken. Holding is
derived from live workers, so a session that forgets to release holds nothing
once it ends.
"#;

const OPEN_HELP: &str = r#"hal open

Usage:
  hal open <hal://url> [options]

Links:
  hal://review?depth=<light|full>&pr=<pull request url>[&title=<text>]
      Spawn a session in the local checkout of the pull request's repository,
      name it "Review: <title>" (or "Review: <repo>#<n>"), and open it with
      "Let's do <depth> review of this PR: <url>". The checkout is found by
      repo name among bookmarks, the cwds of known sessions, and then the
      folders directly inside a bookmark; without one the review runs from
      home, off the pull request alone.
      An optional &post=<1-100> lets the review post findings it is at least
      that confident about without asking.

  hal://fix?pr=<pull request url>[&title=<text>][&kind=<word>]
      File the fix as a Tasks-board card — "Fix: <title>" (or "Fix:
      <repo>#<n>"), with the brief as its first note and <kind> as its kind
      (one of [tasks].kinds) — then spawn its session in the same checkout
      and open it with the standing orders for working through the pull
      request's review comments: switch to its branch, address every
      comment, track each as a Bitbucket task and close it once the fix is
      pushed, answer questions, ask when a comment is ambiguous, skip what is
      already done, sign every reply as Claude/Codex, and finish the card
      with `hal board done --text "Pushed: …"` at the end. With subscription accounts configured the
      session is started through the resource broker (`hal resource
      start`, role implementation, kind <word> or basic), so it runs on
      whichever account has room. The card reaches Running once its session
      is bound; it never waits in Planning, because the comments are the plan.

  hal://merge-target?pr=<pull request url>[&title=<text>][&kind=<word>]
      Filed, started and routed exactly like a fix, as a "Merge Target:
      <title>" card, with the standing orders for bringing the target branch
      in: switch to the pull request's branch, fetch, merge its target branch
      into it (no rebase, no force-push), resolve conflicts keeping both
      sides' intent, ask when one needs the author, check the result builds
      where it can, push, and finish the card with `hal board done`.

  hal://task?id=<tk-…>[&dir=<path>][&kind=<word>][&role=<word>]
      Spawn a session for one Tasks-board card in <dir> (default: the card's
      own recorded cwd), name it "Task: <card>", open it with "/task --task
      <id> [--kind <word>] [--role <word>] <card text>", and bind the card to
      that session so `f` attaches to it. With a role the link is a hand-over:
      a fresh session starts and the card's previous one is closed once this
      command has reported. A hand-over needs a note on the card (the brief
      the next session works from; see `hal help board`) and is refused
      without one. The card's status is left alone.

Options:
  --agent AGENT        Backend (default: [projects].default_session_agent)
  --wait-secs N        Prompt-dispatch readiness timeout (default: 120)
  --dry-run            Resolve cwd/prompt/agent, spawn nothing
  --notify             On failure, also post the error as a desktop
                       notification (for callers nobody watches, like the
                       URL-scheme handler)

Emits one JSON line with kind/tmux/cwd/agent_id/prompt/prompt_status.
hal ships no URL-scheme handler. Register one locally with your OS; it must
call `hal open <url>`.
"#;

const WAKE_HELP: &str = r#"hal wake <name>

Say a named thing happened. Every agent whose agent.toml lists the name in
trigger.wake runs its poll (or interval tick) on the next second instead of
at the end of interval_s:

  [trigger]
  kind = "poll"
  wake = ["board"]

The board wakes `board` on every card status change. Use this verb when a
script changes something the hub cannot see. A wake carries no payload: the
agent's poll still decides what the event is, so a lost wake costs only
latency. trigger.wake needs kind = "poll" or "interval".

The name uses letters, digits, '-' and '_' (at most 64). The wake is a stamp
at ~/.hal/wake/<name>. Emits {"ok":true,"wake":"<name>"}.
"#;

const AGENT_HELP: &str = r#"hal agent

Persistent agents: a directory under ~/.hal/agents/<name>/ with one
agent.toml. The TUI supervises every enabled agent: an event (a file in its
inbox, a poll command's stdout, or an interval) becomes one bounded
`claude -p` tick. Agents report back with `hal agent note`. A spec may also name wakes
(`trigger.wake = ["board"]`): when one happens the poll runs on the next
second instead of at the end of interval_s, so interval_s is a floor under
the latency and not the latency. The board wakes `board` on every card
status change; anything else says so with `hal wake <name>`.

Usage:
  hal agent list [--json]
  hal agent new <name> [--from DIR]        Scaffold <name>/agent.toml (+ work/, inbox/)
  hal agent once <name> [--event TEXT | --event-file F] [--force]
                                              Run one tick now, print the outcome
  hal agent poke <name> [--event TEXT | --event-file F]
                                              Drop an event into the inbox; without one, run now
                                              the way the trigger would (a poll agent polls)
  hal agent pause <name>                   Skip this agent until resumed
  hal agent resume <name>                  Resume; also clears a halt
  hal agent reset <name>                   Clear ticks/spend bookkeeping (workdir untouched)
  hal agent show <name>                    Full state as JSON (history, notes)

Agent-facing (inside a tick, HAL_AGENT is set):
  hal agent note --text TEXT [--level info|warn] [--ref URL]

Layout of ~/.hal/agents/<name>/:
  agent.toml   spec        work/   the agent's world     state.json  bookkeeping
  inbox/       events      notes.jsonl  outbox           log/        stream-json per tick
  events.jsonl harness log: runs, poll failures, halts, edits made in the hub
"#;

const BOARD_HELP: &str = r#"hal board

The personal Tasks board (To-Do · In Progress · Review · Done), one card per
directory under ~/.hal/tasks/.

Usage:
  hal board add --text TEXT [--title TEXT] [--tags "a b"] [--priority p1|p2|p3|p4]
                   [--kind WORD]
      Mint a card in To-Do, exactly as the `a` key does in the TUI. Nothing
      is spawned and no status changes. Emits {"ok":true,"task_id":"tk-…"}.
      --kind is the deliverable the task router places the card by, and must
      be one of [tasks].kinds — the same list the board's `T` picker offers.
  hal board note --task ID [--text TEXT]
      Append a note to the card: --text, or stdin when there is none (a
      heredoc for a long brief). The same `note` attachment the `p` key
      pastes; it shows on the card. Emits
      {"ok":true,"note":{…},"count":N,"status":"…"}.
      A note identical to the card's newest one is refused (exit 2): the
      record does not say the same thing twice in a row.
      A note that opens with `PR:` moves the card to Review — the column for
      work that wants reading, apart from work that wants an answer — which
      is why the status comes back with the note.
  hal board notes --task ID [--json]
      The card's notes in attach order, each under a dated rule.
  hal board done --task ID [--text NOTE]
      Finish the card from the session that did its work: --text becomes its
      closing note, the card moves to Done, and the broker workers on it are
      stopped, so the session ends a few seconds later. A card that still
      waits on you (`Waiting:`, `Needs you:`) is refused; a closing note is
      what answers it. Emits {"ok":true,"status":"done","stopped":[…]}.

A card is how a script hands the user something to look at: mint it here,
then open it with `hal open "hal://task?id=<tk-…>&dir=<path>"`, which
only ever addresses a card that already exists (see `hal help open`).

The notes are a task's record. The `task` skill writes the brief it agreed
with the user as one note, the branch it built as another, and what
verification found as a third; a session taking the card over reads them
first. A hand-over link (`&role=…`) is refused for a card with no note.
"#;

#[cfg(test)]
mod tests {
    use super::*;

    /// The topics `hal help` lists: two-space-indented lines that do not
    /// start with `hal` (usage lines and examples do).
    fn listed_topics() -> Vec<&'static str> {
        GENERAL_HELP
            .lines()
            .filter_map(|line| line.strip_prefix("  "))
            .filter(|rest| !rest.starts_with(' '))
            .filter_map(|rest| rest.split_whitespace().next())
            .filter(|word| *word != "hal")
            .collect()
    }

    #[test]
    fn every_listed_topic_has_help() {
        let topics = listed_topics();
        assert!(topics.contains(&"wake"), "{topics:?}");
        for topic in topics {
            assert!(help_text(Some(topic)).is_some(), "no help for {topic}");
        }
    }

    #[test]
    fn an_unknown_topic_is_a_usage_error() {
        let err = print_cli_help(&["nope".to_string()]).unwrap_err();
        assert!(matches!(err, CliError::Usage(_)), "{err:?}");
    }
}
