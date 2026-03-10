use patchlane::orchestration::model::{
    AgentEventType, ArtifactType, CheckpointStatus, OrchestratorState, PersistedAgent,
    PersistedArtifact, PersistedCheckpoint, PersistedTaskEvent, PersistedTaskRun,
};
use patchlane::orchestration::store::{
    append_task_event, create_task_run, load_task_snapshot, write_agent, write_artifact,
    write_checkpoint,
};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_root() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("patchlane-orchestration-store-{unique}"));
    fs::create_dir_all(&root).expect("temp root should be creatable");
    root
}

#[test]
fn orchestration_store_persists_run_agents_checkpoints_artifacts_events_and_logs() {
    let root = temp_root();
    let run = PersistedTaskRun {
        run_id: "run-001".to_owned(),
        objective: "Ship orchestration".to_owned(),
        runtime: "codex".to_owned(),
        current_phase: "brainstorming".to_owned(),
        overall_state: OrchestratorState::Running,
        blocking_reason: None,
        workspace_root: "run-001/session-root".to_owned(),
        workspace_policy: "isolated_by_default".to_owned(),
        default_isolation: true,
        created_at: "2026-03-10T00:00:00Z".to_owned(),
        updated_at: "2026-03-10T00:00:00Z".to_owned(),
    };
    let agent = PersistedAgent {
        agent_id: "agent-brainstorm".to_owned(),
        run_id: run.run_id.clone(),
        parent_agent_id: None,
        role: "brainstorming".to_owned(),
        current_phase: "brainstorming".to_owned(),
        current_state: OrchestratorState::Running,
        runtime: "codex".to_owned(),
        workspace_path: "run-001/session-root".to_owned(),
        pid: Some(1234),
        related_artifact_ids: vec!["artifact-001".to_owned()],
        stdout_log: "logs/agent-brainstorm-stdout.log".to_owned(),
        stderr_log: "logs/agent-brainstorm-stderr.log".to_owned(),
        created_at: "2026-03-10T00:00:00Z".to_owned(),
        updated_at: "2026-03-10T00:00:00Z".to_owned(),
    };
    let checkpoint = PersistedCheckpoint {
        checkpoint_id: "checkpoint-001".to_owned(),
        run_id: run.run_id.clone(),
        phase: "after-brainstorming".to_owned(),
        target_kind: "artifact".to_owned(),
        target_ref: "artifact-001".to_owned(),
        requested_by: agent.agent_id.clone(),
        status: CheckpointStatus::Pending,
        prompt_text: "Approve? [y/n]".to_owned(),
        response: None,
        note: None,
        created_at: "2026-03-10T00:00:00Z".to_owned(),
        updated_at: "2026-03-10T00:00:00Z".to_owned(),
    };
    let artifact = PersistedArtifact {
        artifact_id: "artifact-001".to_owned(),
        run_id: run.run_id.clone(),
        producing_agent_id: agent.agent_id.clone(),
        artifact_type: ArtifactType::Spec,
        path: "docs/spec.md".to_owned(),
        created_at: "2026-03-10T00:00:00Z".to_owned(),
    };

    let run_dir = create_task_run(&root, &run).expect("run should persist");
    write_agent(&run_dir, &agent).expect("agent should persist");
    write_checkpoint(&run_dir, &checkpoint).expect("checkpoint should persist");
    write_artifact(&run_dir, &artifact).expect("artifact should persist");
    append_task_event(
        &run_dir,
        &PersistedTaskEvent {
            event_id: "event-001".to_owned(),
            run_id: run.run_id.clone(),
            agent_id: Some(agent.agent_id.clone()),
            event_type: AgentEventType::Phase,
            payload_summary: "brainstorming".to_owned(),
            timestamp: "2026-03-10T00:00:00Z".to_owned(),
        },
    )
    .expect("event should persist");
    fs::write(run_dir.join("logs/agent-brainstorm-stdout.log"), "spec draft\n")
        .expect("stdout log should persist");
    fs::write(run_dir.join("logs/agent-brainstorm-stderr.log"), "")
        .expect("stderr log should persist");

    assert!(run_dir.join("run.json").is_file());
    assert!(run_dir.join("agents").is_dir());
    assert!(run_dir.join("checkpoints").is_dir());
    assert!(run_dir.join("artifacts").is_dir());
    assert!(run_dir.join("events.jsonl").is_file());
    assert!(run_dir.join("logs").is_dir());

    let snapshot = load_task_snapshot(&run_dir).expect("snapshot should load");
    assert_eq!(snapshot.run.current_phase, "brainstorming");
    assert_eq!(snapshot.run.overall_state, OrchestratorState::Running);
    assert_eq!(snapshot.run.workspace_policy, "isolated_by_default");
    assert!(snapshot.run.default_isolation);
    assert_eq!(snapshot.agents.len(), 1);
    assert_eq!(snapshot.agents[0].pid, Some(1234));
    assert_eq!(snapshot.checkpoints[0].prompt_text, "Approve? [y/n]");
    assert_eq!(snapshot.checkpoints[0].status, CheckpointStatus::Pending);
    assert_eq!(snapshot.artifacts[0].producing_agent_id, "agent-brainstorm");
    assert_eq!(snapshot.artifacts[0].artifact_type, ArtifactType::Spec);
    assert_eq!(snapshot.events[0].event_type, AgentEventType::Phase);
    assert!(run_dir.join("logs/agent-brainstorm-stdout.log").is_file());
    assert!(run_dir.join("logs/agent-brainstorm-stderr.log").is_file());

    fs::remove_dir_all(root).expect("temp root should be removable");
}
