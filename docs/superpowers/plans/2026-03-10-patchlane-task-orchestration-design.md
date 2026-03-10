# Patchlane Task Orchestration Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a new `patchlane task "<objective>"` workflow that runs `brainstorming -> writing-plans -> subagent-driven-development -> finishing-a-development-branch`, persists orchestration state locally, pauses for terminal approvals, and exposes a minimal TUI for live swarm inspection.

**Architecture:** Extend the existing Rust CLI around a new orchestration control plane instead of replacing the current shard bootstrap in one jump. Keep the persisted run store as the only source of truth, make high-level agents report structured events through Patchlane-owned wrappers, and have the TUI read directly from that store so execution and observability stay decoupled.

**Tech Stack:** Rust, `clap`, `serde`, `serde_json`, filesystem-backed state under `.patchlane/`, local runtime CLIs (`codex` / `claude`), `ratatui` + `crossterm` for the minimal TUI.

---

## File Structure

- Modify: `Cargo.toml`
  Add the TUI dependencies and keep the dependency surface small.
- Modify: `src/cli.rs`
  Add the new `task` entrypoint plus a top-level `tui` entrypoint.
- Modify: `src/commands/mod.rs`
  Route the new commands while preserving the current `swarm` surfaces.
- Create: `src/commands/task.rs`
  Own `patchlane task` startup, input validation, runtime resolution, and workflow invocation.
- Create: `src/commands/agent_event.rs`
  Expose the `patchlane agent-event ...` reporting surface for spawned high-level agents.
- Create: `src/commands/tui.rs`
  Launch the minimal read-only TUI.
- Create: `src/orchestration/mod.rs`
  Export orchestration modules.
- Create: `src/orchestration/model.rs`
  Define persisted run, agent, checkpoint, artifact, event, and snapshot types.
- Create: `src/orchestration/store.rs`
  Persist and load the richer orchestration entities with a filesystem-backed store.
- Create: `src/orchestration/runtime.rs`
  Resolve the run runtime from `--runtime`, detection, or confirmation-required states.
- Create: `src/orchestration/approval.rs`
  Handle `Approve? [y/n]` prompts and persisted checkpoint decisions.
- Create: `src/orchestration/checkpoints.rs`
  Define the v1 checkpoint classes and build concrete checkpoint records.
- Create: `src/orchestration/agent_wrapper.rs`
  Translate high-level agent lifecycle events into persisted state updates.
- Create: `src/orchestration/phases.rs`
  Keep per-phase launch and artifact handling logic focused and testable.
- Create: `src/orchestration/workflow.rs`
  Run the approved phase sequence and coordinate approvals.
- Create: `src/orchestration/recovery.rs`
  Rebuild blocked or resumable state from persisted files after restart.
- Modify: `src/runtime/launcher.rs`
  Add high-level agent launch support and deterministic log path allocation.
- Modify: `src/store/mod.rs`
  Export both the legacy store and the orchestration store cleanly.
- Modify: `src/events/run_events.rs`
  Derive CLI/TUI snapshots from the run/agent/checkpoint/artifact event model.
- Modify: `src/renderers/status_renderer.rs`
  Render run phase, blockers, approval waits, and agent summaries.
- Modify: `src/renderers/watch_renderer.rs`
  Render orchestration events without transcript-noise assumptions.
- Modify: `src/commands/status.rs`
  Read the richer store-backed status snapshot.
- Modify: `src/commands/watch.rs`
  Read the richer store-backed event timeline.
- Modify: `src/commands/board.rs`
  Replace fixture text with a persisted run/agent overview.
- Create: `src/tui/mod.rs`
  Export TUI-specific models and helpers.
- Create: `src/tui/app.rs`
  Load runs, manage selection state, and poll for refreshes.
- Create: `src/tui/store.rs`
  Read persisted runs and selected-run snapshots from the local run store.
- Create: `src/tui/render.rs`
  Draw run list, agent list, selected-agent details, timeline, artifacts, and blockers.
- Create: `src/tui/logs.rs`
  Tail persisted agent stdout/stderr files for the selected agent.
- Modify: `src/lib.rs`
  Export orchestration and TUI modules.
- Modify: `README.md`
  Document the `patchlane task` flow, approvals, persisted store layout, and TUI usage.
- Create: `tests/cli.rs`
  Root integration-test entrypoint for CLI-focused tests.
- Modify: `tests/cli/main.rs`
  Register new command-surface and board-output tests.
- Create: `tests/cli/task_output.rs`
  Verify `patchlane task` output, prompts, and blocked-state messaging.
- Modify: `tests/cli/status_output.rs`
  Update expectations for agent-aware status rendering.
- Modify: `tests/cli/watch_output.rs`
  Update expectations for agent-aware watch rendering.
- Create: `tests/support/mod.rs`
  Shared fixture helpers for orchestration snapshots and persisted-store setups.
- Create: `tests/support/task_fixtures.rs`
  Build fixture snapshots, persisted runs, and log files for CLI/TUI tests.
- Create: `tests/fixtures/agent-plan-stdout.log`
  Fixture stdout log for selected-agent tail tests.
- Create: `tests/fixtures/agent-plan-stderr.log`
  Fixture stderr log for selected-agent tail tests.
- Modify: `tests/cli/overview_commands.rs`
  Replace board fixture expectations with persisted-state expectations.
- Modify: `tests/cli/command_topology.rs`
  Cover the new top-level `task` and `tui` commands.
- Create: `tests/cli/board_output.rs`
  Verify the board summary surface.
- Create: `tests/orchestration.rs`
  Root integration-test entrypoint for orchestration-focused tests.
- Create: `tests/orchestration/main.rs`
  Register orchestration-focused test modules.
- Create: `tests/orchestration/store.rs`
  Test persistence and recovery of the richer entities.
- Create: `tests/orchestration/runtime.rs`
  Test runtime resolution and confirmation policy.
- Create: `tests/orchestration/checkpoints.rs`
  Test v1 checkpoint classes and approval-state transitions.
- Create: `tests/orchestration/agent_wrapper.rs`
  Test the high-level agent reporting contract.
- Create: `tests/orchestration/workflow.rs`
  Test phase sequence, approvals, review interventions, and restart recovery.
- Create: `tests/tui.rs`
  Root integration-test entrypoint for TUI-focused tests.
- Create: `tests/tui/main.rs`
  Register TUI-focused test modules.
- Create: `tests/tui/render.rs`
  Test selected-agent phase/state, blockers, timeline, and artifact rendering.
- Create: `tests/tui/logs.rs`
  Test log tailing from persisted files.
- Modify: `tests/e2e/cli_contract.rs`
  Cover the end-to-end task workflow and persisted artifacts.

## Chunk 1: Control Plane And Persisted Workflow

### Task 1: Add The `patchlane task` CLI Surface

**Files:**
- Modify: `src/cli.rs`
- Modify: `src/commands/mod.rs`
- Create: `src/commands/task.rs`
- Modify: `src/lib.rs`
- Create: `tests/cli.rs`
- Modify: `tests/cli/main.rs`
- Create: `tests/cli/task_output.rs`
- Modify: `tests/cli/command_topology.rs`

- [ ] **Step 1: Register the CLI test target and write the failing parsing/help tests**

```rust
#[test]
fn parses_task_command_with_objective_and_optional_runtime() {
    let cli = Cli::try_parse_from(["patchlane", "task", "--runtime", "codex", "Design run store"])
        .expect("task command should parse");

    match cli.command {
        TopLevelCommand::Task(task) => {
            assert_eq!(task.runtime, Some(Runtime::Codex));
            assert_eq!(task.objective, "Design run store");
        }
        other => panic!("expected task command, got {:?}", other),
    }
}

#[test]
fn top_level_help_lists_task_command() {
    let output = Command::new(env!("CARGO_BIN_EXE_patchlane"))
        .arg("--help")
        .output()
        .expect("help command should run");
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("task"));
}
// tests/cli.rs
#[path = "cli/main.rs"]
mod cli;
// tests/cli/main.rs
mod command_topology;
mod overview_commands;
mod status_output;
mod task_output;
mod watch_output;
```

- [ ] **Step 2: Run the parsing test to verify it fails**

Run: `cargo test --test cli parses_task_command_with_objective_and_optional_runtime -- --exact`
Expected: FAIL because `task` is not defined yet.

- [ ] **Step 3: Add the new command type and route**

```rust
#[derive(Debug, Parser)]
pub struct TaskCommand {
    #[arg(long, value_name = "RUNTIME")]
    pub runtime: Option<Runtime>,
    #[arg(value_name = "OBJECTIVE")]
    pub objective: String,
}

pub enum TopLevelCommand {
    Task(TaskCommand),
    Swarm(SwarmCommandGroup),
}
```

- [ ] **Step 4: Add the command handler stub**

```rust
pub fn execute(command: TaskCommand) -> CommandOutcome {
    CommandOutcome::success(format!("task queued: {}", command.objective))
}
```

- [ ] **Step 5: Run the focused CLI tests to verify they pass**

Run: `cargo test --test cli parses_task_command_with_objective_and_optional_runtime -- --exact`
Expected: PASS.

Run: `cargo test --test cli top_level_help_lists_task_command -- --exact`
Expected: PASS with the new command present in help output.

- [ ] **Step 6: Commit**

```bash
git add src/cli.rs src/commands/mod.rs src/commands/task.rs src/lib.rs tests/cli.rs tests/cli/main.rs tests/cli/task_output.rs tests/cli/command_topology.rs
git commit -m "feat: add patchlane task command surface"
```

### Task 2: Build The Orchestration Domain And Store

**Files:**
- Create: `src/orchestration/mod.rs`
- Create: `src/orchestration/model.rs`
- Create: `src/orchestration/store.rs`
- Modify: `src/store/mod.rs`
- Modify: `src/lib.rs`
- Modify: `src/workspaces/worktree_manager.rs`
- Create: `tests/orchestration.rs`
- Create: `tests/orchestration/main.rs`
- Create: `tests/orchestration/store.rs`
- Modify: `tests/workspaces/worktree_manager.rs`

- [ ] **Step 1: Write the failing persistence test**

```rust
#[test]
fn orchestration_store_persists_run_agents_checkpoints_artifacts_events_and_logs() {
    let root = temp_root();
    let run = PersistedTaskRun::fixture("run-001");
    let agent = PersistedAgent::fixture("agent-brainstorm");
    let checkpoint = PersistedCheckpoint::approval("checkpoint-001", "after-brainstorming");
    let artifact = PersistedArtifact::spec("artifact-001", "agent-brainstorm", "docs/spec.md");

    let run_dir = create_task_run(&root, &run).expect("run should persist");
    write_agent(&run_dir, &agent).expect("agent should persist");
    write_checkpoint(&run_dir, &checkpoint).expect("checkpoint should persist");
    write_artifact(&run_dir, &artifact).expect("artifact should persist");
    append_task_event(&run_dir, &PersistedTaskEvent::phase("run-001", Some("agent-brainstorm"), "brainstorming"))
        .expect("event should persist");
    fs::write(run_dir.join("logs/agent-brainstorm-stdout.log"), "spec draft\n")
        .expect("stdout log should persist");
    fs::write(run_dir.join("logs/agent-brainstorm-stderr.log"), "")
        .expect("stderr log should persist");

    let snapshot = load_task_snapshot(&run_dir).expect("snapshot should load");
    assert_eq!(snapshot.run.current_phase, "brainstorming");
    assert_eq!(snapshot.agents[0].pid, Some(1234));
    assert_eq!(snapshot.checkpoints[0].prompt_text, "Approve? [y/n]");
    assert_eq!(snapshot.artifacts[0].producing_agent_id, "agent-brainstorm");
    assert_eq!(snapshot.events[0].event_type, "phase");
    assert!(run_dir.join("logs/agent-brainstorm-stdout.log").is_file());
    assert!(run_dir.join("logs/agent-brainstorm-stderr.log").is_file());
}
```

- [ ] **Step 2: Run the store test to verify it fails**

Run: `cargo test --test orchestration orchestration_store_persists_run_agents_checkpoints_artifacts_events_and_logs -- --exact`
Expected: FAIL because the orchestration store does not exist yet.

- [ ] **Step 3: Define the persisted run, agent, checkpoint, artifact, and event types**

```rust
pub enum OrchestratorState {
    Queued,
    Running,
    WaitingForInput,
    WaitingForApproval,
    InReview,
    Done,
    Failed,
}

pub struct PersistedTaskRun {
    pub run_id: String,
    pub objective: String,
    pub runtime: String,
    pub current_phase: String,
    pub overall_state: OrchestratorState,
    pub blocking_reason: Option<String>,
    pub workspace_root: String,
    pub workspace_policy: String,
    pub default_isolation: bool,
    pub created_at: String,
    pub updated_at: String,
}

pub struct PersistedAgent {
    pub agent_id: String,
    pub run_id: String,
    pub parent_agent_id: Option<String>,
    pub role: String,
    pub current_phase: String,
    pub current_state: OrchestratorState,
    pub runtime: String,
    pub workspace_path: String,
    pub pid: Option<u32>,
    pub related_artifact_ids: Vec<String>,
    pub stdout_log: String,
    pub stderr_log: String,
    pub created_at: String,
    pub updated_at: String,
}
```

- [ ] **Step 4: Include the remaining spec-required fields explicitly**

```rust
pub struct PersistedCheckpoint {
    pub checkpoint_id: String,
    pub run_id: String,
    pub phase: String,
    pub target_kind: String,
    pub target_ref: String,
    pub requested_by: String,
    pub status: CheckpointStatus,
    pub prompt_text: String,
    pub response: Option<String>,
    pub note: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

pub struct PersistedArtifact {
    pub artifact_id: String,
    pub run_id: String,
    pub producing_agent_id: String,
    pub artifact_type: String,
    pub path: String,
    pub created_at: String,
}

pub struct PersistedTaskEvent {
    pub event_id: String,
    pub run_id: String,
    pub agent_id: Option<String>,
    pub event_type: AgentEventType,
    pub payload_summary: String,
    pub timestamp: String,
}

pub enum CheckpointStatus {
    Pending,
    Approved,
    Rejected,
    Expired,
}

pub enum AgentEventType {
    Start,
    Phase,
    WaitingInput,
    WaitingApproval,
    Artifact,
    ReviewStart,
    ReviewPass,
    ReviewFail,
    Done,
    Fail,
    CheckpointDecision,
}
```

- [ ] **Step 5: Implement explicit file-backed read/write helpers**

```rust
pub fn create_task_run(root: &Path, run: &PersistedTaskRun) -> io::Result<PathBuf> {
    let run_dir = root.join(&run.run_id);
    fs::create_dir_all(run_dir.join("agents"))?;
    fs::create_dir_all(run_dir.join("checkpoints"))?;
    fs::create_dir_all(run_dir.join("artifacts"))?;
    fs::create_dir_all(run_dir.join("logs"))?;
    write_json(run_dir.join("run.json"), run)?;
    Ok(run_dir)
}

pub fn load_task_snapshot(run_dir: &Path) -> io::Result<TaskSnapshot> {
    Ok(TaskSnapshot {
        run: read_json(run_dir.join("run.json"))?,
        agents: read_json_dir(run_dir.join("agents"))?,
        checkpoints: read_json_dir(run_dir.join("checkpoints"))?,
        artifacts: read_json_dir(run_dir.join("artifacts"))?,
        events: read_jsonl(run_dir.join("events.jsonl"))?,
    })
}
```

- [ ] **Step 6: Write the failing workspace-policy test**

```rust
#[test]
fn orchestration_store_persists_workspace_policy() {
    let snapshot = load_task_snapshot(&run_dir).expect("snapshot should load");
    assert_eq!(snapshot.run.workspace_policy, "isolated_by_default");
    assert!(snapshot.run.default_isolation);
}

#[test]
fn workspace_policy_allocates_isolated_workspaces_only_for_risky_subtasks() {
    let policy = WorkspacePolicy::isolated_by_default();
    let safe = policy.allocate(SubtaskRisk::Low, "agent-brainstorm").expect("safe workspace");
    let risky = policy.allocate(SubtaskRisk::High, "agent-implementer").expect("risky workspace");
    assert!(safe.path.ends_with("session-root"));
    assert!(risky.path.contains("worktrees"));
}
```

- [ ] **Step 7: Implement the workspace allocation behavior**

```rust
pub fn allocate(policy: WorkspacePolicy, risk: SubtaskRisk, agent_id: &str) -> io::Result<WorkspaceAllocation> {
    match (policy.default_isolation, risk) {
        (true, SubtaskRisk::High | SubtaskRisk::Medium) => allocate_workspace(root, run_id, agent_id),
        _ => Ok(WorkspaceAllocation::session_root(root.join("session-root"))),
    }
}
```

- [ ] **Step 8: Keep the on-disk layout explicit**

```text
run.json
agents/<agent-id>.json
checkpoints/<checkpoint-id>.json
artifacts/<artifact-id>.json
events.jsonl
logs/<agent-id>-stdout.log
logs/<agent-id>-stderr.log
```

- [ ] **Step 9: Run the orchestration store tests**

Run: `cargo test --test orchestration orchestration_store_persists_run_agents_checkpoints_artifacts_events_and_logs -- --exact`
Expected: PASS with all persisted entities round-tripping through `load_task_snapshot`.

- [ ] **Step 10: Commit**

```bash
git add src/orchestration/mod.rs src/orchestration/model.rs src/orchestration/store.rs src/store/mod.rs src/lib.rs src/workspaces/worktree_manager.rs tests/orchestration.rs tests/orchestration/main.rs tests/orchestration/store.rs tests/workspaces/worktree_manager.rs
git commit -m "feat: add orchestration persistence model"
```

### Task 3: Implement Runtime Resolution And Approval Checkpoints

**Files:**
- Create: `src/orchestration/runtime.rs`
- Create: `src/orchestration/approval.rs`
- Create: `src/orchestration/checkpoints.rs`
- Modify: `src/commands/task.rs`
- Modify: `tests/orchestration/main.rs`
- Create: `tests/orchestration/runtime.rs`
- Create: `tests/orchestration/checkpoints.rs`
- Modify: `tests/cli/task_output.rs`

- [ ] **Step 1: Write the failing runtime-resolution tests**

```rust
#[test]
fn explicit_runtime_wins_over_detection() {
    let resolved = resolve_runtime(Some(Runtime::Claude), DetectionContext::codex())
        .expect("runtime should resolve");
    assert_eq!(resolved.runtime_label, "claude");
}

#[test]
fn ambiguous_detection_requires_confirmation() {
    let resolved = resolve_runtime(None, DetectionContext::ambiguous())
        .expect("ambiguous detection should yield a pending confirmation");
    assert_eq!(resolved.state, RuntimeResolutionState::WaitingForConfirmation);
    assert_eq!(resolved.confirmation_prompt.as_deref(), Some("Detected both codex and claude contexts. Use codex? [y/n]"));
}

#[test]
fn detection_failure_marks_run_blocked() {
    let error = resolve_runtime(None, DetectionContext::missing()).unwrap_err();
    assert_eq!(error.kind, RuntimeResolutionErrorKind::Blocked);
}
```

- [ ] **Step 2: Run the runtime tests to verify they fail**

Run: `cargo test --test orchestration ambiguous_detection_requires_confirmation -- --exact`
Expected: FAIL because the resolver is not implemented.

- [ ] **Step 3: Implement runtime resolution with confirmation-required output**

```rust
pub fn resolve_runtime(
    explicit: Option<Runtime>,
    detection: DetectionContext,
) -> Result<ResolvedRuntime, RuntimeResolutionError> {
    if let Some(runtime) = explicit {
        return Ok(ResolvedRuntime::resolved(runtime));
    }

    match detection.resolve()? {
        DetectionResult::Resolved(runtime) => Ok(ResolvedRuntime::resolved(runtime)),
        DetectionResult::Ambiguous { preferred, prompt } => Ok(ResolvedRuntime {
            runtime: preferred,
            runtime_label: preferred.label().to_owned(),
            state: RuntimeResolutionState::WaitingForConfirmation,
            confirmation_prompt: Some(prompt),
        }),
        DetectionResult::Unavailable(reason) => Err(RuntimeResolutionError::blocked(reason)),
    }
}
```

- [ ] **Step 4: Write failing checkpoint tests for all v1 checkpoint classes**

```rust
#[test]
fn checkpoint_builder_covers_v1_checkpoint_classes() {
    let checkpoints = build_phase_checkpoints("run-001", "agent-review");
    assert!(checkpoints.iter().any(|checkpoint| checkpoint.phase == "after-brainstorming"));
    assert!(checkpoints.iter().any(|checkpoint| checkpoint.phase == "after-writing-plans"));
    assert!(checkpoints.iter().any(|checkpoint| checkpoint.phase == "request-more-information"));
    assert!(checkpoints.iter().any(|checkpoint| checkpoint.phase == "review-intervention"));
    assert!(checkpoints.iter().any(|checkpoint| checkpoint.phase == "before-branch-finishing"));
}

#[test]
fn rejected_checkpoint_moves_run_into_waiting_for_input() {
    let updated = handle_approval_input("n", pending_checkpoint(), &run_dir)
        .expect("rejection should persist");
    let snapshot = load_task_snapshot(&run_dir).expect("snapshot should load");
    assert_eq!(updated.status, "rejected");
    assert_eq!(snapshot.run.overall_state, "waiting_for_input");
}

#[test]
fn approved_checkpoint_persists_requester_target_and_continues_run() {
    let updated = handle_approval_input("y", pending_checkpoint(), &run_dir)
        .expect("approval should persist");
    let snapshot = load_task_snapshot(&run_dir).expect("snapshot should load");
    assert_eq!(updated.status, "approved");
    assert_eq!(updated.requested_by, "agent-plan");
    assert_eq!(updated.target_ref, "artifact-plan");
    assert_eq!(snapshot.run.overall_state, "running");
}
```

- [ ] **Step 5: Implement approval input persistence**

```rust
pub fn handle_approval_input(
    input: &str,
    checkpoint: PersistedCheckpoint,
    run_dir: &Path,
) -> io::Result<PersistedCheckpoint> {
    if !matches!(input.trim(), "y" | "Y" | "n" | "N") {
        append_task_event(run_dir, &PersistedTaskEvent::agent(
            &checkpoint.run_id,
            Some(checkpoint.requested_by.clone()),
            "waiting-input",
            "invalid approval input; re-prompting for y/n".to_owned(),
        ))?;
        return Ok(checkpoint);
    }
    let updated = PersistedCheckpoint {
        status: match input.trim() {
            "y" | "Y" => "approved".to_owned(),
            "n" | "N" => "rejected".to_owned(),
            _ => "pending".to_owned(),
        },
        response: Some(input.trim().to_ascii_lowercase()),
        updated_at: timestamp_now(),
        ..checkpoint
    };
    write_checkpoint(run_dir, &updated)?;
    append_task_event(run_dir, &PersistedTaskEvent::decision(&updated))?;
    update_run_state(run_dir, |run| {
        run.overall_state = if updated.status == "rejected" {
            "waiting_for_input".to_owned()
        } else if updated.status == "approved" {
            "running".to_owned()
        } else {
            run.overall_state.clone()
        };
        run.blocking_reason = (updated.status == "rejected")
            .then(|| format!("checkpoint {} rejected", updated.checkpoint_id));
        run.updated_at = updated.updated_at.clone();
    })?;
    Ok(updated)
}
```

- [ ] **Step 6: Wire the task command to pause for runtime confirmation and approvals**

```rust
if resolution.state == RuntimeResolutionState::WaitingForConfirmation {
    println!("{}", resolution.confirmation_prompt.as_deref().unwrap_or("Use detected runtime? [y/n]"));
    let answer = io.read_line()?;
    let runtime = apply_runtime_confirmation(answer, resolution)?;
}
```

- [ ] **Step 6a: Register the new orchestration test modules**

```rust
// tests/orchestration/main.rs
mod checkpoints;
mod runtime;
mod store;
```

- [ ] **Step 7: Run the focused runtime and checkpoint tests**

Run: `cargo test --test orchestration explicit_runtime_wins_over_detection -- --exact`
Expected: PASS.

Run: `cargo test --test orchestration detection_failure_marks_run_blocked -- --exact`
Expected: PASS.

Run: `cargo test --test orchestration checkpoint_builder_covers_v1_checkpoint_classes -- --exact`
Expected: PASS.

Run: `cargo test --test orchestration rejected_checkpoint_moves_run_into_waiting_for_input -- --exact`
Expected: PASS.

Run: `cargo test --test cli task_command_surfaces_runtime_confirmation_prompt -- --exact`
Expected: PASS with `Approve? [y/n]` or the runtime confirmation prompt rendered correctly.

- [ ] **Step 8: Commit**

```bash
git add src/orchestration/runtime.rs src/orchestration/approval.rs src/orchestration/checkpoints.rs src/commands/task.rs tests/orchestration/main.rs tests/orchestration/runtime.rs tests/orchestration/checkpoints.rs tests/cli/task_output.rs
git commit -m "feat: add runtime resolution and checkpoint approvals"
```

### Task 4: Implement The High-Level Agent Wrapper And Workflow Runner

**Files:**
- Create: `src/commands/agent_event.rs`
- Create: `src/orchestration/agent_wrapper.rs`
- Create: `src/orchestration/phases.rs`
- Create: `src/orchestration/workflow.rs`
- Create: `src/orchestration/recovery.rs`
- Modify: `src/cli.rs`
- Modify: `src/commands/mod.rs`
- Modify: `src/runtime/launcher.rs`
- Modify: `src/commands/task.rs`
- Modify: `tests/orchestration/main.rs`
- Create: `tests/orchestration/agent_wrapper.rs`
- Create: `tests/orchestration/workflow.rs`
- Modify: `tests/cli/task_output.rs`
- Modify: `tests/e2e/cli_contract.rs`

- [ ] **Step 1: Write the failing wrapper contract test**

```rust
#[test]
fn wrapper_phase_and_artifact_events_update_persisted_agent_state() {
    let mut wrapper = AgentWrapper::new(run_dir.clone(), persisted_agent("agent-plan"));
    wrapper.start().expect("start event should persist");
    wrapper.review_start("spec-review").expect("review start should persist");
    wrapper.phase("writing-plans").expect("phase event should persist");
    wrapper.artifact("plan", "docs/superpowers/plans/plan.md")
        .expect("artifact event should persist");
    wrapper.waiting_approval("checkpoint-001", "Approve? [y/n]")
        .expect("waiting approval should persist");
    wrapper.waiting_input("Need approval note").expect("waiting input should persist");
    wrapper.review_pass("looks good").expect("review pass should persist");
    wrapper.review_fail("missing artifact").expect("review fail should persist");
    wrapper.fail("launcher exited").expect("fail should persist");
    wrapper.done("plan accepted").expect("done should persist");

    let snapshot = load_task_snapshot(&run_dir).expect("snapshot should load");
    assert_eq!(snapshot.agents[0].current_phase, "writing-plans");
    assert_eq!(snapshot.artifacts[0].artifact_type, "plan");
    assert!(snapshot.events.iter().any(|event| event.event_type == "waiting-approval"));
    assert!(snapshot.events.iter().any(|event| event.event_type == "review-start"));
    assert!(snapshot.events.iter().any(|event| event.event_type == "waiting-input"));
    assert!(snapshot.events.iter().any(|event| event.event_type == "review-pass"));
    assert!(snapshot.events.iter().any(|event| event.event_type == "review-fail"));
    assert!(snapshot.events.iter().any(|event| event.event_type == "fail"));
    assert!(snapshot.events.iter().any(|event| event.event_type == "done"));
}
```

- [ ] **Step 2: Run the wrapper and workflow tests to verify they fail**

Run: `cargo test --test orchestration wrapper_phase_and_artifact_events_update_persisted_agent_state -- --exact`
Expected: FAIL because the wrapper and workflow runner do not exist yet.

- [ ] **Step 2a: Register the new wrapper and workflow test modules**

```rust
// tests/orchestration/main.rs
mod agent_wrapper;
mod checkpoints;
mod runtime;
mod store;
mod workflow;
```

- [ ] **Step 3: Implement concrete wrapper methods**

```rust
pub fn waiting_approval(&mut self, checkpoint_id: &str, prompt: &str) -> io::Result<()> {
    self.agent.current_state = "waiting_for_approval".to_owned();
    self.agent.updated_at = timestamp_now();
    write_agent(&self.run_dir, &self.agent)?;
    append_task_event(&self.run_dir, &PersistedTaskEvent::agent(
        &self.agent.run_id,
        Some(self.agent.agent_id.clone()),
        "waiting-approval",
        format!("{checkpoint_id}: {prompt}"),
    ))
}

pub fn waiting_input(&mut self, prompt: &str) -> io::Result<()> {
    self.emit_state("waiting-input", "waiting_for_input", prompt)
}

pub fn review_start(&mut self, summary: &str) -> io::Result<()> {
    self.emit_state("review-start", "in_review", summary)
}

pub fn review_pass(&mut self, summary: &str) -> io::Result<()> {
    self.emit_state("review-pass", "in_review", summary)
}

pub fn review_fail(&mut self, summary: &str) -> io::Result<()> {
    self.emit_state("review-fail", "failed", summary)
}

pub fn done(&mut self, summary: &str) -> io::Result<()> {
    self.emit_terminal_state("done", "done", summary)
}

pub fn fail(&mut self, message: &str) -> io::Result<()> {
    self.emit_terminal_state("fail", "failed", message)
}
```

- [ ] **Step 4: Extend the launcher for high-level agents**

```rust
pub struct AgentLaunchRequest {
    pub runtime: Runtime,
    pub role: String,
    pub prompt: String,
    pub workspace: PathBuf,
    pub logs_dir: PathBuf,
    pub run_dir: PathBuf,
}
```

- [ ] **Step 4a: Add the cross-process `patchlane agent-event` reporting command**

```rust
pub enum TopLevelCommand {
    Task(TaskCommand),
    AgentEvent(AgentEventCommand),
    Swarm(SwarmCommandGroup),
}

pub fn execute(command: AgentEventCommand) -> CommandOutcome {
    persist_agent_event_from_cli(command).map(CommandOutcome::success)
}
```

- [ ] **Step 5: Persist the brainstorming artifact and first approval checkpoint**

```rust
let brainstorming = run_brainstorming_phase(&context)?;
persist_artifact(&context.run_dir, ArtifactType::Spec, brainstorming.spec_path.clone())?;
await_checkpoint(&context, checkpoint_after_brainstorming(&brainstorming), io)?;
```

- [ ] **Step 6: Persist the plan artifact and second approval checkpoint**

```rust
let plan = run_writing_plans_phase(&context, &brainstorming)?;
persist_artifact(&context.run_dir, ArtifactType::Plan, plan.plan_path.clone())?;
await_checkpoint(&context, checkpoint_after_writing_plans(&plan), io)?;
```

- [ ] **Step 7: Run execution, partial blocking, and branch-finishing gates**

```rust
let implementation = run_subagent_development_phase(&context, &plan)?;
if implementation.requires_input {
    await_checkpoint(&context, checkpoint_for_requested_input(&implementation), io)?;
}
if implementation.review_requires_intervention {
    await_checkpoint(&context, checkpoint_for_review_intervention(&implementation), io)?;
}
if implementation.requires_finish_confirmation {
    await_checkpoint(&context, checkpoint_before_branch_finishing(&implementation), io)?;
}
for independent_agent in implementation.independent_agents_still_running() {
    persist_agent_state(&context.run_dir, independent_agent, "running")?;
}
run_finishing_branch_phase(&context, &implementation)?;
Ok(TaskRunResult::success(context.run_id))
```

let child_prompt = format!(
    "Use `patchlane agent-event start --run-id {run_id} --agent-id {agent_id}` before work and report `phase`, `artifact`, `waiting-approval`, `waiting-input`, `review-start`, `review-pass`, `review-fail`, `done`, or `fail` as state changes occur."
);
```

- [ ] **Step 8: Add the failing restart-recovery test**

```rust
#[test]
fn restart_recovery_reconstructs_waiting_for_approval_runs() {
    let run_dir = persisted_waiting_for_approval_run();
    let recovered = recover_run_state(&run_dir).expect("run should recover");
    assert_eq!(recovered.run.overall_state, "waiting_for_approval");
    assert_eq!(recovered.pending_checkpoint.unwrap().phase, "after-writing-plans");
}
```

- [ ] **Step 9: Add restart recovery from persisted state**

```rust
pub fn recover_run_state(run_dir: &Path) -> io::Result<RecoveredRunState> {
    let snapshot = load_task_snapshot(run_dir)?;
    let pending_checkpoint = snapshot
        .checkpoints
        .iter()
        .filter(|checkpoint| checkpoint.status == "pending")
        .max_by_key(|checkpoint| checkpoint.updated_at.clone())
        .cloned();

    Ok(RecoveredRunState {
        run: snapshot.run.clone(),
        pending_checkpoint,
        blocked_agents: snapshot
            .agents
            .iter()
            .filter(|agent| matches!(agent.current_state.as_str(), "waiting_for_input" | "waiting_for_approval" | "failed"))
            .cloned()
            .collect(),
        latest_event: snapshot.events.last().cloned(),
    })
}
```

- [ ] **Step 10: Write the end-to-end workflow test**

```rust
#[test]
fn task_command_follows_brainstorming_approval_plan_approval_execution_flow() {
    let output = run_command(
        &["task", "--runtime", "codex", "Ship orchestration flow"],
        &state_root,
        "success",
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.matches("Approve? [y/n]").count() >= 2);
    let snapshot = latest_task_snapshot(&state_root).expect("snapshot should load");
    assert!(snapshot.artifacts.iter().any(|artifact| artifact.artifact_type == "spec"));
    assert!(snapshot.artifacts.iter().any(|artifact| artifact.artifact_type == "plan"));
    assert!(snapshot.checkpoints.iter().any(|checkpoint| checkpoint.phase == "after-brainstorming"));
    assert!(snapshot.checkpoints.iter().any(|checkpoint| checkpoint.phase == "after-writing-plans"));
    assert_eq!(snapshot.run.current_phase, "finishing-a-development-branch");
}

#[test]
fn blocked_agent_does_not_stop_independent_agents() {
    let snapshot = run_fixture_with_one_blocked_and_one_running_agent();
    assert_eq!(snapshot.blocked_agents.len(), 1);
    assert!(snapshot.agents.iter().any(|agent| agent.current_state == "running"));
}
```

- [ ] **Step 11: Run the workflow and e2e tests**

Run: `cargo test --test orchestration task_command_follows_brainstorming_approval_plan_approval_execution_flow -- --exact`
Expected: PASS.

Run: `cargo test --test orchestration blocked_agent_does_not_stop_independent_agents -- --exact`
Expected: PASS.

Run: `cargo test --test orchestration wrapper_phase_and_artifact_events_update_persisted_agent_state -- --exact`
Expected: PASS with all required high-level wrapper events persisted.

Run: `cargo test --test orchestration restart_recovery_reconstructs_waiting_for_approval_runs -- --exact`
Expected: PASS with recovery reconstructing enough state to resume or report blocking accurately.

Run: `cargo test --test e2e cli_contract_covers_task_workflow_and_persisted_artifacts -- --exact`
Expected: PASS with task artifacts, checkpoints, events, and logs validated.

- [ ] **Step 12: Commit**

```bash
git add src/commands/agent_event.rs src/cli.rs src/commands/mod.rs src/orchestration/agent_wrapper.rs src/orchestration/phases.rs src/orchestration/workflow.rs src/orchestration/recovery.rs src/runtime/launcher.rs src/commands/task.rs tests/orchestration/main.rs tests/orchestration/agent_wrapper.rs tests/orchestration/workflow.rs tests/cli/task_output.rs tests/e2e/cli_contract.rs
git commit -m "feat: orchestrate high-level task workflow"
```

## Chunk 2: Observability, TUI, And Documentation

### Task 5: Rebuild Status, Watch, And Board Around The New Store

**Files:**
- Modify: `src/events/run_events.rs`
- Modify: `src/commands/status.rs`
- Modify: `src/commands/watch.rs`
- Modify: `src/commands/board.rs`
- Modify: `src/renderers/status_renderer.rs`
- Modify: `src/renderers/watch_renderer.rs`
- Modify: `tests/cli/main.rs`
- Modify: `tests/cli/status_output.rs`
- Modify: `tests/cli/watch_output.rs`
- Modify: `tests/cli/overview_commands.rs`
- Create: `tests/cli/board_output.rs`

- [ ] **Step 1: Write failing renderer tests for agent-aware snapshots**

```rust
#[test]
fn status_renders_phase_blockers_and_agent_rows() {
    let output = render_status_snapshot(&fixture_status_snapshot());
    assert!(output.contains("phase: writing-plans"));
    assert!(output.contains("waiting_for_approval"));
    assert!(output.contains("brainstorming"));
}

#[test]
fn watch_keeps_blocked_and_failed_agent_events() {
    let output = render_watch_events(&fixture_watch_events_with_blockers());
    assert!(output.contains("waiting_for_input"));
    assert!(output.contains("artifact write failed"));
}

#[test]
fn board_surfaces_current_run_and_blocked_agents() {
    let output = execute_board_against_fixture_snapshot();
    assert!(output.contains("Active Runs"));
    assert!(output.contains("blocked"));
    assert!(output.contains("agent-plan"));
}
```

- [ ] **Step 2: Run the renderer tests to verify they fail**

Run: `cargo test --test cli status_renders_phase_blockers_and_agent_rows -- --exact`
Expected: FAIL because current status output is shard-oriented.

Run: `cargo test --test cli watch_keeps_blocked_and_failed_agent_events -- --exact`
Expected: FAIL because watch is still oriented around the legacy event model.

Run: `cargo test --test cli board_surfaces_current_run_and_blocked_agents -- --exact`
Expected: FAIL because board still renders fixture text.

- [ ] **Step 3: Redefine the derived status snapshot**

```rust
pub struct AgentSnapshot {
    pub id: String,
    pub role: String,
    pub phase: String,
    pub state: String,
    pub runtime: String,
    pub detail: String,
}
```

- [ ] **Step 4: Update status and watch rendering**

```text
Run
  run-001 (waiting_for_approval)
  runtime: codex
  phase: writing-plans
  objective: Ship orchestration flow

Agents
  id                  role             phase            state
  agent-brainstorm    brainstorming    done             done
  agent-plan          writing-plans    in_review        waiting_for_approval
```

- [ ] **Step 5: Replace the board fixture surface with persisted-state summaries**

```rust
assert!(board_output.contains("Active Runs"));
assert!(board_output.contains("waiting_for_approval"));
assert!(board_output.contains("agent-plan"));
assert!(board_output.contains("waiting_for_input"));
assert!(board_output.contains("artifact write failed"));
```

- [ ] **Step 6: Run the focused CLI output tests**

Run: `cargo test --test cli status_renders_phase_blockers_and_agent_rows -- --exact`
Expected: PASS.

Run: `cargo test --test cli watch_keeps_blocked_and_failed_agent_events -- --exact`
Expected: PASS.

Run: `cargo test --test cli board_surfaces_current_run_and_blocked_agents -- --exact`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add src/events/run_events.rs src/commands/status.rs src/commands/watch.rs src/commands/board.rs src/renderers/status_renderer.rs src/renderers/watch_renderer.rs tests/cli/main.rs tests/cli/status_output.rs tests/cli/watch_output.rs tests/cli/overview_commands.rs tests/cli/board_output.rs
git commit -m "feat: render orchestration state in cli surfaces"
```

### Task 6: Add The Minimal Read-Only TUI

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/cli.rs`
- Modify: `src/commands/mod.rs`
- Create: `src/commands/tui.rs`
- Create: `src/tui/mod.rs`
- Create: `src/tui/app.rs`
- Create: `src/tui/store.rs`
- Create: `src/tui/render.rs`
- Create: `src/tui/logs.rs`
- Modify: `src/lib.rs`
- Create: `tests/tui.rs`
- Create: `tests/tui/main.rs`
- Create: `tests/tui/render.rs`
- Create: `tests/tui/logs.rs`
- Create: `tests/support/mod.rs`
- Create: `tests/support/task_fixtures.rs`
- Create: `tests/fixtures/agent-plan-stdout.log`
- Create: `tests/fixtures/agent-plan-stderr.log`
- Modify: `tests/cli/command_topology.rs`

- [ ] **Step 1: Write the failing selected-agent detail test**

```rust
#[test]
fn selected_agent_view_includes_required_detail_fields() {
    let app = TuiApp::from_snapshot(fixture_task_snapshot());
    let detail = app.selected_agent_detail().expect("detail should exist");

    assert_eq!(detail.current_phase, "writing-plans");
    assert_eq!(detail.current_state, "waiting_for_approval");
    assert!(detail.timeline.iter().any(|event| event.event_type == "waiting-approval"));
    assert!(detail.blockers.iter().any(|blocker| blocker.contains("Approve? [y/n]")));
    assert!(detail.artifacts.iter().any(|artifact| artifact.artifact_type == "plan"));
    assert!(detail.artifacts.iter().any(|artifact| artifact.path.ends_with("docs/superpowers/plans/plan.md")));
    assert!(detail.stdout_log.ends_with("agent-plan-stdout.log"));
    assert!(detail.stderr_log.ends_with("agent-plan-stderr.log"));
}
```

- [ ] **Step 2: Run the TUI tests to verify they fail**

Run: `cargo test --test tui selected_agent_view_includes_required_detail_fields -- --exact`
Expected: FAIL because the TUI modules and dependencies do not exist.

- [ ] **Step 3: Add the TUI dependencies and command**

```toml
ratatui = "0.29"
crossterm = "0.28"
```

```rust
pub enum TopLevelCommand {
    Task(TaskCommand),
    Tui,
    Swarm(SwarmCommandGroup),
}
```

- [ ] **Step 4: Write the failing persisted-store loading test**

```rust
#[test]
fn tui_loads_runs_from_persisted_store_and_refreshes() {
    let state_root = persisted_state_root_with_two_runs();
    let mut app = TuiApp::load_from_store(&state_root).expect("app should load");
    assert_eq!(app.runs.len(), 2);
    app.refresh().expect("refresh should succeed");
    assert!(app.agent_rows().iter().any(|row| row.state == "waiting_for_input"));
}
```

- [ ] **Step 5: Write the failing multi-run list test**

```rust
#[test]
fn tui_lists_runs_and_agents_and_highlights_blockers() {
    let app = TuiApp::from_snapshot(fixture_task_snapshot_with_blockers());
    assert_eq!(app.runs.len(), 2);
    assert!(app.agent_rows().iter().any(|row| row.state == "failed"));
    assert!(app.agent_rows().iter().any(|row| row.state == "waiting_for_input"));
}
```

- [ ] **Step 6: Implement the TUI app and persisted-store reader**

```rust
pub struct TuiApp {
    pub runs: Vec<RunListItem>,
    pub selected_run: usize,
    pub selected_agent: usize,
    pub run_snapshots: Vec<TaskSnapshot>,
    pub active_snapshot: TaskSnapshot,
}

pub fn load_runs(state_root: &Path) -> io::Result<Vec<TaskSnapshot>> {
    load_all_task_runs(state_root)
}
```

- [ ] **Step 7: Write the failing render and log-tail tests**

```rust
#[test]
fn render_frame_shows_timeline_artifacts_and_log_tail() {
    let frame = render_to_test_buffer(fixture_task_snapshot_with_blockers());
    assert!(frame.contains("Timeline"));
    assert!(frame.contains("Artifacts"));
    assert!(frame.contains("docs/superpowers/plans/plan.md"));
    assert!(frame.contains("Logs"));
}

#[test]
fn tail_log_returns_latest_lines_for_selected_agent() {
    let lines = tail_log(Path::new("fixtures/agent-plan-stdout.log"), 2)
        .expect("tail should succeed");
    assert_eq!(lines, vec!["phase: writing-plans".to_owned(), "Approve? [y/n]".to_owned()]);
}

#[test]
fn stderr_log_panel_is_available_for_selected_agent() {
    let detail = TuiApp::from_snapshot(fixture_task_snapshot())
        .selected_agent_detail()
        .expect("detail should exist");
    assert!(detail.stderr_log.ends_with("agent-plan-stderr.log"));
}
```

- [ ] **Step 8: Implement log tailing and selected-agent detail panels**

```rust
pub fn tail_log(path: &Path, max_lines: usize) -> io::Result<Vec<String>> {
    fs::read_to_string(path).map(|content| content.lines().rev().take(max_lines).rev().map(str::to_owned).collect())
}

pub struct SelectedAgentDetail {
    pub current_phase: String,
    pub current_state: String,
    pub blockers: Vec<String>,
    pub timeline: Vec<PersistedTaskEvent>,
    pub artifacts: Vec<PersistedArtifact>,
    pub stdout_log: String,
    pub stderr_log: String,
}

pub fn selected_agent_detail(&self) -> Option<SelectedAgentDetail> {
    let agent = self.active_snapshot.agents.get(self.selected_agent)?.clone();
    Some(SelectedAgentDetail {
        current_phase: agent.current_phase.clone(),
        current_state: agent.current_state.clone(),
        blockers: self.active_snapshot.blockers_for(&agent.agent_id),
        timeline: self.active_snapshot.events_for(&agent.agent_id),
        artifacts: self.active_snapshot.artifacts_for(&agent.agent_id),
        stdout_log: agent.stdout_log.clone(),
        stderr_log: agent.stderr_log.clone(),
    })
}
```

- [ ] **Step 9: Write the failing interaction test**

```rust
#[test]
fn key_handling_switches_panes_and_selection() {
    let mut app = TuiApp::from_snapshot(fixture_task_snapshot());
    app.handle_key(KeyCode::Tab);
    app.handle_key(KeyCode::Char('j'));
    assert_eq!(app.active_pane(), Pane::AgentList);
}
```

Run: `cargo test --test tui key_handling_switches_panes_and_selection -- --exact`
Expected: FAIL until pane switching and selection movement are implemented.

- [ ] **Step 10: Implement the render loop and key bindings**

```text
q       quit
j / k   move selection
tab     switch pane
r       refresh from store
```

- [ ] **Step 11: Run the focused TUI and topology tests**

Run: `cargo test --test tui tui_loads_runs_from_persisted_store_and_refreshes -- --exact`
Expected: PASS.

Run: `cargo test --test tui tui_lists_runs_and_agents_and_highlights_blockers -- --exact`
Expected: PASS.

Run: `cargo test --test tui selected_agent_view_includes_required_detail_fields -- --exact`
Expected: PASS.

Run: `cargo test --test tui render_frame_shows_timeline_artifacts_and_log_tail -- --exact`
Expected: PASS.

Run: `cargo test --test tui tail_log_returns_latest_lines_for_selected_agent -- --exact`
Expected: PASS.

Run: `cargo test --test tui stderr_log_panel_is_available_for_selected_agent -- --exact`
Expected: PASS.

Run: `cargo test --test tui key_handling_switches_panes_and_selection -- --exact`
Expected: PASS.

Run: `cargo test --test cli top_level_help_lists_tui_command -- --exact`
Expected: PASS.

- [ ] **Step 12: Commit**

```bash
git add Cargo.toml src/cli.rs src/commands/mod.rs src/commands/tui.rs src/tui/mod.rs src/tui/app.rs src/tui/store.rs src/tui/render.rs src/tui/logs.rs src/lib.rs tests/tui.rs tests/tui/main.rs tests/tui/render.rs tests/tui/logs.rs tests/support/mod.rs tests/support/task_fixtures.rs tests/fixtures/agent-plan-stdout.log tests/fixtures/agent-plan-stderr.log tests/cli/command_topology.rs
git commit -m "feat: add minimal orchestration tui"
```

### Task 7: Finalize Recovery, Docs, And Full Verification

**Files:**
- Modify: `README.md`
- Modify: `src/orchestration/recovery.rs`
- Modify: `src/commands/task.rs`
- Modify: `src/events/run_events.rs`
- Modify: `src/renderers/status_renderer.rs`
- Modify: `tests/e2e/cli_contract.rs`
- Modify: `tests/orchestration/workflow.rs`
- Modify: `tests/cli/task_output.rs`

- [ ] **Step 1: Write the failing restart-recovery test**

```rust
#[test]
fn restart_recovery_reconstructs_waiting_for_approval_runs() {
    let run_dir = persisted_waiting_for_approval_run();
    let recovered = recover_run_state(&run_dir).expect("run should recover");
    assert_eq!(recovered.run.overall_state, "waiting_for_approval");
    assert_eq!(recovered.pending_checkpoint.unwrap().phase, "after-writing-plans");
}
```

- [ ] **Step 2: Run the recovery test to verify it fails**

Run: `cargo test --test orchestration restart_recovery_reconstructs_waiting_for_approval_runs -- --exact`
Expected: FAIL until pending checkpoint reconstruction and blocked-state rendering are complete.

- [ ] **Step 3: Complete the defined recovery edge cases and blocked user-facing output**

```rust
assert!(recover_run_state(&run_dir)?.pending_checkpoint.is_some());
assert!(recover_run_state(&run_dir)?.blocked_agents.iter().any(|agent| agent.current_state == "waiting_for_input"));
assert!(recover_run_state(&run_dir)?.latest_event.unwrap().event_type == "checkpoint-decision");
assert!(stdout.contains("blocked: approval required"));
assert!(stdout.contains("checkpoint: after-writing-plans"));
assert!(stdout.contains("waiting_for_input"));
```

- [ ] **Step 4: Update `README.md` with the new operator flow**

```bash
cargo run -- task --runtime codex "Ship orchestration flow"
cargo run -- tui
cargo test
```

Document:
- the approval checkpoints and `Approve? [y/n]` prompts after `brainstorming` and `writing-plans`
- the persisted run-store layout: `run.json`, `agents/`, `checkpoints/`, `artifacts/`, `events.jsonl`, `logs/`
- restart recovery and blocked-state reporting behavior
- the TUI as a read-only observability surface over persisted state, not the control plane

- [ ] **Step 5: Run the full verification suite**

Run: `cargo test`
Expected: PASS across CLI, orchestration, runtime, TUI, and e2e tests.

- [ ] **Step 6: Run manual smoke checks**

Run: `cargo run -- task --runtime codex "Create a spec and plan with approvals"`
Expected: The command persists a task run, prints `Approve? [y/n]` at checkpoints, and creates `run.json`, `agents/`, `checkpoints/`, `artifacts/`, `events.jsonl`, and `logs/` under `.patchlane/<run-id>/`.

Run: `cargo run -- tui`
Expected: The TUI opens, lists runs and agents, shows the selected agent's phase/state timeline, highlights blockers and approval waits, and tails log output from persisted files.

- [ ] **Step 7: Commit**

```bash
git add README.md src/orchestration/recovery.rs src/commands/task.rs src/events/run_events.rs src/renderers/status_renderer.rs tests/e2e/cli_contract.rs tests/orchestration/workflow.rs tests/cli/task_output.rs
git commit -m "docs: finalize task orchestration flow"
```
