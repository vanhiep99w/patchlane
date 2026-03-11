# Real Brainstorm Phase Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `patchlane task --runtime codex "<objective>"` spawn a real codex process for the brainstorming phase, poll for completion, and present an interactive approval prompt.

**Architecture:** Replace the fake synchronous AgentWrapper calls in `workflow.rs` with real codex process spawning via `launch_agent`, add an event-based poll loop that watches `events.jsonl` for agent completion, and wire an interactive stdin approval prompt. Fix all hardcoded timestamps, log path mismatches, and runtime labels throughout the orchestration layer.

**Tech Stack:** Rust, `std::process::Command`, `std::io::stdin`, filesystem-based polling on `events.jsonl`, `std::time::SystemTime` for real timestamps.

---

## File Structure

- Modify: `src/orchestration/agent_wrapper.rs`
  Fix `timestamp_now()` to use real `SystemTime::now()`.
- Modify: `src/commands/agent_event.rs`
  Fix hardcoded timestamp in `execute()` to use real `SystemTime::now()`.
- Modify: `src/orchestration/phases.rs`
  Accept runtime parameter, use real timestamps.
- Modify: `src/orchestration/approval.rs`
  Fix hardcoded timestamp to use real `SystemTime::now()`.
- Modify: `src/orchestration/runtime.rs`
  Add `probe_binary()` function that checks PATH for codex/claude.
- Modify: `src/runtime/launcher.rs`
  Fix log path format from `shard-{shard_id}-*` to `{agent_id}-*` for agent launches. Return `ManagedLaunchOutcome` (with `Child` handle) from `launch_agent`.
- Modify: `src/orchestration/workflow.rs`
  Remove fake AgentWrapper state calls, launch real brainstorm agent, add poll loop, add interactive approval, store PID.
- Modify: `tests/orchestration/workflow.rs`
  Update tests for new workflow behavior.
- Modify: `tests/e2e/cli_contract.rs`
  Update e2e test for new log paths and workflow behavior.

---

## Chunk 1: Timestamp And Log Path Fixes

### Task 1: Fix All Hardcoded Timestamps

**Files:**
- Modify: `src/orchestration/agent_wrapper.rs`
- Modify: `src/commands/agent_event.rs`
- Modify: `src/orchestration/phases.rs`
- Modify: `src/orchestration/approval.rs`
- Modify: `src/orchestration/workflow.rs`

- [ ] **Step 1: Write the failing timestamp test**

Add to `tests/orchestration/workflow.rs`:

```rust
#[test]
fn workflow_uses_real_timestamps_not_hardcoded() {
    let state_root = temp_state_root("timestamp-test");
    let command = TaskCommand {
        runtime: Some(Runtime::Codex),
        objective: "verify timestamps".to_owned(),
    };
    let run_dir = execute_task_workflow(&state_root, command).expect("workflow should succeed");
    let snapshot = load_task_snapshot(&run_dir).expect("snapshot should load");

    assert_ne!(
        snapshot.run.created_at, "2026-03-10T00:00:00Z",
        "run created_at should use real time"
    );
    assert_ne!(
        snapshot.run.updated_at, "2026-03-10T00:00:00Z",
        "run updated_at should use real time"
    );
    for agent in &snapshot.agents {
        assert_ne!(
            agent.created_at, "2026-03-10T00:00:00Z",
            "agent {} created_at should use real time",
            agent.agent_id
        );
    }
    for event in &snapshot.events {
        assert_ne!(
            event.timestamp, "2026-03-10T00:00:00Z",
            "event {} timestamp should use real time",
            event.event_id
        );
    }
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test --test orchestration workflow_uses_real_timestamps_not_hardcoded -- --exact`
Expected: FAIL because timestamps are hardcoded to `"2026-03-10T00:00:00Z"`.

- [ ] **Step 3: Fix `timestamp_now()` in `agent_wrapper.rs`**

Replace line 136-138 in `src/orchestration/agent_wrapper.rs`:

```rust
fn timestamp_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch");
    let secs = now.as_secs();
    let nanos = now.subsec_nanos();
    // ISO-8601 with sub-second precision
    let datetime = time_from_unix(secs);
    format!("{}T{}.{:03}Z", datetime.0, datetime.1, nanos / 1_000_000)
}

fn time_from_unix(secs: u64) -> (String, String) {
    // Simple UTC date/time from unix timestamp
    let days = secs / 86400;
    let time_secs = secs % 86400;
    let hours = time_secs / 3600;
    let mins = (time_secs % 3600) / 60;
    let s = time_secs % 60;

    // Days since 1970-01-01 to Y-M-D (simplified Rata Die)
    let (y, m, d) = civil_from_days(days as i64);
    (
        format!("{y:04}-{m:02}-{d:02}"),
        format!("{hours:02}:{mins:02}:{s:02}"),
    )
}

fn civil_from_days(days: i64) -> (i64, u32, u32) {
    // Algorithm from Howard Hinnant's date library
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}
```

- [ ] **Step 4: Make `timestamp_now` public and reuse it**

Export `timestamp_now` from `agent_wrapper.rs` as `pub fn timestamp_now()` so other modules can use it.

- [ ] **Step 5: Fix timestamp in `commands/agent_event.rs`**

In `src/commands/agent_event.rs`, line 49, replace:

```rust
timestamp: "2026-03-10T00:00:00Z".to_owned(),
```

with:

```rust
timestamp: crate::orchestration::agent_wrapper::timestamp_now(),
```

- [ ] **Step 6: Fix timestamps in `phases.rs`**

In `src/orchestration/phases.rs`, replace all `"2026-03-10T00:00:00Z".to_owned()` with `crate::orchestration::agent_wrapper::timestamp_now()`:

- Line 22 in `spec_artifact`: `created_at`
- Line 33 in `plan_artifact`: `created_at`
- Line 51 in `persisted_agent`: `created_at`
- Line 52 in `persisted_agent`: `updated_at`

- [ ] **Step 7: Fix timestamp in `approval.rs`**

In `src/orchestration/approval.rs`, line 35, replace:

```rust
updated_at: "2026-03-10T00:00:01Z".to_owned(),
```

with:

```rust
updated_at: crate::orchestration::agent_wrapper::timestamp_now(),
```

- [ ] **Step 8: Fix timestamps in `workflow.rs`**

In `src/orchestration/workflow.rs`, replace lines 33-34:

```rust
created_at: "2026-03-10T00:00:00Z".to_owned(),
updated_at: "2026-03-10T00:00:00Z".to_owned(),
```

with:

```rust
created_at: crate::orchestration::agent_wrapper::timestamp_now(),
updated_at: crate::orchestration::agent_wrapper::timestamp_now(),
```

And line 93 (`timestamp: "2026-03-10T00:00:00Z".to_owned()`) with:

```rust
timestamp: crate::orchestration::agent_wrapper::timestamp_now(),
```

- [ ] **Step 9: Run the timestamp test and full suite**

Run: `cargo test --test orchestration workflow_uses_real_timestamps_not_hardcoded -- --exact`
Expected: PASS.

Run: `cargo test`
Expected: PASS (112+ tests). Some existing tests that assert exact timestamp strings may need updating — if they fail, update the assertions to check for non-hardcoded timestamps instead.

- [ ] **Step 10: Commit**

```bash
git add src/orchestration/agent_wrapper.rs src/commands/agent_event.rs src/orchestration/phases.rs src/orchestration/approval.rs src/orchestration/workflow.rs tests/orchestration/workflow.rs
git commit -m "fix: replace all hardcoded timestamps with real SystemTime::now()"
```

### Task 2: Fix Log Path Mismatch In Launcher

**Files:**
- Modify: `src/runtime/launcher.rs`
- Modify: `tests/e2e/cli_contract.rs`

- [ ] **Step 1: Write the failing log path test**

Add to `tests/orchestration/workflow.rs`:

```rust
#[test]
fn agent_launch_log_paths_match_persisted_agent_log_fields() {
    let state_root = temp_state_root("log-path-test");
    let command = TaskCommand {
        runtime: Some(Runtime::Codex),
        objective: "verify log paths".to_owned(),
    };
    let run_dir = execute_task_workflow(&state_root, command).expect("workflow should succeed");
    let snapshot = load_task_snapshot(&run_dir).expect("snapshot should load");

    for agent in &snapshot.agents {
        let stdout_path = run_dir.join("logs").join(&agent.stdout_log);
        assert!(
            stdout_path.exists(),
            "stdout log should exist at {}, agent {}",
            stdout_path.display(),
            agent.agent_id
        );
        let stderr_path = run_dir.join("logs").join(&agent.stderr_log);
        assert!(
            stderr_path.exists(),
            "stderr log should exist at {}, agent {}",
            stderr_path.display(),
            agent.agent_id
        );
    }
}
```

- [ ] **Step 2: Run the test**

Run: `cargo test --test orchestration agent_launch_log_paths_match_persisted_agent_log_fields -- --exact`
Expected: Should PASS since `ensure_log_files` uses agent log names and those files exist. If it passes already, good — this is a regression guard.

- [ ] **Step 3: Fix `spawn_worker` log paths for agent launches**

In `src/runtime/launcher.rs`, modify `launch_agent` to pass the agent's expected log filenames to `spawn_worker` instead of letting it derive `shard-{shard_id}-*` names.

Add an `agent_id` field to `LaunchRequest`:

```rust
#[derive(Debug, Clone)]
pub struct LaunchRequest {
    pub runtime: Runtime,
    pub shard_id: String,
    pub brief: String,
    pub workspace: PathBuf,
    pub logs_dir: PathBuf,
    pub agent_id: Option<String>,  // NEW: when set, use {agent_id}-stdout.log instead of shard-{shard_id}-stdout.log
}
```

In `spawn_worker`, lines 180-185, change the log path derivation:

```rust
let log_prefix = request.agent_id.as_deref().unwrap_or(&format!("shard-{}", request.shard_id));
let stdout_log = request.logs_dir.join(format!("{log_prefix}-stdout.log"));
let stderr_log = request.logs_dir.join(format!("{log_prefix}-stderr.log"));
```

In `launch_agent`, set `agent_id` on the `LaunchRequest`:

```rust
let launch_request = LaunchRequest {
    runtime: request.runtime.clone(),
    shard_id: format!("agent-{}", request.role),
    brief: format!("{}{}", request.prompt, reporting_contract),
    workspace: request.workspace.clone(),
    logs_dir: request.logs_dir.clone(),
    agent_id: Some(request.role.clone()),
};
```

Update all existing `LaunchRequest` constructions (in `intervention_support.rs`) to include `agent_id: None`.

- [ ] **Step 4: Update e2e test log path assertions**

In `tests/e2e/cli_contract.rs`, the test `cli_contract_covers_task_workflow_and_persisted_artifacts` asserts `logs/shard-agent-agent-implement-stdout.log`. Update to `logs/agent-implement-stdout.log`.

- [ ] **Step 5: Run tests**

Run: `cargo test`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src/runtime/launcher.rs src/commands/intervention_support.rs tests/e2e/cli_contract.rs tests/orchestration/workflow.rs
git commit -m "fix: align agent launch log paths with persisted agent log fields"
```

### Task 3: Fix Runtime Label In Phases

**Files:**
- Modify: `src/orchestration/phases.rs`
- Modify: `src/orchestration/workflow.rs`

- [ ] **Step 1: Write the failing runtime label test**

Add to `tests/orchestration/workflow.rs`:

```rust
#[test]
fn agents_persist_the_resolved_runtime_not_hardcoded_codex() {
    let state_root = temp_state_root("runtime-label-test");
    let command = TaskCommand {
        runtime: Some(Runtime::Claude),
        objective: "verify runtime label".to_owned(),
    };
    let run_dir = execute_task_workflow(&state_root, command).expect("workflow should succeed");
    let snapshot = load_task_snapshot(&run_dir).expect("snapshot should load");

    for agent in &snapshot.agents {
        assert_eq!(
            agent.runtime, "claude",
            "agent {} should use runtime claude, not hardcoded codex",
            agent.agent_id
        );
    }
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test --test orchestration agents_persist_the_resolved_runtime_not_hardcoded_codex -- --exact`
Expected: FAIL because `phases.rs` hardcodes `runtime: "codex"`.

- [ ] **Step 3: Fix `phases.rs` to accept runtime parameter**

Change `persisted_agent` signature and all callers:

```rust
pub fn brainstorming_agent(run_id: &str, runtime: &str) -> PersistedAgent {
    persisted_agent(run_id, "agent-brainstorm", "brainstorming", runtime)
}

pub fn planning_agent(run_id: &str, runtime: &str) -> PersistedAgent {
    persisted_agent(run_id, "agent-plan", "writing-plans", runtime)
}

pub fn implementation_agent(run_id: &str, runtime: &str) -> PersistedAgent {
    persisted_agent(run_id, "agent-implement", "subagent-driven-development", runtime)
}

fn persisted_agent(run_id: &str, agent_id: &str, role: &str, runtime: &str) -> PersistedAgent {
    PersistedAgent {
        // ... same as before, but:
        runtime: runtime.to_owned(),
        // ...
    }
}
```

- [ ] **Step 4: Update callers in `workflow.rs`**

In `workflow.rs`, pass the runtime label to phase constructors:

```rust
let runtime_str = runtime_label(command.runtime.as_ref().unwrap_or(&Runtime::Codex));
let brainstorm = brainstorming_agent(&run_id, runtime_str);
let planner = planning_agent(&run_id, runtime_str);
let implementer = implementation_agent(&run_id, runtime_str);
```

- [ ] **Step 5: Run tests**

Run: `cargo test`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src/orchestration/phases.rs src/orchestration/workflow.rs tests/orchestration/workflow.rs
git commit -m "fix: pass resolved runtime to agent constructors instead of hardcoding codex"
```

## Chunk 2: Runtime Detection And Agent Launch

### Task 4: Add Binary Detection To Runtime Resolution

**Files:**
- Modify: `src/orchestration/runtime.rs`

- [ ] **Step 1: Write the failing binary probe test**

Add to `tests/orchestration/runtime.rs`:

```rust
#[test]
fn probe_binary_finds_installed_runtime() {
    // `sh` is universally available on unix systems
    let result = probe_binary_path("sh");
    assert!(result.is_ok(), "sh should be found in PATH");
    let path = result.unwrap();
    assert!(path.exists(), "returned path should exist");
}

#[test]
fn probe_binary_returns_error_for_missing_binary() {
    let result = probe_binary_path("__patchlane_nonexistent_binary__");
    assert!(result.is_err(), "missing binary should error");
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test orchestration probe_binary -- --nocapture`
Expected: FAIL because `probe_binary_path` does not exist.

- [ ] **Step 3: Implement `probe_binary_path`**

Add to `src/orchestration/runtime.rs`:

```rust
use std::path::PathBuf;
use std::process::Command;

pub fn probe_binary_path(binary: &str) -> io::Result<PathBuf> {
    let output = Command::new("which")
        .arg(binary)
        .output()
        .map_err(|e| io::Error::new(io::ErrorKind::NotFound, e))?;
    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        Ok(PathBuf::from(path))
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("{binary} not found in PATH"),
        ))
    }
}
```

Add `use std::io;` to the imports.

- [ ] **Step 4: Run tests**

Run: `cargo test --test orchestration probe_binary -- --nocapture`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/orchestration/runtime.rs tests/orchestration/runtime.rs
git commit -m "feat: add probe_binary_path for runtime detection"
```

### Task 5: Make Workflow Launch Real Brainstorm Agent

**Files:**
- Modify: `src/orchestration/workflow.rs`
- Modify: `src/runtime/launcher.rs`

This is the core task. The workflow must:
1. Create agents as Queued (no fake state transitions)
2. Launch the brainstorm agent via `launch_agent`
3. Store PID
4. Poll for agent completion
5. Show results
6. Interactive approval
7. Create plan+implement agents as stubs after approval

- [ ] **Step 1: Write the failing real-launch test**

Add to `tests/orchestration/workflow.rs`:

```rust
#[test]
fn brainstorm_agent_is_launched_and_polled_to_completion() {
    std::env::set_var("PATCHLANE_TEST_RUNTIME_MODE", "success");
    let state_root = temp_state_root("real-launch-test");
    let command = TaskCommand {
        runtime: Some(Runtime::Codex),
        objective: "brainstorm with real launch".to_owned(),
    };
    let run_dir = execute_task_workflow(&state_root, command).expect("workflow should succeed");
    let snapshot = load_task_snapshot(&run_dir).expect("snapshot should load");

    let brainstorm = snapshot
        .agents
        .iter()
        .find(|a| a.agent_id == "agent-brainstorm")
        .expect("brainstorm agent should exist");

    // Agent should have been launched (has PID from test script)
    assert!(brainstorm.pid.is_some(), "brainstorm agent should have a PID after launch");

    // Agent should have a phase event from the launched process
    assert!(
        snapshot.events.iter().any(|e| {
            e.agent_id.as_deref() == Some("agent-brainstorm")
                && e.event_type == AgentEventType::Phase
        }),
        "brainstorm agent should have a phase event from launched process"
    );

    std::env::remove_var("PATCHLANE_TEST_RUNTIME_MODE");
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test --test orchestration brainstorm_agent_is_launched_and_polled_to_completion -- --exact`
Expected: FAIL because current workflow doesn't launch brainstorm agent.

- [ ] **Step 3: Modify `launch_agent` to return `ManagedLaunchOutcome`**

In `src/runtime/launcher.rs`, change `launch_agent` return type:

```rust
pub fn launch_agent(request: &AgentLaunchRequest) -> Result<ManagedLaunchOutcome, RuntimeLaunchError> {
    let reporting_contract = format!(
        " Report state with `{} agent-event --run-dir {} --run-id {} --agent-id {} --message <payload> <event-type>`. Use artifact payload `<type>|<path>` and waiting-approval payload `<checkpoint-id>|<prompt>`.",
        std::env::current_exe().unwrap_or_else(|_| PathBuf::from("patchlane")).display(),
        request.run_dir.display(),
        request.run_id,
        request.role
    );
    let launch_request = LaunchRequest {
        runtime: request.runtime.clone(),
        shard_id: format!("agent-{}", request.role),
        brief: format!("{}{}", request.prompt, reporting_contract),
        workspace: request.workspace.clone(),
        logs_dir: request.logs_dir.clone(),
        agent_id: Some(request.role.clone()),
    };
    let spec = build_agent_launch_spec(request, &launch_request);
    let args = spec.args.iter().map(String::as_str).collect::<Vec<_>>();
    spawn_worker(&launch_request, spec.program, &args)
}
```

- [ ] **Step 4: Restructure `workflow.rs` — remove fake calls, launch real brainstorm agent**

Replace `execute_task_workflow` with the real implementation. Key changes:

1. Create agents as Queued — no fake `.start()`, `.phase()`, `.artifact()`, `.done()` calls
2. Launch brainstorm agent via `launch_agent`
3. Store PID in agent
4. Poll `events.jsonl` for Done/Fail events from agent-brainstorm
5. After brainstorm completes, check for approval checkpoint
6. Create plan+implement agents remain as Queued stubs

```rust
pub fn execute_task_workflow(root: &Path, command: TaskCommand) -> io::Result<PathBuf> {
    let run_id = generate_run_id();
    let runtime = command.runtime.as_ref().unwrap_or(&Runtime::Codex);
    let runtime_str = runtime_label(runtime);

    let mut run = PersistedTaskRun {
        run_id: run_id.clone(),
        objective: command.objective.clone(),
        runtime: runtime_str.to_owned(),
        current_phase: "brainstorming".to_owned(),
        overall_state: OrchestratorState::Running,
        blocking_reason: None,
        workspace_root: "workspace".to_owned(),
        workspace_policy: "isolated_by_default".to_owned(),
        default_isolation: true,
        created_at: crate::orchestration::agent_wrapper::timestamp_now(),
        updated_at: crate::orchestration::agent_wrapper::timestamp_now(),
    };

    let run_dir = create_task_run(root, &run)?;

    // Create all three agents as Queued
    let brainstorm = brainstorming_agent(&run_id, runtime_str);
    let planner = planning_agent(&run_id, runtime_str);
    let implementer = implementation_agent(&run_id, runtime_str);
    write_agent(&run_dir, &brainstorm)?;
    write_agent(&run_dir, &planner)?;
    write_agent(&run_dir, &implementer)?;
    ensure_log_files(&run_dir, &[&brainstorm, &planner, &implementer])?;

    // Create checkpoints
    for mut checkpoint in build_phase_checkpoints(&run_id, "agent-brainstorm")
        .into_iter()
        .filter(|c| c.phase == "after-brainstorming")
    {
        checkpoint.status = CheckpointStatus::Pending;
        write_checkpoint(&run_dir, &checkpoint)?;
    }

    // Launch brainstorm agent
    let request = AgentLaunchRequest {
        runtime: runtime.clone(),
        run_id: run_id.clone(),
        role: "agent-brainstorm".to_owned(),
        prompt: format!(
            "Brainstorm approaches for: {}. When done, report completion.",
            command.objective
        ),
        workspace: run_dir.join("workspace-agent-brainstorm"),
        logs_dir: run_dir.join("logs"),
        run_dir: run_dir.clone(),
    };

    match launch_agent(&request) {
        Ok(mut launch) => {
            // Store PID
            let pid = launch.child.id();
            let mut agent = brainstorm;
            agent.pid = Some(pid);
            agent.current_state = OrchestratorState::Running;
            agent.updated_at = crate::orchestration::agent_wrapper::timestamp_now();
            write_agent(&run_dir, &agent)?;

            // Poll for completion
            let result = poll_for_agent_completion(
                &run_dir,
                "agent-brainstorm",
                &mut launch.child,
                Duration::from_secs(600),
            );

            match result {
                Ok(AgentCompletionResult::Done) => {
                    eprintln!("Brainstorm complete.");
                    eprintln!(
                        "Output: {}",
                        run_dir.join("logs").join(&agent.stdout_log).display()
                    );
                }
                Ok(AgentCompletionResult::Failed(reason)) => {
                    eprintln!("Brainstorm failed: {reason}");
                    agent.current_state = OrchestratorState::Failed;
                    agent.updated_at = crate::orchestration::agent_wrapper::timestamp_now();
                    write_agent(&run_dir, &agent)?;
                    run.overall_state = OrchestratorState::Failed;
                    run.blocking_reason = Some(format!("brainstorm failed: {reason}"));
                    run.updated_at = crate::orchestration::agent_wrapper::timestamp_now();
                    write_task_run(&run_dir, &run)?;
                    return Ok(run_dir);
                }
                Ok(AgentCompletionResult::Timeout) => {
                    eprintln!("Brainstorm timed out after 10 minutes.");
                    agent.current_state = OrchestratorState::Failed;
                    agent.updated_at = crate::orchestration::agent_wrapper::timestamp_now();
                    write_agent(&run_dir, &agent)?;
                    run.overall_state = OrchestratorState::Failed;
                    run.blocking_reason = Some("brainstorm timed out".to_owned());
                    run.updated_at = crate::orchestration::agent_wrapper::timestamp_now();
                    write_task_run(&run_dir, &run)?;
                    return Ok(run_dir);
                }
                Err(error) => {
                    return Err(error);
                }
            }
        }
        Err(error) => {
            return Err(io::Error::other(format!(
                "failed to launch brainstorm agent: {error:?}"
            )));
        }
    }

    // Interactive approval
    let approved = if std::env::var("PATCHLANE_TEST_RUNTIME_MODE").is_ok() {
        // In test mode, auto-approve
        true
    } else {
        interactive_approval("Approve brainstorm?")?
    };

    if approved {
        // Write after-brainstorming checkpoint as approved
        if let Some(mut checkpoint) = load_task_snapshot(&run_dir)?
            .checkpoints
            .into_iter()
            .find(|c| c.phase == "after-brainstorming")
        {
            checkpoint.status = CheckpointStatus::Approved;
            checkpoint.response = Some("y".to_owned());
            checkpoint.updated_at = crate::orchestration::agent_wrapper::timestamp_now();
            write_checkpoint(&run_dir, &checkpoint)?;
        }

        run.current_phase = "writing-plans".to_owned();
        run.overall_state = OrchestratorState::WaitingForApproval;
        run.blocking_reason = Some("approval required".to_owned());
    } else {
        run.overall_state = OrchestratorState::WaitingForInput;
        run.blocking_reason = Some("brainstorm rejected".to_owned());
    }

    run.updated_at = crate::orchestration::agent_wrapper::timestamp_now();
    write_task_run(&run_dir, &run)?;
    append_task_event(
        &run_dir,
        &crate::orchestration::model::PersistedTaskEvent {
            event_id: "event-run-brainstorm-complete".to_owned(),
            run_id: run_id.clone(),
            agent_id: None,
            event_type: crate::orchestration::model::AgentEventType::Phase,
            payload_summary: run.current_phase.clone(),
            timestamp: crate::orchestration::agent_wrapper::timestamp_now(),
        },
    )?;

    Ok(run_dir)
}
```

- [ ] **Step 5: Add `poll_for_agent_completion` function**

Add to `src/orchestration/workflow.rs`:

```rust
enum AgentCompletionResult {
    Done,
    Failed(String),
    Timeout,
}

fn poll_for_agent_completion(
    run_dir: &Path,
    agent_id: &str,
    child: &mut Child,
    timeout: Duration,
) -> io::Result<AgentCompletionResult> {
    let deadline = std::time::Instant::now() + timeout;
    loop {
        if std::time::Instant::now() > deadline {
            let _ = child.kill();
            return Ok(AgentCompletionResult::Timeout);
        }

        // Check if process exited
        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    // Check events for a Fail event before declaring failure
                    if let Ok(snapshot) = load_task_snapshot(run_dir) {
                        if let Some(fail_event) = snapshot.events.iter().rev().find(|e| {
                            e.agent_id.as_deref() == Some(agent_id)
                                && e.event_type == crate::orchestration::model::AgentEventType::Fail
                        }) {
                            return Ok(AgentCompletionResult::Failed(
                                fail_event.payload_summary.clone(),
                            ));
                        }
                    }
                    return Ok(AgentCompletionResult::Failed(format!(
                        "process exited with {}",
                        status
                    )));
                }
                // Process exited successfully — check for Done event
                if let Ok(snapshot) = load_task_snapshot(run_dir) {
                    if snapshot.events.iter().any(|e| {
                        e.agent_id.as_deref() == Some(agent_id)
                            && e.event_type == crate::orchestration::model::AgentEventType::Done
                    }) {
                        return Ok(AgentCompletionResult::Done);
                    }
                }
                // Process exited 0 but no Done event — treat as done anyway
                return Ok(AgentCompletionResult::Done);
            }
            Ok(None) => {
                // Process still running — check events
                if let Ok(snapshot) = load_task_snapshot(run_dir) {
                    for event in snapshot.events.iter().rev() {
                        if event.agent_id.as_deref() == Some(agent_id) {
                            match event.event_type {
                                crate::orchestration::model::AgentEventType::Done => {
                                    return Ok(AgentCompletionResult::Done);
                                }
                                crate::orchestration::model::AgentEventType::Fail => {
                                    return Ok(AgentCompletionResult::Failed(
                                        event.payload_summary.clone(),
                                    ));
                                }
                                _ => break,
                            }
                        }
                    }
                }
                thread::sleep(Duration::from_secs(2));
            }
            Err(e) => return Err(e),
        }
    }
}
```

- [ ] **Step 6: Add `interactive_approval` function**

Add to `src/orchestration/workflow.rs`:

```rust
fn interactive_approval(prompt: &str) -> io::Result<bool> {
    use std::io::Write;
    loop {
        eprint!("\n{} [y/n]: ", prompt);
        io::stderr().flush()?;
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => return Ok(false), // EOF
            Ok(_) => match input.trim() {
                "y" | "Y" => return Ok(true),
                "n" | "N" => return Ok(false),
                _ => eprintln!("Please enter y or n."),
            },
            Err(e) => return Err(e),
        }
    }
}
```

- [ ] **Step 7: Update imports in `workflow.rs`**

Add to imports:

```rust
use std::process::Child;
use crate::runtime::launcher::{launch_agent, AgentLaunchRequest, ManagedLaunchOutcome};
```

Remove unused imports from the old fake-agent code path.

- [ ] **Step 8: Run the new test**

Run: `PATCHLANE_TEST_RUNTIME_MODE=success cargo test --test orchestration brainstorm_agent_is_launched_and_polled_to_completion -- --exact --nocapture`
Expected: PASS.

- [ ] **Step 9: Update the test success spec script**

In `src/runtime/launcher.rs`, update `build_agent_success_spec` to emit a `done` event instead of a `phase` event, so the poll loop can detect completion:

Change the script line from:
```
"\"$1\" agent-event phase --run-dir \"$2\" --run-id \"$3\" --agent-id \"$4\" --message \"$5\""
```
to:
```
"\"$1\" agent-event phase --run-dir \"$2\" --run-id \"$3\" --agent-id \"$4\" --message \"$5\"; \"$1\" agent-event done --run-dir \"$2\" --run-id \"$3\" --agent-id \"$4\" --message \"brainstorm complete\""
```

- [ ] **Step 10: Update existing tests that depend on old workflow behavior**

The existing tests in `tests/orchestration/workflow.rs` and `tests/e2e/cli_contract.rs` rely on the old fake-agent workflow. Update them:

- `task_command_follows_brainstorming_approval_plan_approval_execution_flow` — update to expect real agent launch behavior (agent-brainstorm running with PID, brainstorm Done event)
- `cli_contract_covers_task_workflow_and_persisted_artifacts` — update assertions for new log paths, agent states, and event structure
- `blocked_agent_does_not_stop_independent_agents` — may need to use fixture snapshots instead of workflow execution

- [ ] **Step 11: Run full test suite**

Run: `cargo test`
Expected: PASS.

- [ ] **Step 12: Commit**

```bash
git add src/orchestration/workflow.rs src/runtime/launcher.rs tests/orchestration/workflow.rs tests/e2e/cli_contract.rs
git commit -m "feat: launch real brainstorm agent with poll loop and interactive approval"
```

### Task 6: Full Verification And Smoke Test

**Files:**
- No new files

- [ ] **Step 1: Run the full test suite**

Run: `cargo test`
Expected: PASS across all test targets.

- [ ] **Step 2: Run manual smoke test with codex (if available)**

Run: `cargo run -- task --runtime codex "Design a simple greeting function"`

Expected behavior:
1. Prints "Starting brainstorm agent..."
2. Codex process runs (visible in `.patchlane/tasks/run-<id>/logs/agent-brainstorm-stdout.log`)
3. After codex finishes, prints "Brainstorm complete."
4. Prompts "Approve brainstorm? [y/n]: "
5. On `y`: prints summary, run state is `WaitingForApproval` with phase `writing-plans`
6. On `n`: run state is `WaitingForInput` with reason `brainstorm rejected`

If codex is not installed, test with:
Run: `PATCHLANE_TEST_RUNTIME_MODE=success cargo run -- task --runtime codex "Design a simple greeting function"`

- [ ] **Step 3: Verify TUI still works with real run data**

Run: `cargo run -- tui`
Expected: TUI displays the run created in step 2.

- [ ] **Step 4: Commit resume doc update**

```bash
# Update docs/superpowers/plans/2026-03-10-patchlane-task-orchestration-resume.md with real brainstorm status
git add docs/superpowers/plans/2026-03-10-patchlane-task-orchestration-resume.md
git commit -m "docs: update resume with real brainstorm phase status"
```
