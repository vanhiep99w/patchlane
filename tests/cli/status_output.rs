use patchlane::orchestration::model::{
    AgentEventType, ArtifactType, CheckpointStatus, OrchestratorState, PersistedAgent,
    PersistedArtifact, PersistedCheckpoint, PersistedTaskEvent, PersistedTaskRun,
};
use patchlane::orchestration::store::{
    append_task_event, create_task_run, write_agent, write_artifact, write_checkpoint,
};
use patchlane::store::run_store::{append_event, create_run, PersistedEvent, PersistedRun, PersistedShard};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_root() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("patchlane-task-status-{unique}"));
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
fn status_output_reads_the_latest_persisted_task_snapshot() {
    let state_root = temp_root();
    let tasks_root = state_root.join("tasks");
    let run_dir = create_task_run(
        &tasks_root,
        &PersistedTaskRun {
            run_id: "run-task-002".to_owned(),
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
        &run_dir,
        &PersistedAgent {
            agent_id: "agent-brainstorm".to_owned(),
            run_id: "run-task-002".to_owned(),
            parent_agent_id: None,
            role: "brainstorming".to_owned(),
            current_phase: "done".to_owned(),
            current_state: OrchestratorState::Done,
            runtime: "codex".to_owned(),
            workspace_path: "workspace/brainstorm".to_owned(),
            pid: Some(4101),
            related_artifact_ids: vec!["artifact-spec".to_owned()],
            stdout_log: "logs/agent-brainstorm-stdout.log".to_owned(),
            stderr_log: "logs/agent-brainstorm-stderr.log".to_owned(),
            created_at: "2026-03-10T10:00:00Z".to_owned(),
            updated_at: "2026-03-10T10:04:00Z".to_owned(),
        },
    )
    .expect("brainstorm agent should persist");
    write_agent(
        &run_dir,
        &PersistedAgent {
            agent_id: "agent-plan".to_owned(),
            run_id: "run-task-002".to_owned(),
            parent_agent_id: None,
            role: "writing-plans".to_owned(),
            current_phase: "writing-plans".to_owned(),
            current_state: OrchestratorState::WaitingForApproval,
            runtime: "codex".to_owned(),
            workspace_path: "workspace/plan".to_owned(),
            pid: None,
            related_artifact_ids: vec!["artifact-plan".to_owned()],
            stdout_log: "logs/agent-plan-stdout.log".to_owned(),
            stderr_log: "logs/agent-plan-stderr.log".to_owned(),
            created_at: "2026-03-10T10:04:00Z".to_owned(),
            updated_at: "2026-03-10T10:06:00Z".to_owned(),
        },
    )
    .expect("plan agent should persist");
    write_checkpoint(
        &run_dir,
        &PersistedCheckpoint {
            checkpoint_id: "checkpoint-plan".to_owned(),
            run_id: "run-task-002".to_owned(),
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
    write_artifact(
        &run_dir,
        &PersistedArtifact {
            artifact_id: "artifact-plan".to_owned(),
            run_id: "run-task-002".to_owned(),
            producing_agent_id: "agent-plan".to_owned(),
            artifact_type: ArtifactType::Plan,
            path: "docs/superpowers/plans/plan.md".to_owned(),
            created_at: "2026-03-10T10:05:00Z".to_owned(),
        },
    )
    .expect("artifact should persist");
    append_task_event(
        &run_dir,
        &PersistedTaskEvent {
            event_id: "event-plan".to_owned(),
            run_id: "run-task-002".to_owned(),
            agent_id: Some("agent-plan".to_owned()),
            event_type: AgentEventType::WaitingApproval,
            payload_summary: "checkpoint-plan|Approve? [y/n]".to_owned(),
            timestamp: "2026-03-10T10:06:00Z".to_owned(),
        },
    )
    .expect("event should persist");

    let output = run_command(&["swarm", "status"], &state_root);
    assert!(output.status.success(), "status should succeed");
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let expected = "\
Run
  run-task-002 (waiting_for_approval)
  runtime: codex
  phase: writing-plans
  objective: Ship orchestration flow

Agents
  id                 role             phase                state                runtime  detail
  agent-brainstorm   brainstorming    done                 done                 codex   none
  agent-plan         writing-plans    writing-plans        waiting_for_approval codex   waiting on after-writing-plans

Blockers
  - run checkpoint pending
  - agent-plan waiting on after-writing-plans

Latest Event
  2026-03-10T10:06:00Z [agent-plan] waiting-approval checkpoint-plan|Approve? [y/n]

Next
  patchlane swarm watch";
    assert_eq!(stdout.trim(), expected);

    fs::remove_dir_all(state_root).expect("temp root should be removable");
}

#[test]
fn status_output_surfaces_run_level_blocking_reason_without_agent_blockers() {
    let state_root = temp_root();
    let run_dir = create_task_run(
        &state_root.join("tasks"),
        &PersistedTaskRun {
            run_id: "run-task-003".to_owned(),
            runtime: "codex".to_owned(),
            objective: "Wait on operator context".to_owned(),
            current_phase: "brainstorming".to_owned(),
            overall_state: OrchestratorState::WaitingForInput,
            blocking_reason: Some("waiting for operator context".to_owned()),
            workspace_root: "workspace".to_owned(),
            workspace_policy: "isolated_by_default".to_owned(),
            default_isolation: true,
            created_at: "2026-03-10T11:00:00Z".to_owned(),
            updated_at: "2026-03-10T11:03:00Z".to_owned(),
        },
    )
    .expect("task run should persist");
    write_agent(
        &run_dir,
        &PersistedAgent {
            agent_id: "agent-brainstorm".to_owned(),
            run_id: "run-task-003".to_owned(),
            parent_agent_id: None,
            role: "brainstorming".to_owned(),
            current_phase: "brainstorming".to_owned(),
            current_state: OrchestratorState::Running,
            runtime: "codex".to_owned(),
            workspace_path: "workspace/brainstorm".to_owned(),
            pid: Some(5010),
            related_artifact_ids: vec![],
            stdout_log: "logs/agent-brainstorm-stdout.log".to_owned(),
            stderr_log: "logs/agent-brainstorm-stderr.log".to_owned(),
            created_at: "2026-03-10T11:00:00Z".to_owned(),
            updated_at: "2026-03-10T11:03:00Z".to_owned(),
        },
    )
    .expect("agent should persist");
    append_task_event(
        &run_dir,
        &PersistedTaskEvent {
            event_id: "event-wait".to_owned(),
            run_id: "run-task-003".to_owned(),
            agent_id: Some("agent-brainstorm".to_owned()),
            event_type: AgentEventType::WaitingInput,
            payload_summary: "Need objective clarification".to_owned(),
            timestamp: "2026-03-10T11:03:00Z".to_owned(),
        },
    )
    .expect("event should persist");

    let output = run_command(&["swarm", "status"], &state_root);
    assert!(output.status.success(), "status should succeed");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let expected = "\
Run
  run-task-003 (waiting_for_input)
  runtime: codex
  phase: brainstorming
  objective: Wait on operator context

Agents
  id                 role             phase                state                runtime  detail
  agent-brainstorm   brainstorming    brainstorming        running              codex   Need objective clarification

Blockers
  - run waiting for operator context

Latest Event
  2026-03-10T11:03:00Z [agent-brainstorm] waiting-input Need objective clarification

Next
  patchlane swarm watch";
    assert_eq!(stdout.trim(), expected);

    fs::remove_dir_all(state_root).expect("temp root should be removable");
}

#[test]
fn status_output_prefers_latest_task_snapshot_by_updated_at_and_shows_failed_agent_detail() {
    let state_root = temp_root();
    let tasks_root = state_root.join("tasks");

    let older_run_dir = create_task_run(
        &tasks_root,
        &PersistedTaskRun {
            run_id: "run-task-zeta".to_owned(),
            runtime: "claude".to_owned(),
            objective: "Older lexicographically-late task".to_owned(),
            current_phase: "execution".to_owned(),
            overall_state: OrchestratorState::Running,
            blocking_reason: None,
            workspace_root: "workspace".to_owned(),
            workspace_policy: "isolated_by_default".to_owned(),
            default_isolation: true,
            created_at: "2026-03-10T10:00:00Z".to_owned(),
            updated_at: "2026-03-10T10:01:00Z".to_owned(),
        },
    )
    .expect("older task run should persist");
    write_agent(
        &older_run_dir,
        &PersistedAgent {
            agent_id: "agent-old".to_owned(),
            run_id: "run-task-zeta".to_owned(),
            parent_agent_id: None,
            role: "brainstorming".to_owned(),
            current_phase: "brainstorming".to_owned(),
            current_state: OrchestratorState::Running,
            runtime: "claude".to_owned(),
            workspace_path: "workspace/old".to_owned(),
            pid: Some(6001),
            related_artifact_ids: vec![],
            stdout_log: "logs/agent-old-stdout.log".to_owned(),
            stderr_log: "logs/agent-old-stderr.log".to_owned(),
            created_at: "2026-03-10T10:00:00Z".to_owned(),
            updated_at: "2026-03-10T10:01:00Z".to_owned(),
        },
    )
    .expect("older agent should persist");
    append_task_event(
        &older_run_dir,
        &PersistedTaskEvent {
            event_id: "event-old".to_owned(),
            run_id: "run-task-zeta".to_owned(),
            agent_id: Some("agent-old".to_owned()),
            event_type: AgentEventType::Phase,
            payload_summary: "brainstorming".to_owned(),
            timestamp: "2026-03-10T10:01:00Z".to_owned(),
        },
    )
    .expect("older event should persist");

    let newer_run_dir = create_task_run(
        &tasks_root,
        &PersistedTaskRun {
            run_id: "run-task-alpha".to_owned(),
            runtime: "codex".to_owned(),
            objective: "Newest failed task".to_owned(),
            current_phase: "execution".to_owned(),
            overall_state: OrchestratorState::Failed,
            blocking_reason: Some("artifact persistence failed".to_owned()),
            workspace_root: "workspace".to_owned(),
            workspace_policy: "isolated_by_default".to_owned(),
            default_isolation: true,
            created_at: "2026-03-10T10:02:00Z".to_owned(),
            updated_at: "2026-03-10T10:05:00Z".to_owned(),
        },
    )
    .expect("newer task run should persist");
    write_agent(
        &newer_run_dir,
        &PersistedAgent {
            agent_id: "agent-exec".to_owned(),
            run_id: "run-task-alpha".to_owned(),
            parent_agent_id: None,
            role: "subagent-driven-development".to_owned(),
            current_phase: "execution".to_owned(),
            current_state: OrchestratorState::Failed,
            runtime: "codex".to_owned(),
            workspace_path: "workspace/exec".to_owned(),
            pid: Some(6002),
            related_artifact_ids: vec![],
            stdout_log: "logs/agent-exec-stdout.log".to_owned(),
            stderr_log: "logs/agent-exec-stderr.log".to_owned(),
            created_at: "2026-03-10T10:02:00Z".to_owned(),
            updated_at: "2026-03-10T10:05:00Z".to_owned(),
        },
    )
    .expect("newer agent should persist");
    append_task_event(
        &newer_run_dir,
        &PersistedTaskEvent {
            event_id: "event-fail".to_owned(),
            run_id: "run-task-alpha".to_owned(),
            agent_id: Some("agent-exec".to_owned()),
            event_type: AgentEventType::Fail,
            payload_summary: "artifact write failed".to_owned(),
            timestamp: "2026-03-10T10:05:00Z".to_owned(),
        },
    )
    .expect("newer event should persist");

    let output = run_command(&["swarm", "status"], &state_root);
    assert!(output.status.success(), "status should succeed");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let expected = "\
Run
  run-task-alpha (failed)
  runtime: codex
  phase: execution
  objective: Newest failed task

Agents
  id                 role             phase                state                runtime  detail
  agent-exec         subagent-driven-development execution            failed               codex   artifact write failed

Blockers
  - run artifact persistence failed
  - agent-exec artifact write failed

Latest Event
  2026-03-10T10:05:00Z [agent-exec] fail artifact write failed

Next
  patchlane swarm watch";
    assert_eq!(stdout.trim(), expected);

    fs::remove_dir_all(state_root).expect("temp root should be removable");
}

#[test]
fn status_output_skips_malformed_task_runs_when_selecting_latest_snapshot() {
    let state_root = temp_root();
    let tasks_root = state_root.join("tasks");

    let malformed_dir = tasks_root.join("bad-run");
    fs::create_dir_all(&malformed_dir).expect("malformed task dir should be creatable");
    fs::write(malformed_dir.join("run.json"), "{not valid json").expect("malformed run should persist");

    let good_run_dir = create_task_run(
        &tasks_root,
        &PersistedTaskRun {
            run_id: "run-task-good".to_owned(),
            runtime: "codex".to_owned(),
            objective: "Ignore malformed task runs".to_owned(),
            current_phase: "review".to_owned(),
            overall_state: OrchestratorState::InReview,
            blocking_reason: None,
            workspace_root: "workspace".to_owned(),
            workspace_policy: "isolated_by_default".to_owned(),
            default_isolation: true,
            created_at: "2026-03-10T11:00:00Z".to_owned(),
            updated_at: "2026-03-10T11:05:00Z".to_owned(),
        },
    )
    .expect("good task run should persist");
    write_agent(
        &good_run_dir,
        &PersistedAgent {
            agent_id: "agent-review".to_owned(),
            run_id: "run-task-good".to_owned(),
            parent_agent_id: None,
            role: "requesting-code-review".to_owned(),
            current_phase: "review".to_owned(),
            current_state: OrchestratorState::InReview,
            runtime: "codex".to_owned(),
            workspace_path: "workspace/review".to_owned(),
            pid: Some(6201),
            related_artifact_ids: vec![],
            stdout_log: "logs/agent-review-stdout.log".to_owned(),
            stderr_log: "logs/agent-review-stderr.log".to_owned(),
            created_at: "2026-03-10T11:00:00Z".to_owned(),
            updated_at: "2026-03-10T11:05:00Z".to_owned(),
        },
    )
    .expect("good agent should persist");
    append_task_event(
        &good_run_dir,
        &PersistedTaskEvent {
            event_id: "event-review".to_owned(),
            run_id: "run-task-good".to_owned(),
            agent_id: Some("agent-review".to_owned()),
            event_type: AgentEventType::ReviewStart,
            payload_summary: "quality review".to_owned(),
            timestamp: "2026-03-10T11:05:00Z".to_owned(),
        },
    )
    .expect("good event should persist");

    let output = run_command(&["swarm", "status"], &state_root);
    assert!(output.status.success(), "status should succeed");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("run-task-good (in_review)"));
    assert!(stdout.contains("[agent-review] review-start quality review"));

    fs::remove_dir_all(state_root).expect("temp root should be removable");
}

#[test]
fn status_output_falls_back_to_legacy_swarm_runs_when_no_task_runs_exist() {
    let state_root = temp_root();
    let run_dir = create_run(
        &state_root,
        &PersistedRun {
            run_id: "run-legacy-002".to_owned(),
            runtime: "codex".to_owned(),
            objective: "legacy fallback".to_owned(),
            shard_count: 2,
        },
        &[
            PersistedShard {
                shard_id: "01".to_owned(),
                runtime: "codex".to_owned(),
                pid: Some(4242),
                state: "launched".to_owned(),
                workspace: "run-legacy-002/workspace-01".to_owned(),
            },
            PersistedShard {
                shard_id: "02".to_owned(),
                runtime: "codex".to_owned(),
                pid: None,
                state: "failed".to_owned(),
                workspace: "run-legacy-002/workspace-02".to_owned(),
            },
        ],
    )
    .expect("legacy run should persist");
    append_event(
        &run_dir,
        &PersistedEvent {
            timestamp: "2026-03-10T12:05:00Z".to_owned(),
            shard_id: Some("02".to_owned()),
            message: "spawn failure for shard 02 using codex: missing binary".to_owned(),
        },
    )
    .expect("legacy event should persist");

    let output = run_command(&["swarm", "status"], &state_root);
    assert!(output.status.success(), "status should succeed");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let expected = "\
Run
  run-legacy-002 (degraded)
  runtime: codex
  objective: legacy fallback

Shards
  shard  runtime  pid    state      workspace  detail
  01     codex   4242   launched   run-legacy-002/workspace-01  none
  02     codex   -      failed     run-legacy-002/workspace-02  spawn failure for shard 02 using codex: missing binary

Blockers
  - shard 02 spawn failure for shard 02 using codex: missing binary

Latest Event
  2026-03-10T12:05:00Z spawn failure for shard 02 using codex: missing binary

Next
  patchlane swarm retry <shard-id>";
    assert_eq!(stdout.trim(), expected);

    fs::remove_dir_all(state_root).expect("temp root should be removable");
}

#[test]
fn status_output_prefers_task_runs_when_task_and_legacy_runs_coexist() {
    let state_root = temp_root();
    let run_dir = create_task_run(
        &state_root.join("tasks"),
        &PersistedTaskRun {
            run_id: "run-task-004".to_owned(),
            runtime: "codex".to_owned(),
            objective: "Prefer task status".to_owned(),
            current_phase: "review".to_owned(),
            overall_state: OrchestratorState::InReview,
            blocking_reason: None,
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
            agent_id: "agent-review".to_owned(),
            run_id: "run-task-004".to_owned(),
            parent_agent_id: None,
            role: "requesting-code-review".to_owned(),
            current_phase: "review".to_owned(),
            current_state: OrchestratorState::InReview,
            runtime: "codex".to_owned(),
            workspace_path: "workspace/review".to_owned(),
            pid: Some(6010),
            related_artifact_ids: vec![],
            stdout_log: "logs/agent-review-stdout.log".to_owned(),
            stderr_log: "logs/agent-review-stderr.log".to_owned(),
            created_at: "2026-03-10T12:00:00Z".to_owned(),
            updated_at: "2026-03-10T12:05:00Z".to_owned(),
        },
    )
    .expect("agent should persist");
    append_task_event(
        &run_dir,
        &PersistedTaskEvent {
            event_id: "event-review".to_owned(),
            run_id: "run-task-004".to_owned(),
            agent_id: Some("agent-review".to_owned()),
            event_type: AgentEventType::ReviewStart,
            payload_summary: "quality review".to_owned(),
            timestamp: "2026-03-10T12:05:00Z".to_owned(),
        },
    )
    .expect("task event should persist");

    let legacy_run_dir = create_run(
        &state_root,
        &PersistedRun {
            run_id: "run-legacy-004".to_owned(),
            runtime: "claude".to_owned(),
            objective: "Legacy status".to_owned(),
            shard_count: 1,
        },
        &[PersistedShard {
            shard_id: "01".to_owned(),
            runtime: "claude".to_owned(),
            pid: Some(4242),
            state: "failed".to_owned(),
            workspace: "run-legacy-004/workspace-01".to_owned(),
        }],
    )
    .expect("legacy run should persist");
    append_event(
        &legacy_run_dir,
        &PersistedEvent {
            timestamp: "2026-03-10T12:06:00Z".to_owned(),
            shard_id: Some("01".to_owned()),
            message: "legacy failure".to_owned(),
        },
    )
    .expect("legacy event should persist");

    let output = run_command(&["swarm", "status"], &state_root);
    assert!(output.status.success(), "status should succeed");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let expected = "\
Run
  run-task-004 (in_review)
  runtime: codex
  phase: review
  objective: Prefer task status

Agents
  id                 role             phase                state                runtime  detail
  agent-review       requesting-code-review review               in_review            codex   quality review

Blockers
  none

Latest Event
  2026-03-10T12:05:00Z [agent-review] review-start quality review

Next
  patchlane swarm watch";
    assert_eq!(stdout.trim(), expected);

    fs::remove_dir_all(state_root).expect("temp root should be removable");
}
