use patchlane::orchestration::approval::handle_approval_input;
use patchlane::orchestration::checkpoints::build_phase_checkpoints;
use patchlane::orchestration::model::{
    CheckpointStatus, OrchestratorState, PersistedTaskRun,
};
use patchlane::orchestration::store::{create_task_run, load_task_snapshot};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_root() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("patchlane-checkpoints-{unique}"));
    fs::create_dir_all(&root).expect("temp root should be creatable");
    root
}

fn persisted_run() -> PersistedTaskRun {
    PersistedTaskRun {
        run_id: "run-001".to_owned(),
        objective: "Design task flow".to_owned(),
        runtime: "codex".to_owned(),
        current_phase: "writing-plans".to_owned(),
        overall_state: OrchestratorState::Queued,
        blocking_reason: None,
        workspace_root: "workspace".to_owned(),
        workspace_policy: "isolated_by_default".to_owned(),
        default_isolation: true,
        created_at: "2026-03-10T00:00:00Z".to_owned(),
        updated_at: "2026-03-10T00:00:00Z".to_owned(),
    }
}

#[test]
fn checkpoint_builder_covers_v1_checkpoint_classes() {
    let checkpoints = build_phase_checkpoints("run-001", "agent-plan");
    assert!(checkpoints.iter().any(|checkpoint| checkpoint.phase == "after-brainstorming"));
    assert!(checkpoints.iter().any(|checkpoint| checkpoint.phase == "after-writing-plans"));
    assert!(checkpoints.iter().any(|checkpoint| checkpoint.phase == "request-more-information"));
    assert!(checkpoints.iter().any(|checkpoint| checkpoint.phase == "review-intervention"));
    assert!(checkpoints.iter().any(|checkpoint| checkpoint.phase == "before-branch-finishing"));
}

#[test]
fn rejected_checkpoint_moves_run_into_waiting_for_input() {
    let root = temp_root();
    let run_dir = create_task_run(&root, &persisted_run()).expect("run should persist");
    let checkpoint = build_phase_checkpoints("run-001", "agent-plan")
        .into_iter()
        .find(|checkpoint| checkpoint.phase == "after-writing-plans")
        .expect("checkpoint should exist");

    let updated = handle_approval_input("n", checkpoint, &run_dir).expect("rejection should persist");
    let snapshot = load_task_snapshot(&run_dir).expect("snapshot should load");
    assert_eq!(updated.status, CheckpointStatus::Rejected);
    assert_eq!(snapshot.run.overall_state, OrchestratorState::WaitingForInput);

    fs::remove_dir_all(root).expect("temp root should be removable");
}

#[test]
fn approved_checkpoint_persists_requester_target_and_continues_run() {
    let root = temp_root();
    let run_dir = create_task_run(&root, &persisted_run()).expect("run should persist");
    let checkpoint = build_phase_checkpoints("run-001", "agent-plan")
        .into_iter()
        .find(|checkpoint| checkpoint.phase == "after-writing-plans")
        .expect("checkpoint should exist");

    let updated = handle_approval_input("y", checkpoint, &run_dir).expect("approval should persist");
    let snapshot = load_task_snapshot(&run_dir).expect("snapshot should load");
    assert_eq!(updated.status, CheckpointStatus::Approved);
    assert_eq!(updated.requested_by, "agent-plan");
    assert_eq!(updated.target_ref, "artifact-plan");
    assert_eq!(snapshot.run.overall_state, OrchestratorState::Running);

    fs::remove_dir_all(root).expect("temp root should be removable");
}
