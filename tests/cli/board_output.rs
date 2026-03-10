use patchlane::orchestration::model::{
    AgentEventType, CheckpointStatus, OrchestratorState, PersistedAgent, PersistedCheckpoint,
    PersistedTaskEvent, PersistedTaskRun,
};
use patchlane::orchestration::store::{
    append_task_event, create_task_run, write_agent, write_checkpoint,
};
use patchlane::store::run_store::{create_run, PersistedRun, PersistedShard};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_root() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("patchlane-board-{unique}"));
    fs::create_dir_all(root.join("tasks")).expect("temp root should be creatable");
    root
}

fn run_command(args: &[&str], state_root: &PathBuf) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_patchlane"))
        .args(args)
        .env("PATCHLANE_STATE_ROOT", state_root)
        .output()
        .expect("CLI should be executable")
}

#[test]
fn board_command_summarizes_persisted_task_runs() {
    let state_root = temp_root();
    let tasks_root = state_root.join("tasks");

    let active_run_dir = create_task_run(
        &tasks_root,
        &PersistedTaskRun {
            run_id: "run-task-001".to_owned(),
            runtime: "codex".to_owned(),
            objective: "Ship orchestration flow".to_owned(),
            current_phase: "writing-plans".to_owned(),
            overall_state: OrchestratorState::WaitingForApproval,
            blocking_reason: Some("checkpoint pending".to_owned()),
            workspace_root: "workspace".to_owned(),
            workspace_policy: "isolated_by_default".to_owned(),
            default_isolation: true,
            created_at: "2026-03-10T10:00:00Z".to_owned(),
            updated_at: "2026-03-10T10:06:00Z".to_owned(),
        },
    )
    .expect("task run should persist");
    write_agent(
        &active_run_dir,
        &PersistedAgent {
            agent_id: "agent-plan".to_owned(),
            run_id: "run-task-001".to_owned(),
            parent_agent_id: None,
            role: "writing-plans".to_owned(),
            current_phase: "writing-plans".to_owned(),
            current_state: OrchestratorState::WaitingForApproval,
            runtime: "codex".to_owned(),
            workspace_path: "workspace/plan".to_owned(),
            pid: None,
            related_artifact_ids: vec![],
            stdout_log: "logs/agent-plan-stdout.log".to_owned(),
            stderr_log: "logs/agent-plan-stderr.log".to_owned(),
            created_at: "2026-03-10T10:00:00Z".to_owned(),
            updated_at: "2026-03-10T10:06:00Z".to_owned(),
        },
    )
    .expect("agent should persist");
    write_checkpoint(
        &active_run_dir,
        &PersistedCheckpoint {
            checkpoint_id: "checkpoint-plan".to_owned(),
            run_id: "run-task-001".to_owned(),
            phase: "after-writing-plans".to_owned(),
            target_kind: "artifact".to_owned(),
            target_ref: "artifact-plan".to_owned(),
            requested_by: "agent-plan".to_owned(),
            status: CheckpointStatus::Pending,
            prompt_text: "Approve? [y/n]".to_owned(),
            response: None,
            note: None,
            created_at: "2026-03-10T10:06:00Z".to_owned(),
            updated_at: "2026-03-10T10:06:00Z".to_owned(),
        },
    )
    .expect("checkpoint should persist");

    let done_run_dir = create_task_run(
        &tasks_root,
        &PersistedTaskRun {
            run_id: "run-task-002".to_owned(),
            runtime: "claude".to_owned(),
            objective: "Finish branch cleanup".to_owned(),
            current_phase: "finishing-a-development-branch".to_owned(),
            overall_state: OrchestratorState::Done,
            blocking_reason: None,
            workspace_root: "workspace".to_owned(),
            workspace_policy: "isolated_by_default".to_owned(),
            default_isolation: true,
            created_at: "2026-03-10T11:00:00Z".to_owned(),
            updated_at: "2026-03-10T11:08:00Z".to_owned(),
        },
    )
    .expect("second task run should persist");
    write_agent(
        &done_run_dir,
        &PersistedAgent {
            agent_id: "agent-finish".to_owned(),
            run_id: "run-task-002".to_owned(),
            parent_agent_id: None,
            role: "finishing-a-development-branch".to_owned(),
            current_phase: "done".to_owned(),
            current_state: OrchestratorState::Done,
            runtime: "claude".to_owned(),
            workspace_path: "workspace/finish".to_owned(),
            pid: Some(5020),
            related_artifact_ids: vec![],
            stdout_log: "logs/agent-finish-stdout.log".to_owned(),
            stderr_log: "logs/agent-finish-stderr.log".to_owned(),
            created_at: "2026-03-10T11:00:00Z".to_owned(),
            updated_at: "2026-03-10T11:08:00Z".to_owned(),
        },
    )
    .expect("second agent should persist");

    let output = run_command(&["swarm", "board"], &state_root);
    assert!(output.status.success(), "board should succeed");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let expected = "\
Board
  active runs: 1
  blocked agents: 1

Active Runs
  run-task-002 done 1 agents objective: Finish branch cleanup
  run-task-001 waiting_for_approval 1 agents objective: Ship orchestration flow

Blocked Agents
  run-task-001 agent-plan writing-plans waiting on after-writing-plans

Next
  use `patchlane swarm status` for the latest run or `patchlane swarm watch` for event flow";
    assert_eq!(stdout.trim(), expected);

    fs::remove_dir_all(state_root).expect("temp root should be removable");
}

#[test]
fn board_command_surfaces_failed_agent_detail_from_persisted_events() {
    let state_root = temp_root();
    let run_dir = create_task_run(
        &state_root.join("tasks"),
        &PersistedTaskRun {
            run_id: "run-task-003".to_owned(),
            runtime: "codex".to_owned(),
            objective: "Handle artifact writes".to_owned(),
            current_phase: "execution".to_owned(),
            overall_state: OrchestratorState::Failed,
            blocking_reason: Some("execution failure".to_owned()),
            workspace_root: "workspace".to_owned(),
            workspace_policy: "isolated_by_default".to_owned(),
            default_isolation: true,
            created_at: "2026-03-10T12:00:00Z".to_owned(),
            updated_at: "2026-03-10T12:05:00Z".to_owned(),
        },
    )
    .expect("task run should persist");
    write_agent(
        &run_dir,
        &PersistedAgent {
            agent_id: "agent-exec".to_owned(),
            run_id: "run-task-003".to_owned(),
            parent_agent_id: None,
            role: "subagent-driven-development".to_owned(),
            current_phase: "execution".to_owned(),
            current_state: OrchestratorState::Failed,
            runtime: "codex".to_owned(),
            workspace_path: "workspace/exec".to_owned(),
            pid: Some(9001),
            related_artifact_ids: vec![],
            stdout_log: "logs/agent-exec-stdout.log".to_owned(),
            stderr_log: "logs/agent-exec-stderr.log".to_owned(),
            created_at: "2026-03-10T12:00:00Z".to_owned(),
            updated_at: "2026-03-10T12:05:00Z".to_owned(),
        },
    )
    .expect("agent should persist");
    append_task_event(
        &run_dir,
        &PersistedTaskEvent {
            event_id: "event-fail".to_owned(),
            run_id: "run-task-003".to_owned(),
            agent_id: Some("agent-exec".to_owned()),
            event_type: AgentEventType::Fail,
            payload_summary: "artifact write failed".to_owned(),
            timestamp: "2026-03-10T12:05:00Z".to_owned(),
        },
    )
    .expect("event should persist");

    let output = run_command(&["swarm", "board"], &state_root);
    assert!(output.status.success(), "board should succeed");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let expected = "\
Board
  active runs: 0
  blocked agents: 1

Active Runs
  run-task-003 failed 1 agents objective: Handle artifact writes

Blocked Agents
  run-task-003 agent-exec subagent-driven-development artifact write failed

Next
  use `patchlane swarm status` for the latest run or `patchlane swarm watch` for event flow";
    assert_eq!(stdout.trim(), expected);

    fs::remove_dir_all(state_root).expect("temp root should be removable");
}

#[test]
fn board_command_skips_malformed_task_runs_and_disambiguates_blocked_agents_by_run() {
    let state_root = temp_root();
    let tasks_root = state_root.join("tasks");

    let malformed_dir = tasks_root.join("bad-run");
    fs::create_dir_all(&malformed_dir).expect("malformed task dir should be creatable");
    fs::write(malformed_dir.join("run.json"), "{not valid json").expect("malformed run should persist");

    for (run_id, updated_at, detail) in [
        ("run-task-010", "2026-03-10T13:05:00Z", "waiting on after-writing-plans"),
        ("run-task-020", "2026-03-10T13:06:00Z", "artifact write failed"),
    ] {
        let run_dir = create_task_run(
            &tasks_root,
            &PersistedTaskRun {
                run_id: run_id.to_owned(),
                runtime: "codex".to_owned(),
                objective: format!("Board for {run_id}"),
                current_phase: "execution".to_owned(),
                overall_state: OrchestratorState::WaitingForInput,
                blocking_reason: Some("operator input required".to_owned()),
                workspace_root: "workspace".to_owned(),
                workspace_policy: "isolated_by_default".to_owned(),
                default_isolation: true,
                created_at: "2026-03-10T13:00:00Z".to_owned(),
                updated_at: updated_at.to_owned(),
            },
        )
        .expect("task run should persist");
        write_agent(
            &run_dir,
            &PersistedAgent {
                agent_id: "agent-plan".to_owned(),
                run_id: run_id.to_owned(),
                parent_agent_id: None,
                role: "writing-plans".to_owned(),
                current_phase: "execution".to_owned(),
                current_state: OrchestratorState::WaitingForInput,
                runtime: "codex".to_owned(),
                workspace_path: format!("workspace/{run_id}"),
                pid: Some(6301),
                related_artifact_ids: vec![],
                stdout_log: format!("logs/{run_id}-stdout.log"),
                stderr_log: format!("logs/{run_id}-stderr.log"),
                created_at: "2026-03-10T13:00:00Z".to_owned(),
                updated_at: updated_at.to_owned(),
            },
        )
        .expect("agent should persist");
        append_task_event(
            &run_dir,
            &PersistedTaskEvent {
                event_id: format!("event-{run_id}"),
                run_id: run_id.to_owned(),
                agent_id: Some("agent-plan".to_owned()),
                event_type: AgentEventType::Fail,
                payload_summary: detail.to_owned(),
                timestamp: updated_at.to_owned(),
            },
        )
        .expect("event should persist");
    }

    let output = run_command(&["swarm", "board"], &state_root);
    assert!(output.status.success(), "board should succeed");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("run-task-020 agent-plan writing-plans artifact write failed"));
    assert!(stdout.contains("run-task-010 agent-plan writing-plans waiting on after-writing-plans"));

    fs::remove_dir_all(state_root).expect("temp root should be removable");
}

#[test]
fn board_command_falls_back_to_legacy_placeholder_when_no_task_runs_exist() {
    let state_root = temp_root();
    create_run(
        &state_root,
        &PersistedRun {
            run_id: "run-legacy-001".to_owned(),
            runtime: "codex".to_owned(),
            objective: "legacy flow".to_owned(),
            shard_count: 1,
        },
        &[PersistedShard {
            shard_id: "01".to_owned(),
            runtime: "codex".to_owned(),
            pid: Some(4101),
            state: "running".to_owned(),
            workspace: "workspace-01".to_owned(),
        }],
    )
    .expect("legacy run should persist");

    let output = run_command(&["swarm", "board"], &state_root);
    assert!(output.status.success(), "board should succeed");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert_eq!(
        stdout,
        "\
Board
  active runs: 1
  blocked shards: 0
  merge queue: unavailable

Active Runs
  run-legacy-001 active 1 shards objective: legacy flow

Blocked Shards
  none

Next
  use `patchlane swarm status` for a single run or `patchlane swarm web` for a broader overview
"
    );

    fs::remove_dir_all(state_root).expect("temp root should be removable");
}

#[test]
fn board_command_prefers_task_runs_when_task_and_legacy_runs_coexist() {
    let state_root = temp_root();
    let run_dir = create_task_run(
        &state_root.join("tasks"),
        &PersistedTaskRun {
            run_id: "run-task-004".to_owned(),
            runtime: "codex".to_owned(),
            objective: "Prefer task board".to_owned(),
            current_phase: "execution".to_owned(),
            overall_state: OrchestratorState::WaitingForInput,
            blocking_reason: Some("waiting on operator".to_owned()),
            workspace_root: "workspace".to_owned(),
            workspace_policy: "isolated_by_default".to_owned(),
            default_isolation: true,
            created_at: "2026-03-10T13:00:00Z".to_owned(),
            updated_at: "2026-03-10T13:05:00Z".to_owned(),
        },
    )
    .expect("task run should persist");
    write_agent(
        &run_dir,
        &PersistedAgent {
            agent_id: "agent-exec".to_owned(),
            run_id: "run-task-004".to_owned(),
            parent_agent_id: None,
            role: "subagent-driven-development".to_owned(),
            current_phase: "execution".to_owned(),
            current_state: OrchestratorState::WaitingForInput,
            runtime: "codex".to_owned(),
            workspace_path: "workspace/exec".to_owned(),
            pid: Some(6101),
            related_artifact_ids: vec![],
            stdout_log: "logs/agent-exec-stdout.log".to_owned(),
            stderr_log: "logs/agent-exec-stderr.log".to_owned(),
            created_at: "2026-03-10T13:00:00Z".to_owned(),
            updated_at: "2026-03-10T13:05:00Z".to_owned(),
        },
    )
    .expect("agent should persist");
    append_task_event(
        &run_dir,
        &PersistedTaskEvent {
            event_id: "event-input".to_owned(),
            run_id: "run-task-004".to_owned(),
            agent_id: Some("agent-exec".to_owned()),
            event_type: AgentEventType::WaitingInput,
            payload_summary: "Need operator context".to_owned(),
            timestamp: "2026-03-10T13:05:00Z".to_owned(),
        },
    )
    .expect("task event should persist");

    create_run(
        &state_root,
        &PersistedRun {
            run_id: "run-legacy-004".to_owned(),
            runtime: "claude".to_owned(),
            objective: "Legacy board".to_owned(),
            shard_count: 1,
        },
        &[PersistedShard {
            shard_id: "01".to_owned(),
            runtime: "claude".to_owned(),
            pid: Some(7101),
            state: "running".to_owned(),
            workspace: "workspace-01".to_owned(),
        }],
    )
    .expect("legacy run should persist");

    let output = run_command(&["swarm", "board"], &state_root);
    assert!(output.status.success(), "board should succeed");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let expected = "\
Board
  active runs: 1
  blocked agents: 1

Active Runs
  run-task-004 waiting_for_input 1 agents objective: Prefer task board

Blocked Agents
  run-task-004 agent-exec subagent-driven-development Need operator context

Next
  use `patchlane swarm status` for the latest run or `patchlane swarm watch` for event flow";
    assert_eq!(stdout.trim(), expected);

    fs::remove_dir_all(state_root).expect("temp root should be removable");
}
