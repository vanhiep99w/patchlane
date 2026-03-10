use crate::support::task_fixtures::{
    fixture_task_snapshot, fixture_task_snapshot_with_blockers, persisted_state_root_with_two_runs,
    temp_state_root,
};
use crossterm::event::KeyCode;
use patchlane::orchestration::model::PersistedTaskRun;
use patchlane::orchestration::store::create_task_run;
use patchlane::tui::app::{Pane, TuiApp};
use patchlane::tui::render::render_to_test_buffer;
use std::fs;

#[test]
fn selected_agent_view_includes_required_detail_fields() {
    let app = TuiApp::from_snapshot(fixture_task_snapshot());
    let detail = app.selected_agent_detail().expect("detail should exist");

    assert_eq!(detail.current_phase, "writing-plans");
    assert_eq!(detail.current_state, "waiting_for_approval");
    assert!(
        detail
            .timeline
            .iter()
            .any(|event| event.event_type == "waiting-approval")
    );
    assert!(
        detail
            .blockers
            .iter()
            .any(|blocker| blocker.contains("Approve? [y/n]"))
    );
    assert!(
        detail
            .artifacts
            .iter()
            .any(|artifact| artifact.artifact_type == "plan")
    );
    assert!(
        detail
            .artifacts
            .iter()
            .any(|artifact| artifact.path.ends_with("docs/superpowers/plans/plan.md"))
    );
    assert!(detail.stdout_log.ends_with("agent-plan-stdout.log"));
    assert!(detail.stderr_log.ends_with("agent-plan-stderr.log"));
}

#[test]
fn tui_loads_runs_from_persisted_store_and_refreshes() {
    let state_root = persisted_state_root_with_two_runs();
    let mut app = TuiApp::load_from_store(&state_root).expect("app should load");

    assert_eq!(app.runs().len(), 2);
    app.refresh().expect("refresh should succeed");
    assert!(
        app.agent_rows()
            .iter()
            .any(|row| row.state == "waiting_for_input")
    );
}

#[test]
fn tui_refresh_preserves_selected_run_and_agent_by_identity() {
    let state_root = persisted_state_root_with_two_runs();
    let mut app = TuiApp::load_from_store(&state_root).expect("app should load");

    app.handle_key(KeyCode::Char('j'));
    app.handle_key(KeyCode::Tab);
    let selected_run_before = app
        .selected_run()
        .expect("run should be selected")
        .run
        .run_id
        .clone();
    let selected_agent_before = app
        .selected_agent()
        .expect("agent should be selected")
        .agent_id;

    let run_json = state_root.join("tasks/run-task-002/run.json");
    let updated_run = PersistedTaskRun {
        run_id: "run-task-002".to_owned(),
        objective: "Ship orchestration flow".to_owned(),
        runtime: "codex".to_owned(),
        current_phase: "writing-plans".to_owned(),
        overall_state: patchlane::orchestration::model::OrchestratorState::WaitingForApproval,
        blocking_reason: Some("checkpoint pending".to_owned()),
        workspace_root: "workspace".to_owned(),
        workspace_policy: "isolated_by_default".to_owned(),
        default_isolation: true,
        created_at: "2026-03-10T10:00:00Z".to_owned(),
        updated_at: "2026-03-10T12:30:00Z".to_owned(),
    };
    fs::write(
        &run_json,
        serde_json::to_string_pretty(&updated_run).expect("run json should serialize"),
    )
    .expect("updated run should persist");

    app.refresh().expect("refresh should succeed");

    assert_eq!(
        app.selected_run()
            .expect("run should stay selected")
            .run
            .run_id,
        selected_run_before
    );
    assert_eq!(
        app.selected_agent()
            .expect("agent should stay selected")
            .agent_id,
        selected_agent_before
    );
}

#[test]
fn tui_load_from_store_errors_on_corrupt_run() {
    let state_root = temp_state_root("patchlane-tui-corrupt");
    let tasks_root = state_root.join("tasks");

    create_task_run(
        &tasks_root,
        &PersistedTaskRun {
            run_id: "run-task-good".to_owned(),
            objective: "Valid run".to_owned(),
            runtime: "codex".to_owned(),
            current_phase: "writing-plans".to_owned(),
            overall_state: patchlane::orchestration::model::OrchestratorState::Running,
            blocking_reason: None,
            workspace_root: "workspace".to_owned(),
            workspace_policy: "isolated_by_default".to_owned(),
            default_isolation: true,
            created_at: "2026-03-10T10:00:00Z".to_owned(),
            updated_at: "2026-03-10T10:01:00Z".to_owned(),
        },
    )
    .expect("valid run should persist");

    let bad_run_dir = tasks_root.join("run-task-bad");
    fs::create_dir_all(&bad_run_dir).expect("bad run dir should persist");
    fs::write(bad_run_dir.join("run.json"), "{invalid").expect("bad run should persist");

    let error = match TuiApp::load_from_store(&state_root) {
        Ok(_) => panic!("corrupt run should surface"),
        Err(error) => error,
    };
    assert!(
        error.to_string().contains("run-task-bad"),
        "error should identify corrupt run, got: {error}"
    );
}

#[test]
fn tui_lists_runs_and_agents_and_highlights_blockers() {
    let app = TuiApp::from_snapshots(fixture_task_snapshot_with_blockers());

    assert_eq!(app.runs().len(), 2);
    assert!(app.agent_rows().iter().any(|row| row.state == "failed"));
    assert!(
        app.agent_rows()
            .iter()
            .any(|row| row.state == "waiting_for_input")
    );
}

#[test]
fn render_frame_shows_timeline_artifacts_and_log_tail() {
    let state_root = persisted_state_root_with_two_runs();
    let mut app = TuiApp::load_from_store(&state_root).expect("app should load");
    app.handle_key(KeyCode::Char('j'));
    let frame = render_to_test_buffer(app);

    assert!(frame.contains("Timeline"));
    assert!(frame.contains("Artifacts"));
    assert!(frame.contains("docs/superpowers/plans/plan.md"));
    assert!(frame.contains("Logs"));
    assert!(frame.contains("phase: writing-plans"));
    assert!(frame.contains("Approve? [y/n]"));
}

#[test]
fn render_frame_surfaces_missing_log_files() {
    let state_root = persisted_state_root_with_two_runs();
    fs::remove_file(state_root.join("tasks/run-task-002/logs/agent-plan-stdout.log"))
        .expect("stdout log should be removable");

    let mut app = TuiApp::load_from_store(&state_root).expect("app should load");
    app.handle_key(KeyCode::Char('j'));
    let frame = render_to_test_buffer(app);

    assert!(
        frame.contains("log unavailable:"),
        "missing logs should be surfaced in the panel, got: {frame}"
    );
}

#[test]
fn stderr_log_panel_is_available_for_selected_agent() {
    let detail = TuiApp::from_snapshot(fixture_task_snapshot())
        .selected_agent_detail()
        .expect("detail should exist");
    assert!(detail.stderr_log.ends_with("agent-plan-stderr.log"));
}

#[test]
fn detail_pane_renders_selected_agent_blockers_not_run_wide_blockers() {
    let frame = render_to_test_buffer(TuiApp::from_snapshot(fixture_task_snapshot()));

    assert!(frame.contains("Approve? [y/n]"));
    assert!(
        !frame.contains("checkpoint pending"),
        "detail pane should not render run-level blockers: {frame}"
    );
}

#[test]
fn agent_rows_ignore_stale_blocker_history_after_agent_recovers() {
    let mut snapshot = fixture_task_snapshot();
    snapshot.agents[1].current_state = patchlane::orchestration::model::OrchestratorState::Done;
    snapshot.checkpoints.clear();
    snapshot.events.push(patchlane::orchestration::model::PersistedTaskEvent {
        event_id: "event-plan-done".to_owned(),
        run_id: snapshot.run.run_id.clone(),
        agent_id: Some("agent-plan".to_owned()),
        event_type: patchlane::orchestration::model::AgentEventType::Done,
        payload_summary: "writing-plans complete".to_owned(),
        timestamp: "2026-03-10T10:07:00Z".to_owned(),
    });

    let app = TuiApp::from_snapshot(snapshot);
    let recovered_agent = app
        .agent_rows()
        .into_iter()
        .find(|row| row.agent_id == "agent-plan")
        .expect("agent should exist");

    assert!(
        !recovered_agent.has_blocker,
        "stale waiting-approval history should not keep blocker badges"
    );
    assert_eq!(app.runs()[0].blocker_count, 1);
}

#[test]
fn selected_agent_blockers_match_current_blocked_state() {
    let mut snapshot = fixture_task_snapshot();
    snapshot.agents[1].current_state = patchlane::orchestration::model::OrchestratorState::Failed;
    snapshot.checkpoints.clear();
    snapshot.events.push(patchlane::orchestration::model::PersistedTaskEvent {
        event_id: "event-plan-fail".to_owned(),
        run_id: snapshot.run.run_id.clone(),
        agent_id: Some("agent-plan".to_owned()),
        event_type: patchlane::orchestration::model::AgentEventType::Fail,
        payload_summary: "artifact write failed".to_owned(),
        timestamp: "2026-03-10T10:07:00Z".to_owned(),
    });
    snapshot.events.push(patchlane::orchestration::model::PersistedTaskEvent {
        event_id: "event-plan-wait-input".to_owned(),
        run_id: snapshot.run.run_id.clone(),
        agent_id: Some("agent-plan".to_owned()),
        event_type: patchlane::orchestration::model::AgentEventType::WaitingInput,
        payload_summary: "Need operator clarification".to_owned(),
        timestamp: "2026-03-10T10:08:00Z".to_owned(),
    });

    let app = TuiApp::from_snapshot(snapshot);
    let detail = app
        .selected_agent_detail()
        .expect("selected agent detail should exist");

    assert!(
        detail
            .blockers
            .iter()
            .any(|blocker| blocker.contains("artifact write failed"))
    );
    assert!(
        !detail
            .blockers
            .iter()
            .any(|blocker| blocker.contains("Need operator clarification")),
        "failed agents should not surface stale waiting-input prompts"
    );
}

#[test]
fn selected_agent_blockers_dedupe_checkpoint_and_waiting_approval_event() {
    let app = TuiApp::from_snapshot(fixture_task_snapshot());
    let detail = app
        .selected_agent_detail()
        .expect("selected agent detail should exist");

    let approval_blockers = detail
        .blockers
        .iter()
        .filter(|blocker| blocker.contains("Approve? [y/n]"))
        .collect::<Vec<_>>();

    assert_eq!(approval_blockers.len(), 1);
    assert_eq!(approval_blockers[0].as_str(), "Approve? [y/n]");
}

#[test]
fn key_handling_switches_panes_and_selection() {
    let mut app = TuiApp::from_snapshot(fixture_task_snapshot());
    app.handle_key(KeyCode::Tab);
    app.handle_key(KeyCode::Char('j'));
    assert_eq!(app.active_pane(), Pane::AgentList);
}
