use patchlane::orchestration::model::{
    AgentEventType, OrchestratorState, PersistedAgent, PersistedTaskEvent, PersistedTaskRun,
};
use patchlane::orchestration::store::{append_task_event, create_task_run, write_agent};
use patchlane::store::run_store::{
    append_event, create_run, PersistedEvent, PersistedRun, PersistedShard,
};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_root() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("patchlane-watch-{unique}"));
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
fn watch_output_reads_orchestration_events_for_the_latest_task_run() {
    let state_root = temp_root();
    let run_dir = create_task_run(
        &state_root.join("tasks"),
        &PersistedTaskRun {
            run_id: "run-task-002".to_owned(),
            runtime: "codex".to_owned(),
            objective: "stream orchestration events".to_owned(),
            current_phase: "writing-plans".to_owned(),
            overall_state: OrchestratorState::Running,
            blocking_reason: None,
            workspace_root: "workspace".to_owned(),
            workspace_policy: "isolated_by_default".to_owned(),
            default_isolation: true,
            created_at: "2026-03-10T10:00:00Z".to_owned(),
            updated_at: "2026-03-10T10:04:00Z".to_owned(),
        },
    )
    .expect("task run should persist");

    write_agent(
        &run_dir,
        &PersistedAgent {
            agent_id: "agent-plan".to_owned(),
            run_id: "run-task-002".to_owned(),
            parent_agent_id: None,
            role: "writing-plans".to_owned(),
            current_phase: "writing-plans".to_owned(),
            current_state: OrchestratorState::Running,
            runtime: "codex".to_owned(),
            workspace_path: "workspace/plan".to_owned(),
            pid: Some(4102),
            related_artifact_ids: vec![],
            stdout_log: "logs/agent-plan-stdout.log".to_owned(),
            stderr_log: "logs/agent-plan-stderr.log".to_owned(),
            created_at: "2026-03-10T10:00:00Z".to_owned(),
            updated_at: "2026-03-10T10:04:00Z".to_owned(),
        },
    )
    .expect("agent should persist");

    for event in [
        PersistedTaskEvent {
            event_id: "event-001".to_owned(),
            run_id: "run-task-002".to_owned(),
            agent_id: Some("agent-plan".to_owned()),
            event_type: AgentEventType::Start,
            payload_summary: "writing-plans".to_owned(),
            timestamp: "2026-03-10T10:00:00Z".to_owned(),
        },
        PersistedTaskEvent {
            event_id: "event-002".to_owned(),
            run_id: "run-task-002".to_owned(),
            agent_id: Some("agent-plan".to_owned()),
            event_type: AgentEventType::Artifact,
            payload_summary: "plan.md".to_owned(),
            timestamp: "2026-03-10T10:03:00Z".to_owned(),
        },
        PersistedTaskEvent {
            event_id: "event-003".to_owned(),
            run_id: "run-task-002".to_owned(),
            agent_id: Some("agent-plan".to_owned()),
            event_type: AgentEventType::Done,
            payload_summary: "writing-plans complete".to_owned(),
            timestamp: "2026-03-10T10:04:00Z".to_owned(),
        },
    ] {
        append_task_event(&run_dir, &event).expect("event should persist");
    }

    let output = run_command(&["swarm", "watch"], &state_root);

    assert!(
        output.status.success(),
        "expected swarm watch to succeed, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    let expected = "\
2026-03-10T10:00:00Z [agent-plan] start writing-plans
2026-03-10T10:03:00Z [agent-plan] artifact plan.md
2026-03-10T10:04:00Z [agent-plan] done writing-plans complete";
    assert_eq!(stdout.trim(), expected);

    fs::remove_dir_all(state_root).expect("temp root should be removable");
}

#[test]
fn watch_output_falls_back_to_legacy_swarm_events_when_no_task_runs_exist() {
    let state_root = temp_root();
    let run_dir = create_run(
        &state_root,
        &PersistedRun {
            run_id: "run-001".to_owned(),
            runtime: "codex".to_owned(),
            objective: "stream real worker events".to_owned(),
            shard_count: 2,
        },
        &[
            PersistedShard {
                shard_id: "01".to_owned(),
                runtime: "codex".to_owned(),
                pid: Some(4101),
                state: "launched".to_owned(),
                workspace: "workspace-01".to_owned(),
            },
            PersistedShard {
                shard_id: "02".to_owned(),
                runtime: "codex".to_owned(),
                pid: Some(4102),
                state: "running".to_owned(),
                workspace: "workspace-02".to_owned(),
            },
        ],
    )
    .expect("legacy run should persist");

    for event in [
        PersistedEvent {
            timestamp: "2026-03-10T10:00:00Z".to_owned(),
            shard_id: Some("01".to_owned()),
            message: "launched local codex worker for shard 01".to_owned(),
        },
        PersistedEvent {
            timestamp: "2026-03-10T10:04:00Z".to_owned(),
            shard_id: Some("02".to_owned()),
            message: "assistant: transcript summary for shard 02".to_owned(),
        },
    ] {
        append_event(&run_dir, &event).expect("event should persist");
    }

    let output = run_command(&["swarm", "watch"], &state_root);
    assert!(output.status.success(), "watch should succeed");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert_eq!(
        stdout,
        "2026-03-10T10:00:00Z launched local codex worker for shard 01\n"
    );

    fs::remove_dir_all(state_root).expect("temp root should be removable");
}

#[test]
fn watch_output_prefers_task_runs_when_task_and_legacy_runs_coexist() {
    let state_root = temp_root();

    let task_run_dir = create_task_run(
        &state_root.join("tasks"),
        &PersistedTaskRun {
            run_id: "run-task-003".to_owned(),
            runtime: "codex".to_owned(),
            objective: "prefer task state".to_owned(),
            current_phase: "review".to_owned(),
            overall_state: OrchestratorState::InReview,
            blocking_reason: None,
            workspace_root: "workspace".to_owned(),
            workspace_policy: "isolated_by_default".to_owned(),
            default_isolation: true,
            created_at: "2026-03-10T10:10:00Z".to_owned(),
            updated_at: "2026-03-10T10:11:00Z".to_owned(),
        },
    )
    .expect("task run should persist");
    append_task_event(
        &task_run_dir,
        &PersistedTaskEvent {
            event_id: "event-task".to_owned(),
            run_id: "run-task-003".to_owned(),
            agent_id: Some("agent-review".to_owned()),
            event_type: AgentEventType::ReviewStart,
            payload_summary: "quality review".to_owned(),
            timestamp: "2026-03-10T10:11:00Z".to_owned(),
        },
    )
    .expect("task event should persist");

    let legacy_run_dir = create_run(
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
            pid: Some(7001),
            state: "running".to_owned(),
            workspace: "workspace-01".to_owned(),
        }],
    )
    .expect("legacy run should persist");
    append_event(
        &legacy_run_dir,
        &PersistedEvent {
            timestamp: "2026-03-10T10:12:00Z".to_owned(),
            shard_id: Some("01".to_owned()),
            message: "legacy event".to_owned(),
        },
    )
    .expect("legacy event should persist");

    let output = run_command(&["swarm", "watch"], &state_root);
    assert!(output.status.success(), "watch should succeed");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert_eq!(
        stdout.trim(),
        "2026-03-10T10:11:00Z [agent-review] review-start quality review"
    );

    fs::remove_dir_all(state_root).expect("temp root should be removable");
}

#[test]
fn watch_output_prefers_latest_task_run_by_updated_at() {
    let state_root = temp_root();

    let older_run_dir = create_task_run(
        &state_root.join("tasks"),
        &PersistedTaskRun {
            run_id: "run-task-zeta".to_owned(),
            runtime: "claude".to_owned(),
            objective: "Older task".to_owned(),
            current_phase: "brainstorming".to_owned(),
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
    append_task_event(
        &older_run_dir,
        &PersistedTaskEvent {
            event_id: "event-old".to_owned(),
            run_id: "run-task-zeta".to_owned(),
            agent_id: Some("agent-old".to_owned()),
            event_type: AgentEventType::Start,
            payload_summary: "brainstorming".to_owned(),
            timestamp: "2026-03-10T10:01:00Z".to_owned(),
        },
    )
    .expect("older task event should persist");

    let newer_run_dir = create_task_run(
        &state_root.join("tasks"),
        &PersistedTaskRun {
            run_id: "run-task-alpha".to_owned(),
            runtime: "codex".to_owned(),
            objective: "Newest task".to_owned(),
            current_phase: "review".to_owned(),
            overall_state: OrchestratorState::InReview,
            blocking_reason: None,
            workspace_root: "workspace".to_owned(),
            workspace_policy: "isolated_by_default".to_owned(),
            default_isolation: true,
            created_at: "2026-03-10T10:02:00Z".to_owned(),
            updated_at: "2026-03-10T10:06:00Z".to_owned(),
        },
    )
    .expect("newer task run should persist");
    append_task_event(
        &newer_run_dir,
        &PersistedTaskEvent {
            event_id: "event-new".to_owned(),
            run_id: "run-task-alpha".to_owned(),
            agent_id: Some("agent-new".to_owned()),
            event_type: AgentEventType::ReviewStart,
            payload_summary: "fresh review".to_owned(),
            timestamp: "2026-03-10T10:06:00Z".to_owned(),
        },
    )
    .expect("newer task event should persist");

    let output = run_command(&["swarm", "watch"], &state_root);
    assert!(output.status.success(), "watch should succeed");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert_eq!(
        stdout.trim(),
        "2026-03-10T10:06:00Z [agent-new] review-start fresh review"
    );

    fs::remove_dir_all(state_root).expect("temp root should be removable");
}

#[test]
fn watch_output_skips_malformed_task_runs() {
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
            objective: "Ignore malformed watch runs".to_owned(),
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

    let output = run_command(&["swarm", "watch"], &state_root);
    assert!(output.status.success(), "watch should succeed");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert_eq!(
        stdout.trim(),
        "2026-03-10T11:05:00Z [agent-review] review-start quality review"
    );

    fs::remove_dir_all(state_root).expect("temp root should be removable");
}

#[test]
fn watch_output_distinguishes_blocked_and_failed_agents_with_concrete_details() {
    let state_root = temp_root();
    let run_dir = create_task_run(
        &state_root.join("tasks"),
        &PersistedTaskRun {
            run_id: "run-task-004".to_owned(),
            runtime: "codex".to_owned(),
            objective: "show blocker detail".to_owned(),
            current_phase: "execution".to_owned(),
            overall_state: OrchestratorState::Failed,
            blocking_reason: Some("run blocked".to_owned()),
            workspace_root: "workspace".to_owned(),
            workspace_policy: "isolated_by_default".to_owned(),
            default_isolation: true,
            created_at: "2026-03-10T11:00:00Z".to_owned(),
            updated_at: "2026-03-10T11:05:00Z".to_owned(),
        },
    )
    .expect("task run should persist");

    for agent in [
        PersistedAgent {
            agent_id: "agent-plan".to_owned(),
            run_id: "run-task-004".to_owned(),
            parent_agent_id: None,
            role: "writing-plans".to_owned(),
            current_phase: "writing-plans".to_owned(),
            current_state: OrchestratorState::WaitingForInput,
            runtime: "codex".to_owned(),
            workspace_path: "workspace/plan".to_owned(),
            pid: Some(7001),
            related_artifact_ids: vec![],
            stdout_log: "logs/agent-plan-stdout.log".to_owned(),
            stderr_log: "logs/agent-plan-stderr.log".to_owned(),
            created_at: "2026-03-10T11:00:00Z".to_owned(),
            updated_at: "2026-03-10T11:04:00Z".to_owned(),
        },
        PersistedAgent {
            agent_id: "agent-exec".to_owned(),
            run_id: "run-task-004".to_owned(),
            parent_agent_id: None,
            role: "subagent-driven-development".to_owned(),
            current_phase: "execution".to_owned(),
            current_state: OrchestratorState::Failed,
            runtime: "codex".to_owned(),
            workspace_path: "workspace/exec".to_owned(),
            pid: Some(7002),
            related_artifact_ids: vec![],
            stdout_log: "logs/agent-exec-stdout.log".to_owned(),
            stderr_log: "logs/agent-exec-stderr.log".to_owned(),
            created_at: "2026-03-10T11:01:00Z".to_owned(),
            updated_at: "2026-03-10T11:05:00Z".to_owned(),
        },
    ] {
        write_agent(&run_dir, &agent).expect("agent should persist");
    }

    for event in [
        PersistedTaskEvent {
            event_id: "event-input".to_owned(),
            run_id: "run-task-004".to_owned(),
            agent_id: Some("agent-plan".to_owned()),
            event_type: AgentEventType::WaitingInput,
            payload_summary: "Need objective clarification".to_owned(),
            timestamp: "2026-03-10T11:04:00Z".to_owned(),
        },
        PersistedTaskEvent {
            event_id: "event-fail".to_owned(),
            run_id: "run-task-004".to_owned(),
            agent_id: Some("agent-exec".to_owned()),
            event_type: AgentEventType::Fail,
            payload_summary: "artifact write failed".to_owned(),
            timestamp: "2026-03-10T11:05:00Z".to_owned(),
        },
    ] {
        append_task_event(&run_dir, &event).expect("event should persist");
    }

    let output = run_command(&["swarm", "watch"], &state_root);
    assert!(output.status.success(), "watch should succeed");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let expected = "\
2026-03-10T11:04:00Z [agent-plan] waiting-input Need objective clarification
2026-03-10T11:05:00Z [agent-exec] fail artifact write failed";
    assert_eq!(stdout.trim(), expected);

    fs::remove_dir_all(state_root).expect("temp root should be removable");
}
