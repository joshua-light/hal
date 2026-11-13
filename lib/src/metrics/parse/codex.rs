use super::{project_of, AssistantCall, ParsedSession, ToolUse};
use crate::conversation::parse_timestamp_ms;
use crate::metrics::cost::Tokens;
use serde_json::Value;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Codex rollouts report token usage as a running `total_token_usage` in
/// repeated `token_count` events, not per-message `usage` blocks — so summing
/// per-turn deltas would multi-count. We take the FINAL cumulative total as a
/// single synthetic call: session/project token and cost totals come out
/// correct, at the cost of per-turn context-growth granularity (codex sessions
/// simply won't surface in the growth findings).
///
/// Interruptions follow the rollout's own signals. Codex answers an Esc'd
/// tool call with an output that begins "aborted", so a call counts as
/// interrupted when that is its output, or when its turn ended
/// (`turn_aborted`, or the next turn starting) before any output arrived.
/// Calls still open at the end of the file are in flight, not interrupted.
pub(super) fn parse_codex_session_file(path: &Path) -> Option<ParsedSession> {
    let file = File::open(path).ok()?;
    let reader = BufReader::new(file);

    let mut session_id = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let mut cwd: Option<String> = None;
    let mut end_time_ms = 0u64;
    let mut model = String::new();
    let mut total_usage: Option<Value> = None;
    let mut tool_uses: Vec<ToolUse> = Vec::new();
    let mut tool_result_ids: HashSet<String> = HashSet::new();
    // Calls of the current turn that have no output yet.
    let mut open: HashSet<String> = HashSet::new();

    for line in reader.lines() {
        let Ok(line) = line else { continue };
        if line.trim().is_empty() {
            continue;
        }
        let Ok(v): Result<Value, _> = serde_json::from_str(&line) else {
            continue;
        };
        if let Some(ts) = v.get("timestamp").and_then(parse_timestamp_ms) {
            end_time_ms = end_time_ms.max(ts);
        }
        let rec_type = v.get("type").and_then(|t| t.as_str()).unwrap_or("");
        let payload = v.get("payload");
        let payload_type = payload
            .and_then(|p| p.get("type"))
            .and_then(|t| t.as_str())
            .unwrap_or("");
        match (rec_type, payload_type) {
            ("session_meta", _) => {
                if let Some(p) = payload {
                    if let Some(id) = p.get("session_id").and_then(|x| x.as_str()) {
                        session_id = id.to_string();
                    }
                    if let Some(c) = p.get("cwd").and_then(|x| x.as_str()) {
                        cwd = Some(c.to_string());
                    }
                }
            }
            ("turn_context", _) => {
                if let Some(m) = payload
                    .and_then(|p| p.get("model"))
                    .and_then(|m| m.as_str())
                {
                    model = m.to_string();
                }
            }
            ("event_msg", "token_count") => {
                if let Some(t) = payload
                    .and_then(|p| p.get("info"))
                    .and_then(|i| i.get("total_token_usage"))
                {
                    total_usage = Some(t.clone());
                }
            }
            ("response_item", "function_call" | "custom_tool_call") => {
                if let Some(p) = payload {
                    let name = p.get("name").and_then(|n| n.as_str()).unwrap_or("");
                    if name.is_empty() {
                        continue;
                    }
                    let id = p
                        .get("call_id")
                        .or_else(|| p.get("id"))
                        .and_then(|i| i.as_str())
                        .unwrap_or("");
                    if !id.is_empty() {
                        open.insert(id.to_string());
                    }
                    tool_uses.push(ToolUse {
                        name: name.to_string(),
                        id: id.to_string(),
                        bash_commands: Vec::new(),
                    });
                }
            }
            ("response_item", "function_call_output" | "custom_tool_call_output") => {
                if let Some(id) = payload
                    .and_then(|p| p.get("call_id"))
                    .and_then(|i| i.as_str())
                {
                    open.remove(id);
                    if !output_is_aborted(payload.and_then(|p| p.get("output"))) {
                        tool_result_ids.insert(id.to_string());
                    }
                }
            }
            // The turn is over: whatever it left open never got an output.
            ("event_msg", "turn_aborted" | "task_complete" | "task_started" | "user_message") => {
                open.clear();
            }
            _ => {}
        }
    }

    let usage = total_usage?;
    let f = |k: &str| usage.get(k).and_then(|v| v.as_u64()).unwrap_or(0);
    let cached = f("cached_input_tokens");
    let tokens = Tokens {
        input: f("input_tokens").saturating_sub(cached),
        // Reasoning is a subset of output_tokens, not additional usage.
        output: f("output_tokens"),
        cache_read: cached,
        cache_creation: 0,
    };
    if tokens.total() == 0 && tool_uses.is_empty() {
        return None;
    }

    let call = AssistantCall {
        model,
        tokens,
        timestamp_ms: end_time_ms,
        tool_uses,
        cost_override: None,
        // Codex resume copies prior history into a new rollout, but we have no
        // per-message id to dedup on; leave empty (same as Pi).
        dedup_key: String::new(),
        folded: true,
    };

    Some(ParsedSession {
        session_id,
        project: project_of(cwd.as_deref()),
        cwd: cwd.unwrap_or_default(),
        jsonl_path: path.to_path_buf(),
        is_subagent: false,
        calls: vec![call],
        tool_result_ids,
        in_flight_tool_use_ids: open,
    })
}

/// Whether a tool output is Codex's "aborted by user after 40.8s" notice.
/// The output is a string, or a list of text parts.
fn output_is_aborted(output: Option<&Value>) -> bool {
    let text = match output {
        Some(Value::String(s)) => Some(s.as_str()),
        Some(Value::Array(parts)) => parts
            .first()
            .and_then(|p| p.get("text"))
            .and_then(|t| t.as_str()),
        _ => None,
    };
    text.is_some_and(|t| t.trim_start().starts_with("aborted"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn parse(lines: &[Value]) -> ParsedSession {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("rollout.jsonl");
        let mut body: Vec<String> = lines.iter().map(Value::to_string).collect();
        body.push(
            json!({"type": "event_msg", "payload": {"type": "token_count",
                "info": {"total_token_usage": {"input_tokens": 10, "output_tokens": 5}}}})
            .to_string(),
        );
        std::fs::write(&path, body.join("\n")).unwrap();
        parse_codex_session_file(&path).expect("parsed")
    }

    fn call(id: &str) -> Value {
        json!({"type": "response_item",
            "payload": {"type": "function_call", "name": "shell", "call_id": id}})
    }

    fn output(id: &str, output: Value) -> Value {
        json!({"type": "response_item",
            "payload": {"type": "function_call_output", "call_id": id, "output": output}})
    }

    fn event(kind: &str) -> Value {
        json!({"type": "event_msg", "payload": {"type": kind}})
    }

    /// Whether the analysis would count `id` as an interrupted tool call.
    fn interrupted(s: &ParsedSession, id: &str) -> bool {
        !s.tool_result_ids.contains(id) && !s.in_flight_tool_use_ids.contains(id)
    }

    #[test]
    fn a_call_still_running_is_in_flight() {
        let s = parse(&[event("task_started"), call("a")]);
        assert!(!interrupted(&s, "a"));
    }

    #[test]
    fn an_aborted_output_is_an_interruption() {
        let s = parse(&[
            call("a"),
            output("a", json!("aborted by user after 40.8s")),
            call("b"),
            output(
                "b",
                json!([{"type": "input_text", "text": "aborted by user"}]),
            ),
            call("c"),
            output("c", json!("ok")),
        ]);
        assert!(interrupted(&s, "a"));
        assert!(interrupted(&s, "b"));
        assert!(!interrupted(&s, "c"));
    }

    #[test]
    fn a_call_its_turn_abandoned_is_an_interruption() {
        let s = parse(&[call("a"), event("turn_aborted"), call("b")]);
        assert!(interrupted(&s, "a"));
        assert!(!interrupted(&s, "b"));
    }

    /// The analysis charges no wasted cost against a folded call.
    #[test]
    fn the_session_is_one_folded_call() {
        let s = parse(&[call("a"), event("turn_aborted")]);
        assert_eq!(s.calls.len(), 1);
        assert!(s.calls[0].folded);
    }
}
