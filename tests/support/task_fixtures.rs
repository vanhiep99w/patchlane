use patchlane::orchestration::model::{
    AgentEventType, ArtifactType, CheckpointStatus, OrchestratorState, PersistedAgent,
    PersistedArtifact, PersistedCheckpoint, PersistedTaskEvent, PersistedTaskRun, TaskSnapshot,
};
use patchlane::orchestration::store::{
    append_task_event, create_task_run, write_agent, write_artifact, write_checkpoint,
};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn temp_state_root(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("{prefix}-{unique}"));
    fs::create_dir_all(root.join("tasks")).expect("temp root should be creatable");
    root
}

pub fn fixture_task_snapshot() -> TaskSnapshot {
    TaskSnapshot {
        run: plan_run(
            "run-task-002",
            "Ship orchestration flow",
            OrchestratorState::WaitingForApproval,
            "2026-03-10T10:06:00Z",
        ),
        agents: vec![
            agent_brainstorm(
                "run-task-002",
                OrchestratorState::Done,
                "done",
                "2026-03-10T10:04:00Z",
            ),
            agent_plan(
                "run-task-002",
                OrchestratorState::WaitingForApproval,
                "2026-03-10T10:06:00Z",
            ),
        ],
        checkpoints: vec![PersistedCheckpoint {
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
        }],
        artifacts: vec![PersistedArtifact {
            artifact_id: "artifact-plan".to_owned(),
            run_id: "run-task-002".to_owned(),
            producing_agent_id: "agent-plan".to_owned(),
            artifact_type: ArtifactType::Plan,
            path: "docs/superpowers/plans/plan.md".to_owned(),
            created_at: "2026-03-10T10:05:00Z".to_owned(),
        }],
        events: vec![
            PersistedTaskEvent {
                event_id: "event-plan-phase".to_owned(),
                run_id: "run-task-002".to_owned(),
                agent_id: Some("agent-plan".to_owned()),
                event_type: AgentEventType::Phase,
                payload_summary: "phase: writing-plans".to_owned(),
                timestamp: "2026-03-10T10:05:00Z".to_owned(),
            },
            PersistedTaskEvent {
                event_id: "event-plan-wait".to_owned(),
                run_id: "run-task-002".to_owned(),
                agent_id: Some("agent-plan".to_owned()),
                event_type: AgentEventType::WaitingApproval,
                payload_summary: "checkpoint-plan|Approve? [y/n]".to_owned(),
                timestamp: "2026-03-10T10:06:00Z".to_owned(),
            },
        ],
    }
}

pub fn fixture_task_snapshot_with_blockers() -> Vec<TaskSnapshot> {
    vec![
        fixture_task_snapshot(),
        TaskSnapshot {
            run: PersistedTaskRun {
                run_id: "run-task-003".to_owned(),
                objective: "Wait on operator context".to_owned(),
                runtime: "codex".to_owned(),
                current_phase: "subagent-driven-development".to_owned(),
                overall_state: OrchestratorState::WaitingForInput,
                blocking_reason: Some("waiting for operator context".to_owned()),
                workspace_root: "workspace".to_owned(),
                workspace_policy: "isolated_by_default".to_owned(),
                default_isolation: true,
                created_at: "2026-03-10T11:00:00Z".to_owned(),
                updated_at: "2026-03-10T11:07:00Z".to_owned(),
            },
            agents: vec![
                PersistedAgent {
                    agent_id: "agent-implement".to_owned(),
                    run_id: "run-task-003".to_owned(),
                    parent_agent_id: None,
                    role: "subagent-driven-development".to_owned(),
                    current_phase: "subagent-driven-development".to_owned(),
                    current_state: OrchestratorState::Failed,
                    runtime: "codex".to_owned(),
                    workspace_path: "workspace/implement".to_owned(),
                    pid: Some(6101),
                    related_artifact_ids: vec!["artifact-summary".to_owned()],
                    stdout_log: "logs/agent-implement-stdout.log".to_owned(),
                    stderr_log: "logs/agent-implement-stderr.log".to_owned(),
                    created_at: "2026-03-10T11:00:00Z".to_owned(),
                    updated_at: "2026-03-10T11:06:00Z".to_owned(),
                },
                PersistedAgent {
                    agent_id: "agent-review".to_owned(),
                    run_id: "run-task-003".to_owned(),
                    parent_agent_id: None,
                    role: "finishing-a-development-branch".to_owned(),
                    current_phase: "finishing-a-development-branch".to_owned(),
                    current_state: OrchestratorState::WaitingForInput,
                    runtime: "codex".to_owned(),
                    workspace_path: "workspace/review".to_owned(),
                    pid: None,
                    related_artifact_ids: vec![],
                    stdout_log: "logs/agent-review-stdout.log".to_owned(),
                    stderr_log: "logs/agent-review-stderr.log".to_owned(),
                    created_at: "2026-03-10T11:05:00Z".to_owned(),
                    updated_at: "2026-03-10T11:07:00Z".to_owned(),
                },
            ],
            checkpoints: vec![],
            artifacts: vec![PersistedArtifact {
                artifact_id: "artifact-summary".to_owned(),
                run_id: "run-task-003".to_owned(),
                producing_agent_id: "agent-implement".to_owned(),
                artifact_type: ArtifactType::Summary,
                path: "docs/superpowers/summaries/run-task-003.md".to_owned(),
                created_at: "2026-03-10T11:05:30Z".to_owned(),
            }],
            events: vec![
                PersistedTaskEvent {
                    event_id: "event-implement-fail".to_owned(),
                    run_id: "run-task-003".to_owned(),
                    agent_id: Some("agent-implement".to_owned()),
                    event_type: AgentEventType::Fail,
                    payload_summary: "artifact write failed".to_owned(),
                    timestamp: "2026-03-10T11:06:00Z".to_owned(),
                },
                PersistedTaskEvent {
                    event_id: "event-review-wait".to_owned(),
                    run_id: "run-task-003".to_owned(),
                    agent_id: Some("agent-review".to_owned()),
                    event_type: AgentEventType::WaitingInput,
                    payload_summary: "Need objective clarification".to_owned(),
                    timestamp: "2026-03-10T11:07:00Z".to_owned(),
                },
            ],
        },
    ]
}

pub fn persisted_state_root_with_two_runs() -> PathBuf {
    let state_root = temp_state_root("patchlane-tui");
    let tasks_root = state_root.join("tasks");

    persist_snapshot(&tasks_root, &fixture_task_snapshot());
    for snapshot in fixture_task_snapshot_with_blockers()
        .into_iter()
        .filter(|snapshot| snapshot.run.run_id == "run-task-003")
    {
        persist_snapshot(&tasks_root, &snapshot);
    }

    state_root
}

fn persist_snapshot(tasks_root: &PathBuf, snapshot: &TaskSnapshot) {
    let run_dir = create_task_run(tasks_root, &snapshot.run).expect("task run should persist");
    for agent in &snapshot.agents {
        write_agent(&run_dir, agent).expect("agent should persist");
    }
    for checkpoint in &snapshot.checkpoints {
        write_checkpoint(&run_dir, checkpoint).expect("checkpoint should persist");
    }
    for artifact in &snapshot.artifacts {
        write_artifact(&run_dir, artifact).expect("artifact should persist");
    }
    for event in &snapshot.events {
        append_task_event(&run_dir, event).expect("event should persist");
    }

    for agent in &snapshot.agents {
        let (stdout, stderr) = if agent.agent_id == "agent-plan" {
            (
                include_str!("../fixtures/agent-plan-stdout.log").to_owned(),
                include_str!("../fixtures/agent-plan-stderr.log").to_owned(),
            )
        } else {
            (
                format!(
                    "phase: {}\nstate: {:?}\n",
                    agent.current_phase,
                    agent.current_state
                ),
                String::new(),
            )
        };
        fs::write(run_dir.join(&agent.stdout_log), stdout).expect("stdout fixture should persist");
        fs::write(run_dir.join(&agent.stderr_log), stderr).expect("stderr fixture should persist");
    }
}

fn plan_run(
    run_id: &str,
    objective: &str,
    overall_state: OrchestratorState,
    updated_at: &str,
) -> PersistedTaskRun {
    PersistedTaskRun {
        run_id: run_id.to_owned(),
        objective: objective.to_owned(),
        runtime: "codex".to_owned(),
        current_phase: "writing-plans".to_owned(),
        overall_state,
        blocking_reason: Some("checkpoint pending".to_owned()),
        workspace_root: "workspace".to_owned(),
        workspace_policy: "isolated_by_default".to_owned(),
        default_isolation: true,
        created_at: "2026-03-10T10:00:00Z".to_owned(),
        updated_at: updated_at.to_owned(),
    }
}

fn agent_brainstorm(
    run_id: &str,
    current_state: OrchestratorState,
    current_phase: &str,
    updated_at: &str,
) -> PersistedAgent {
    PersistedAgent {
        agent_id: "agent-brainstorm".to_owned(),
        run_id: run_id.to_owned(),
        parent_agent_id: None,
        role: "brainstorming".to_owned(),
        current_phase: current_phase.to_owned(),
        current_state,
        runtime: "codex".to_owned(),
        workspace_path: "workspace/brainstorm".to_owned(),
        pid: Some(4101),
        related_artifact_ids: vec!["artifact-spec".to_owned()],
        stdout_log: "logs/agent-brainstorm-stdout.log".to_owned(),
        stderr_log: "logs/agent-brainstorm-stderr.log".to_owned(),
        created_at: "2026-03-10T10:00:00Z".to_owned(),
        updated_at: updated_at.to_owned(),
    }
}

fn agent_plan(run_id: &str, current_state: OrchestratorState, updated_at: &str) -> PersistedAgent {
    PersistedAgent {
        agent_id: "agent-plan".to_owned(),
        run_id: run_id.to_owned(),
        parent_agent_id: None,
        role: "writing-plans".to_owned(),
        current_phase: "writing-plans".to_owned(),
        current_state,
        runtime: "codex".to_owned(),
        workspace_path: "workspace/plan".to_owned(),
        pid: None,
        related_artifact_ids: vec!["artifact-plan".to_owned()],
        stdout_log: "logs/agent-plan-stdout.log".to_owned(),
        stderr_log: "logs/agent-plan-stderr.log".to_owned(),
        created_at: "2026-03-10T10:04:00Z".to_owned(),
        updated_at: updated_at.to_owned(),
    }
}
