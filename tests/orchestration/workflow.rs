use patchlane::cli::{Runtime, TaskCommand};
use patchlane::orchestration::model::{
    ArtifactType, CheckpointStatus, OrchestratorState, PersistedAgent, PersistedCheckpoint,
    PersistedTaskRun,
};
use patchlane::orchestration::recovery::recover_run_state;
use patchlane::orchestration::store::{create_task_run, load_task_snapshot, write_agent, write_checkpoint};
use patchlane::orchestration::workflow::execute_task_workflow;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_root() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("patchlane-workflow-{unique}"));
    fs::create_dir_all(&root).expect("temp root should be creatable");
    root
}

#[test]
fn task_command_follows_brainstorming_approval_plan_approval_execution_flow() {
    let root = temp_root();
    let command = TaskCommand {
        runtime: Some(Runtime::Codex),
        objective: "Ship orchestration flow".to_owned(),
    };

    let run_dir = execute_task_workflow(&root, command).expect("workflow should succeed");
    let snapshot = load_task_snapshot(&run_dir).expect("snapshot should load");

    assert!(snapshot.run.run_id.starts_with("run-"));
    assert_ne!(snapshot.run.run_id, "run-task-001");
    assert_eq!(
        run_dir.file_name().and_then(|name| name.to_str()),
        Some(snapshot.run.run_id.as_str())
    );
    assert_eq!(snapshot.run.current_phase, "finishing-a-development-branch");
    assert!(snapshot.run.overall_state == OrchestratorState::WaitingForApproval);
    assert!(
        snapshot
            .artifacts
            .iter()
            .any(|artifact| artifact.artifact_type == ArtifactType::Spec)
    );
    assert!(
        snapshot
            .artifacts
            .iter()
            .any(|artifact| artifact.artifact_type == ArtifactType::Plan)
    );
    assert!(
        snapshot
            .checkpoints
            .iter()
            .any(|checkpoint| checkpoint.phase == "after-brainstorming")
    );
    assert!(
        snapshot
            .checkpoints
            .iter()
            .any(|checkpoint| checkpoint.phase == "after-writing-plans")
    );
    fs::remove_dir_all(root).expect("temp root should be removable");
}

#[test]
fn blocked_agent_does_not_stop_independent_agents() {
    let root = temp_root();
    let command = TaskCommand {
        runtime: Some(Runtime::Codex),
        objective: "Ship orchestration flow".to_owned(),
    };

    let run_dir = execute_task_workflow(&root, command).expect("workflow should succeed");
    let snapshot = load_task_snapshot(&run_dir).expect("snapshot should load");

    assert!(snapshot
        .agents
        .iter()
        .any(|agent| agent.current_state == OrchestratorState::Running));
    assert!(snapshot
        .agents
        .iter()
        .any(|agent| agent.current_state == OrchestratorState::WaitingForApproval));

    fs::remove_dir_all(root).expect("temp root should be removable");
}

#[test]
fn restart_recovery_reconstructs_waiting_for_approval_runs() {
    let root = temp_root();
    let run = PersistedTaskRun {
        run_id: "run-001".to_owned(),
        objective: "Recover run".to_owned(),
        runtime: "codex".to_owned(),
        current_phase: "writing-plans".to_owned(),
        overall_state: OrchestratorState::WaitingForApproval,
        blocking_reason: Some("approval required".to_owned()),
        workspace_root: "workspace".to_owned(),
        workspace_policy: "isolated_by_default".to_owned(),
        default_isolation: true,
        created_at: "2026-03-10T00:00:00Z".to_owned(),
        updated_at: "2026-03-10T00:00:00Z".to_owned(),
    };
    let run_dir = create_task_run(&root, &run).expect("run should persist");
    let checkpoint = patchlane::orchestration::model::PersistedCheckpoint {
        checkpoint_id: "checkpoint-001".to_owned(),
        run_id: "run-001".to_owned(),
        phase: "after-writing-plans".to_owned(),
        target_kind: "artifact".to_owned(),
        target_ref: "artifact-plan".to_owned(),
        requested_by: "agent-plan".to_owned(),
        status: patchlane::orchestration::model::CheckpointStatus::Pending,
        prompt_text: "Approve? [y/n]".to_owned(),
        response: None,
        note: None,
        created_at: "2026-03-10T00:00:00Z".to_owned(),
        updated_at: "2026-03-10T00:00:01Z".to_owned(),
    };
    patchlane::orchestration::store::write_checkpoint(&run_dir, &checkpoint)
        .expect("checkpoint should persist");
    let recovered = recover_run_state(&run_dir).expect("run should recover");

    assert_eq!(recovered.run.overall_state, OrchestratorState::WaitingForApproval);
    assert_eq!(
        recovered.pending_checkpoint.as_ref().map(|checkpoint| checkpoint.phase.as_str()),
        Some("after-writing-plans")
    );
    assert!(!recovered
        .blocked_agents
        .iter()
        .any(|agent| agent.current_state == OrchestratorState::Running));

    fs::remove_dir_all(root).expect("temp root should be removable");
}

#[test]
fn recovery_prefers_latest_pending_checkpoint_and_excludes_running_agents() {
    let root = temp_root();
    let run = PersistedTaskRun {
        run_id: "run-002".to_owned(),
        objective: "Recover latest pending checkpoint".to_owned(),
        runtime: "codex".to_owned(),
        current_phase: "review".to_owned(),
        overall_state: OrchestratorState::WaitingForApproval,
        blocking_reason: Some("approval required".to_owned()),
        workspace_root: "workspace".to_owned(),
        workspace_policy: "isolated_by_default".to_owned(),
        default_isolation: true,
        created_at: "2026-03-10T00:00:00Z".to_owned(),
        updated_at: "2026-03-10T00:00:02Z".to_owned(),
    };
    let run_dir = create_task_run(&root, &run).expect("run should persist");
    write_agent(
        &run_dir,
        &PersistedAgent {
            agent_id: "agent-running".to_owned(),
            run_id: "run-002".to_owned(),
            parent_agent_id: None,
            role: "implementer".to_owned(),
            current_phase: "coding".to_owned(),
            current_state: OrchestratorState::Running,
            runtime: "codex".to_owned(),
            workspace_path: "workspace".to_owned(),
            pid: None,
            related_artifact_ids: Vec::new(),
            stdout_log: "agent-running-stdout.log".to_owned(),
            stderr_log: "agent-running-stderr.log".to_owned(),
            created_at: "2026-03-10T00:00:00Z".to_owned(),
            updated_at: "2026-03-10T00:00:02Z".to_owned(),
        },
    )
    .expect("running agent should persist");
    write_agent(
        &run_dir,
        &PersistedAgent {
            agent_id: "agent-blocked".to_owned(),
            run_id: "run-002".to_owned(),
            parent_agent_id: None,
            role: "planner".to_owned(),
            current_phase: "review".to_owned(),
            current_state: OrchestratorState::WaitingForApproval,
            runtime: "codex".to_owned(),
            workspace_path: "workspace".to_owned(),
            pid: None,
            related_artifact_ids: Vec::new(),
            stdout_log: "agent-blocked-stdout.log".to_owned(),
            stderr_log: "agent-blocked-stderr.log".to_owned(),
            created_at: "2026-03-10T00:00:00Z".to_owned(),
            updated_at: "2026-03-10T00:00:02Z".to_owned(),
        },
    )
    .expect("blocked agent should persist");
    write_checkpoint(
        &run_dir,
        &PersistedCheckpoint {
            checkpoint_id: "checkpoint-old".to_owned(),
            run_id: "run-002".to_owned(),
            phase: "after-brainstorming".to_owned(),
            target_kind: "artifact".to_owned(),
            target_ref: "artifact-spec".to_owned(),
            requested_by: "agent-plan".to_owned(),
            status: CheckpointStatus::Pending,
            prompt_text: "Approve? [y/n]".to_owned(),
            response: None,
            note: None,
            created_at: "2026-03-10T00:00:00Z".to_owned(),
            updated_at: "2026-03-10T00:00:01Z".to_owned(),
        },
    )
    .expect("old checkpoint should persist");
    write_checkpoint(
        &run_dir,
        &PersistedCheckpoint {
            checkpoint_id: "checkpoint-new".to_owned(),
            run_id: "run-002".to_owned(),
            phase: "after-writing-plans".to_owned(),
            target_kind: "artifact".to_owned(),
            target_ref: "artifact-plan".to_owned(),
            requested_by: "agent-plan".to_owned(),
            status: CheckpointStatus::Pending,
            prompt_text: "Approve? [y/n]".to_owned(),
            response: None,
            note: None,
            created_at: "2026-03-10T00:00:00Z".to_owned(),
            updated_at: "2026-03-10T00:00:02Z".to_owned(),
        },
    )
    .expect("new checkpoint should persist");

    let recovered = recover_run_state(&run_dir).expect("run should recover");
    assert_eq!(
        recovered.pending_checkpoint.as_ref().map(|checkpoint| checkpoint.checkpoint_id.as_str()),
        Some("checkpoint-new")
    );
    assert_eq!(recovered.blocked_agents.len(), 1);
    assert_eq!(recovered.blocked_agents[0].agent_id, "agent-blocked");

    fs::remove_dir_all(root).expect("temp root should be removable");
}
