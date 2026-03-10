use patchlane::orchestration::agent_wrapper::AgentWrapper;
use patchlane::orchestration::model::{
    AgentEventType, ArtifactType, CheckpointStatus, OrchestratorState, PersistedAgent,
    PersistedCheckpoint, PersistedTaskRun,
};
use patchlane::orchestration::store::{
    create_task_run, load_task_snapshot, write_agent, write_checkpoint,
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
    let root = std::env::temp_dir().join(format!("patchlane-agent-wrapper-{unique}"));
    fs::create_dir_all(&root).expect("temp root should be creatable");
    root
}

fn persisted_run() -> PersistedTaskRun {
    PersistedTaskRun {
        run_id: "run-001".to_owned(),
        objective: "Ship orchestration flow".to_owned(),
        runtime: "codex".to_owned(),
        current_phase: "queued".to_owned(),
        overall_state: OrchestratorState::Queued,
        blocking_reason: None,
        workspace_root: "workspace".to_owned(),
        workspace_policy: "isolated_by_default".to_owned(),
        default_isolation: true,
        created_at: "2026-03-10T00:00:00Z".to_owned(),
        updated_at: "2026-03-10T00:00:00Z".to_owned(),
    }
}

fn persisted_agent(agent_id: &str) -> PersistedAgent {
    PersistedAgent {
        agent_id: agent_id.to_owned(),
        run_id: "run-001".to_owned(),
        parent_agent_id: None,
        role: "writing-plans".to_owned(),
        current_phase: "queued".to_owned(),
        current_state: OrchestratorState::Queued,
        runtime: "codex".to_owned(),
        workspace_path: "workspace".to_owned(),
        pid: None,
        related_artifact_ids: Vec::new(),
        stdout_log: "stdout.log".to_owned(),
        stderr_log: "stderr.log".to_owned(),
        created_at: "2026-03-10T00:00:00Z".to_owned(),
        updated_at: "2026-03-10T00:00:00Z".to_owned(),
    }
}

fn persisted_checkpoint() -> PersistedCheckpoint {
    PersistedCheckpoint {
        checkpoint_id: "checkpoint-001".to_owned(),
        run_id: "run-001".to_owned(),
        phase: "after-writing-plans".to_owned(),
        target_kind: "artifact".to_owned(),
        target_ref: "artifact-plan".to_owned(),
        requested_by: "agent-plan".to_owned(),
        status: CheckpointStatus::Pending,
        prompt_text: "Approve? [y/n]".to_owned(),
        response: None,
        note: None,
        created_at: "2026-03-10T00:00:00Z".to_owned(),
        updated_at: "2026-03-10T00:00:00Z".to_owned(),
    }
}

fn run_agent_event(run_dir: &PathBuf, event_type: &str, message: &str) {
    let output = Command::new(env!("CARGO_BIN_EXE_patchlane"))
        .args([
            "agent-event",
            event_type,
            "--run-dir",
            run_dir.to_str().expect("run dir should be utf-8"),
            "--run-id",
            "run-001",
            "--agent-id",
            "agent-plan",
            "--message",
            message,
        ])
        .output()
        .expect("agent-event command should run");
    assert!(
        output.status.success(),
        "agent-event should succeed, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn setup_run_dir(root: &PathBuf) -> PathBuf {
    let run_dir = create_task_run(root, &persisted_run()).expect("run should persist");
    write_agent(&run_dir, &persisted_agent("agent-plan")).expect("agent should persist");
    write_checkpoint(&run_dir, &persisted_checkpoint()).expect("checkpoint should persist");
    run_dir
}

fn latest_agent_event_payload(run_dir: &PathBuf, event_type: AgentEventType) -> String {
    load_task_snapshot(run_dir)
        .expect("snapshot should load")
        .events
        .into_iter()
        .rev()
        .find(|event| event.event_type == event_type)
        .map(|event| event.payload_summary)
        .expect("matching event should exist")
}

fn assert_transition_state(
    run_dir: &PathBuf,
    expected_run_state: OrchestratorState,
    expected_agent_state: OrchestratorState,
) {
    let snapshot = load_task_snapshot(run_dir).expect("snapshot should load");
    assert_eq!(snapshot.run.overall_state, expected_run_state);
    assert_eq!(snapshot.agents[0].current_state, expected_agent_state);
}

#[test]
fn wrapper_phase_and_artifact_events_update_persisted_agent_state() {
    let root = temp_root();
    let run_dir = setup_run_dir(&root);
    let agent = persisted_agent("agent-plan");

    let mut wrapper = AgentWrapper::new(run_dir.clone(), agent);
    wrapper.start().expect("start event should persist");
    assert_transition_state(&run_dir, OrchestratorState::Queued, OrchestratorState::Running);

    wrapper.review_start("spec-review").expect("review start should persist");
    assert_transition_state(&run_dir, OrchestratorState::Queued, OrchestratorState::InReview);

    wrapper.phase("writing-plans")
        .expect("phase event should persist");
    let snapshot = load_task_snapshot(&run_dir).expect("snapshot should load after phase");
    assert_eq!(snapshot.agents[0].current_phase, "writing-plans");
    assert_eq!(snapshot.agents[0].current_state, OrchestratorState::Running);

    wrapper
        .artifact(ArtifactType::Plan, "docs/superpowers/plans/plan.md")
        .expect("artifact event should persist");
    let snapshot = load_task_snapshot(&run_dir).expect("snapshot should load after artifact");
    assert_eq!(snapshot.artifacts[0].artifact_type, ArtifactType::Plan);
    assert_eq!(
        latest_agent_event_payload(&run_dir, AgentEventType::Artifact),
        "plan|docs/superpowers/plans/plan.md"
    );

    wrapper
        .waiting_approval("checkpoint-001", "Approve? [y/n]")
        .expect("waiting approval should persist");
    let snapshot =
        load_task_snapshot(&run_dir).expect("snapshot should load after waiting approval");
    assert_eq!(snapshot.run.overall_state, OrchestratorState::WaitingForApproval);
    assert_eq!(snapshot.agents[0].current_state, OrchestratorState::WaitingForApproval);
    assert_eq!(snapshot.checkpoints[0].checkpoint_id, "checkpoint-001");
    assert_eq!(snapshot.checkpoints[0].status, CheckpointStatus::Pending);
    assert_eq!(snapshot.checkpoints[0].prompt_text, "Approve? [y/n]");
    assert_eq!(
        latest_agent_event_payload(&run_dir, AgentEventType::WaitingApproval),
        "checkpoint-001|Approve? [y/n]"
    );

    wrapper
        .waiting_input("Need approval note")
        .expect("waiting input should persist");
    assert_transition_state(
        &run_dir,
        OrchestratorState::WaitingForInput,
        OrchestratorState::WaitingForInput,
    );

    wrapper.review_pass("looks good").expect("review pass should persist");
    let snapshot = load_task_snapshot(&run_dir).expect("snapshot should load after review pass");
    assert_eq!(snapshot.run.overall_state, OrchestratorState::WaitingForInput);
    assert_eq!(snapshot.agents[0].current_state, OrchestratorState::WaitingForInput);

    wrapper.review_fail("missing artifact").expect("review fail should persist");
    let snapshot = load_task_snapshot(&run_dir).expect("snapshot should load after review fail");
    assert_eq!(snapshot.run.overall_state, OrchestratorState::WaitingForInput);
    assert_eq!(snapshot.agents[0].current_state, OrchestratorState::Failed);

    wrapper.fail("launcher exited").expect("fail should persist");
    assert_transition_state(&run_dir, OrchestratorState::Failed, OrchestratorState::Failed);

    wrapper.done("plan accepted").expect("done should persist");
    assert_transition_state(&run_dir, OrchestratorState::Done, OrchestratorState::Done);

    let snapshot = load_task_snapshot(&run_dir).expect("snapshot should load");
    assert!(
        snapshot
            .events
            .iter()
            .any(|event| event.event_type == AgentEventType::WaitingApproval)
    );
    assert!(
        snapshot
            .events
            .iter()
            .any(|event| event.event_type == AgentEventType::ReviewStart)
    );
    assert!(
        snapshot
            .events
            .iter()
            .any(|event| event.event_type == AgentEventType::WaitingInput)
    );
    assert!(
        snapshot
            .events
            .iter()
            .any(|event| event.event_type == AgentEventType::ReviewPass)
    );
    assert!(
        snapshot
            .events
            .iter()
            .any(|event| event.event_type == AgentEventType::ReviewFail)
    );
    assert!(
        snapshot
            .events
            .iter()
            .any(|event| event.event_type == AgentEventType::Fail)
    );
    assert!(
        snapshot
            .events
            .iter()
            .any(|event| event.event_type == AgentEventType::Done)
    );

    fs::remove_dir_all(root).expect("temp root should be removable");
}

#[test]
fn cli_agent_event_updates_stateful_transitions_consistently() {
    let root = temp_root();
    let wrapper_run_dir = setup_run_dir(&root);
    let cli_run_dir = setup_run_dir(&root);

    let mut wrapper = AgentWrapper::new(wrapper_run_dir.clone(), persisted_agent("agent-plan"));
    wrapper
        .waiting_approval("checkpoint-001", "Approve? [y/n]")
        .expect("wrapper waiting approval should persist");
    run_agent_event(&cli_run_dir, "waiting-approval", "checkpoint-001|Approve? [y/n]");
    assert_eq!(
        latest_agent_event_payload(&wrapper_run_dir, AgentEventType::WaitingApproval),
        latest_agent_event_payload(&cli_run_dir, AgentEventType::WaitingApproval)
    );
    let wrapper_snapshot =
        load_task_snapshot(&wrapper_run_dir).expect("wrapper snapshot should load");
    let cli_snapshot = load_task_snapshot(&cli_run_dir).expect("cli snapshot should load");
    assert_eq!(wrapper_snapshot.run.overall_state, cli_snapshot.run.overall_state);
    assert_eq!(
        wrapper_snapshot.agents[0].current_state,
        cli_snapshot.agents[0].current_state
    );
    assert_eq!(
        wrapper_snapshot.checkpoints[0].prompt_text,
        cli_snapshot.checkpoints[0].prompt_text
    );
    assert_eq!(
        wrapper_snapshot.checkpoints[0].status,
        cli_snapshot.checkpoints[0].status
    );

    wrapper
        .waiting_input("Need approval note")
        .expect("wrapper waiting input should persist");
    run_agent_event(&cli_run_dir, "waiting-input", "Need approval note");
    let wrapper_snapshot =
        load_task_snapshot(&wrapper_run_dir).expect("wrapper snapshot should load");
    let cli_snapshot = load_task_snapshot(&cli_run_dir).expect("cli snapshot should load");
    assert_eq!(wrapper_snapshot.run.overall_state, cli_snapshot.run.overall_state);
    assert_eq!(
        wrapper_snapshot.agents[0].current_state,
        cli_snapshot.agents[0].current_state
    );

    wrapper
        .review_start("spec-review")
        .expect("wrapper review start should persist");
    run_agent_event(&cli_run_dir, "review-start", "spec-review");
    let wrapper_snapshot =
        load_task_snapshot(&wrapper_run_dir).expect("wrapper snapshot should load");
    let cli_snapshot = load_task_snapshot(&cli_run_dir).expect("cli snapshot should load");
    assert_eq!(
        wrapper_snapshot.agents[0].current_state,
        cli_snapshot.agents[0].current_state
    );

    wrapper
        .review_pass("looks good")
        .expect("wrapper review pass should persist");
    run_agent_event(&cli_run_dir, "review-pass", "looks good");
    let wrapper_snapshot =
        load_task_snapshot(&wrapper_run_dir).expect("wrapper snapshot should load");
    let cli_snapshot = load_task_snapshot(&cli_run_dir).expect("cli snapshot should load");
    assert_eq!(wrapper_snapshot.run.overall_state, cli_snapshot.run.overall_state);
    assert_eq!(
        wrapper_snapshot.agents[0].current_state,
        cli_snapshot.agents[0].current_state
    );
    assert_eq!(
        latest_agent_event_payload(&wrapper_run_dir, AgentEventType::ReviewPass),
        latest_agent_event_payload(&cli_run_dir, AgentEventType::ReviewPass)
    );

    wrapper
        .review_fail("missing artifact")
        .expect("wrapper review fail should persist");
    run_agent_event(&cli_run_dir, "review-fail", "missing artifact");
    let wrapper_snapshot =
        load_task_snapshot(&wrapper_run_dir).expect("wrapper snapshot should load");
    let cli_snapshot = load_task_snapshot(&cli_run_dir).expect("cli snapshot should load");
    assert_eq!(wrapper_snapshot.run.overall_state, cli_snapshot.run.overall_state);
    assert_eq!(
        wrapper_snapshot.agents[0].current_state,
        cli_snapshot.agents[0].current_state
    );

    wrapper.fail("launcher exited").expect("wrapper fail should persist");
    run_agent_event(&cli_run_dir, "fail", "launcher exited");
    let wrapper_snapshot =
        load_task_snapshot(&wrapper_run_dir).expect("wrapper snapshot should load");
    let cli_snapshot = load_task_snapshot(&cli_run_dir).expect("cli snapshot should load");
    assert_eq!(wrapper_snapshot.run.overall_state, cli_snapshot.run.overall_state);
    assert_eq!(
        wrapper_snapshot.agents[0].current_state,
        cli_snapshot.agents[0].current_state
    );

    wrapper.done("plan accepted").expect("wrapper done should persist");
    run_agent_event(&cli_run_dir, "done", "plan accepted");
    let wrapper_snapshot =
        load_task_snapshot(&wrapper_run_dir).expect("wrapper snapshot should load");
    let cli_snapshot = load_task_snapshot(&cli_run_dir).expect("cli snapshot should load");
    assert_eq!(wrapper_snapshot.run.overall_state, cli_snapshot.run.overall_state);
    assert_eq!(
        wrapper_snapshot.agents[0].current_state,
        cli_snapshot.agents[0].current_state
    );
    assert_eq!(
        latest_agent_event_payload(&wrapper_run_dir, AgentEventType::Done),
        latest_agent_event_payload(&cli_run_dir, AgentEventType::Done)
    );

    fs::remove_dir_all(root).expect("temp root should be removable");
}
