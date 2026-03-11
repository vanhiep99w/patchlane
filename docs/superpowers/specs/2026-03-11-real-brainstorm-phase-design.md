# Real Brainstorm Phase Design

## Goal

Make `patchlane task --runtime codex "<objective>"` spawn a real codex process for the brainstorming phase, wait for completion, and present an interactive approval prompt. Plan and implement phases remain stubbed.

## Scope

Phase 1 only: brainstorm agent runs for real. The plan and implement agents are created as `Queued` stubs after brainstorm approval.

## User Flow

```
$ patchlane task --runtime codex "Design a caching layer"

Resolving runtime... codex found at /usr/local/bin/codex
Starting brainstorm agent...

[polls events.jsonl every 2s, timeout 10min]
[codex runs, calls `patchlane agent-event done --run-dir ... --message "brainstorm complete"`]

Brainstorm complete.
Output: .patchlane/tasks/run-<id>/logs/agent-brainstorm-stdout.log

Approve brainstorm? [y/n]: y

Brainstorm approved. Run state: waiting_for_approval (next phase: writing-plans)
```

## Architecture

### Execution Flow

```
patchlane task --runtime codex "Build X"
  1. probe_binary("codex") → verify codex is in PATH
  2. create run dir + run.json with real timestamps
  3. create brainstorm PersistedAgent (Queued, runtime from flag)
  4. launch_agent(codex, brainstorm prompt + callback instructions)
     - codex exec runs in background
     - stdout/stderr → logs/agent-brainstorm-stdout.log
     - codex calls: patchlane agent-event done --run-dir <X> ...
  5. poll_for_agent_completion("agent-brainstorm", timeout=10min)
     - read events.jsonl every 2s
     - look for Done or Fail event from agent-brainstorm
     - if process exits non-zero before event → mark Failed
  6. show brainstorm output summary
  7. interactive_approval("Approve brainstorm? [y/n]")
     - y → checkpoint Approved, create plan+implement agents as Queued stubs
     - n → checkpoint Rejected, run → WaitingForInput
  8. print summary, exit
```

### Changes by File

**`src/orchestration/runtime.rs`**
- Add `probe_binary(runtime: &Runtime) -> io::Result<PathBuf>` that runs `which codex` or `which claude`
- `DetectionContext::from_env()` falls back to `probe_binary` instead of hardcoding `Codex`

**`src/orchestration/workflow.rs`**
- Remove the fake synchronous AgentWrapper calls (the three blocks that pre-write agent state transitions)
- Make agent launch unconditional (remove `PATCHLANE_TEST_RUNTIME_MODE` gate from `maybe_launch_high_level_agents`)
- Add `poll_for_agent_completion(run_dir, agent_id, timeout)` that reads `events.jsonl` every 2s
- Add `interactive_approval(run_dir, checkpoint)` that reads stdin y/n
- Use real timestamps everywhere (`SystemTime::now()`)
- Only launch brainstorm agent; plan+implement stay as Queued stubs after approval

**`src/orchestration/phases.rs`**
- `persisted_agent()` accepts `runtime: &str` parameter instead of hardcoding `"codex"`
- Use real `SystemTime::now()` for `created_at`/`updated_at`
- `workspace_path` uses absolute path from run dir

**`src/orchestration/agent_wrapper.rs`**
- `timestamp_now()` uses `SystemTime::now()` formatted as ISO-8601
- Keep the wrapper pattern for single-owner in-process use

**`src/commands/agent_event.rs`**
- `execute()` uses `SystemTime::now()` instead of hardcoded timestamp

**`src/runtime/launcher.rs`**
- Fix log path: use `{agent_id}-stdout.log` instead of `shard-{shard_id}-stdout.log`
- After `launch_agent`, write PID to `PersistedAgent.pid` and persist
- Return `Child` handle so orchestrator can check liveness

**`src/commands/task.rs`**
- No stdin loop here; approval is handled inside `workflow.rs`

### Poll Mechanism

```rust
fn poll_for_agent_completion(
    run_dir: &Path,
    agent_id: &str,
    timeout: Duration,
) -> io::Result<AgentCompletionResult> {
    let deadline = Instant::now() + timeout;
    loop {
        if Instant::now() > deadline {
            return Ok(AgentCompletionResult::Timeout);
        }
        let events = load_task_events(run_dir)?;
        for event in events.iter().rev() {
            if event.agent_id == agent_id {
                match event.event_type {
                    AgentEventType::Done => return Ok(AgentCompletionResult::Done),
                    AgentEventType::Fail => return Ok(AgentCompletionResult::Failed(event.payload_summary.clone())),
                    _ => {}
                }
            }
        }
        thread::sleep(Duration::from_secs(2));
    }
}
```

### Interactive Approval

```rust
fn interactive_approval(
    run_dir: &Path,
    checkpoint_id: &str,
    prompt: &str,
) -> io::Result<bool> {
    loop {
        eprint!("{} [y/n]: ", prompt);
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        match input.trim() {
            "y" | "Y" => return Ok(true),
            "n" | "N" => return Ok(false),
            _ => eprintln!("Please enter y or n."),
        }
    }
}
```

### Test Compatibility

- `PATCHLANE_TEST_RUNTIME_MODE` is preserved as a test-only override
- When set, the fake `sh -c` script runs instead of real codex
- When NOT set, real codex is launched (the new default)
- All existing tests continue to use the test mode env var
- New integration test: `task_workflow_launches_real_agent_in_test_mode`

### Reporting Contract

The prompt injected into the codex exec call includes:

```
When you are done, run:
patchlane agent-event done --run-dir <PATH> --run-id <ID> --agent-id agent-brainstorm --message "brainstorm complete"

If you encounter an error, run:
patchlane agent-event fail --run-dir <PATH> --run-id <ID> --agent-id agent-brainstorm --message "<error description>"
```

This requires `patchlane` binary to be in PATH. The `launch_agent` function uses `std::env::current_exe()` to get the absolute path and passes it in the prompt.

### Error Handling

- codex not in PATH → clear error message before any state is created
- codex process exits non-zero → mark agent Failed, show stderr tail
- poll timeout (10min) → mark agent Failed with timeout message
- stdin EOF during approval → treat as rejection
- events.jsonl read error → retry next poll cycle

### Out of Scope

- Plan and implement phases (future work)
- Claude runtime (only codex for now, but plumbing supports both)
- TUI integration with live agent output
- Multi-agent parallel execution
- Workspace isolation per agent
