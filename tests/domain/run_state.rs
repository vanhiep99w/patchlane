use patchlane::domain::{
    intervention::InterventionResult, placement::PlacementState, run::RunState, shard::ShardState,
};
use serde_json::json;

#[test]
fn run_state_exposes_operator_visible_states_and_transitions() {
    assert_eq!(serde_json::to_value(RunState::Queued).unwrap(), json!("queued"));
    assert_eq!(serde_json::to_value(RunState::Running).unwrap(), json!("running"));
    assert_eq!(serde_json::to_value(RunState::Paused).unwrap(), json!("paused"));
    assert_eq!(serde_json::to_value(RunState::Succeeded).unwrap(), json!("succeeded"));
    assert_eq!(serde_json::to_value(RunState::Failed).unwrap(), json!("failed"));
    assert_eq!(serde_json::to_value(RunState::Stopped).unwrap(), json!("stopped"));

    assert_eq!(RunState::Queued.start(), Some(RunState::Running));
    assert_eq!(RunState::Running.pause(), Some(RunState::Paused));
    assert_eq!(RunState::Paused.resume(), Some(RunState::Running));
    assert_eq!(RunState::Running.succeed(), Some(RunState::Succeeded));
    assert_eq!(RunState::Running.fail(), Some(RunState::Failed));
    assert_eq!(RunState::Running.stop(), Some(RunState::Stopped));

    assert!(RunState::Succeeded.is_terminal());
    assert!(RunState::Failed.is_terminal());
    assert!(RunState::Stopped.is_terminal());
    assert!(!RunState::Running.is_terminal());
}

#[test]
fn run_state_rejects_invalid_transitions_and_round_trips_through_serde() {
    assert_eq!(RunState::Running.start(), None);
    assert_eq!(RunState::Queued.pause(), None);
    assert_eq!(RunState::Queued.resume(), None);
    assert_eq!(RunState::Paused.succeed(), None);
    assert_eq!(RunState::Stopped.fail(), None);
    assert_eq!(RunState::Succeeded.stop(), None);

    assert_eq!(
        serde_json::from_str::<RunState>("\"paused\"").unwrap(),
        RunState::Paused
    );
    assert_eq!(
        serde_json::from_value::<RunState>(serde_json::to_value(RunState::Stopped).unwrap())
            .unwrap(),
        RunState::Stopped
    );
}

#[test]
fn shard_state_exposes_operator_visible_states_and_transitions() {
    assert_eq!(serde_json::to_value(ShardState::Queued).unwrap(), json!("queued"));
    assert_eq!(
        serde_json::to_value(ShardState::Assigned).unwrap(),
        json!("assigned")
    );
    assert_eq!(serde_json::to_value(ShardState::Running).unwrap(), json!("running"));
    assert_eq!(
        serde_json::to_value(ShardState::Succeeded).unwrap(),
        json!("succeeded")
    );
    assert_eq!(serde_json::to_value(ShardState::Failed).unwrap(), json!("failed"));
    assert_eq!(serde_json::to_value(ShardState::Blocked).unwrap(), json!("blocked"));

    assert_eq!(ShardState::Queued.assign(), Some(ShardState::Assigned));
    assert_eq!(ShardState::Assigned.start(), Some(ShardState::Running));
    assert_eq!(ShardState::Running.succeed(), Some(ShardState::Succeeded));
    assert_eq!(ShardState::Running.fail(), Some(ShardState::Failed));
    assert_eq!(ShardState::Assigned.block(), Some(ShardState::Blocked));

    assert!(ShardState::Succeeded.is_terminal());
    assert!(ShardState::Failed.is_terminal());
    assert!(ShardState::Blocked.is_terminal());
    assert!(!ShardState::Assigned.is_terminal());
}

#[test]
fn shard_state_rejects_invalid_transitions() {
    assert_eq!(ShardState::Running.assign(), None);
    assert_eq!(ShardState::Queued.start(), None);
    assert_eq!(ShardState::Queued.succeed(), None);
    assert_eq!(ShardState::Blocked.fail(), None);
    assert_eq!(ShardState::Succeeded.block(), None);
}

#[test]
fn placement_state_serializes_with_cli_facing_names() {
    assert_eq!(
        serde_json::to_value(PlacementState::MainRepo).unwrap(),
        json!("main_repo")
    );
    assert_eq!(
        serde_json::to_value(PlacementState::Worktree).unwrap(),
        json!("worktree")
    );
    assert_eq!(
        serde_json::to_value(PlacementState::Blocked).unwrap(),
        json!("blocked")
    );
}

#[test]
fn placement_state_round_trips_through_serde() {
    assert_eq!(
        serde_json::from_str::<PlacementState>("\"worktree\"").unwrap(),
        PlacementState::Worktree
    );
    assert_eq!(
        serde_json::from_value::<PlacementState>(
            serde_json::to_value(PlacementState::MainRepo).unwrap()
        )
        .unwrap(),
        PlacementState::MainRepo
    );
}

#[test]
fn intervention_result_exposes_operator_visible_states_and_transitions() {
    assert_eq!(
        serde_json::to_value(InterventionResult::Queued).unwrap(),
        json!("queued")
    );
    assert_eq!(
        serde_json::to_value(InterventionResult::Acknowledged).unwrap(),
        json!("acknowledged")
    );
    assert_eq!(
        serde_json::to_value(InterventionResult::Applied).unwrap(),
        json!("applied")
    );
    assert_eq!(
        serde_json::to_value(InterventionResult::Failed).unwrap(),
        json!("failed")
    );

    assert_eq!(
        InterventionResult::Queued.acknowledge(),
        Some(InterventionResult::Acknowledged)
    );
    assert_eq!(
        InterventionResult::Acknowledged.apply(),
        Some(InterventionResult::Applied)
    );
    assert_eq!(
        InterventionResult::Queued.fail(),
        Some(InterventionResult::Failed)
    );
    assert_eq!(
        InterventionResult::Acknowledged.fail(),
        Some(InterventionResult::Failed)
    );
    assert!(InterventionResult::Applied.is_terminal());
    assert!(InterventionResult::Failed.is_terminal());
    assert!(!InterventionResult::Queued.is_terminal());
}

#[test]
fn intervention_result_rejects_invalid_transitions_and_round_trips_through_serde() {
    assert_eq!(InterventionResult::Applied.acknowledge(), None);
    assert_eq!(InterventionResult::Queued.apply(), None);
    assert_eq!(InterventionResult::Applied.fail(), None);
    assert_eq!(InterventionResult::Failed.apply(), None);

    assert_eq!(
        serde_json::from_str::<InterventionResult>("\"acknowledged\"").unwrap(),
        InterventionResult::Acknowledged
    );
    assert_eq!(
        serde_json::from_value::<InterventionResult>(
            serde_json::to_value(InterventionResult::Applied).unwrap()
        )
        .unwrap(),
        InterventionResult::Applied
    );
}
