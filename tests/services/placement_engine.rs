use patchlane::domain::placement::PlacementState;
use patchlane::services::placement_engine::{
    decide_placement, PlacementDecisionInput, PlacementMode,
};

#[test]
fn placement_engine_prefers_main_repo_for_a_single_low_risk_shard() {
    let decision = decide_placement(PlacementDecisionInput {
        mode: PlacementMode::Balanced,
        shard_count: 1,
        writable_shard_count: 1,
        has_overlap_risk: false,
        repo_is_dirty: false,
        blocked_reason: None,
    });

    assert_eq!(decision.placement, PlacementState::MainRepo);
    assert_eq!(
        decision.reason,
        "single low-risk shard can stay in main repo"
    );
    assert_eq!(decision.block_reason, None);
}

#[test]
fn placement_engine_prefers_worktree_for_multiple_writable_shards() {
    let decision = decide_placement(PlacementDecisionInput {
        mode: PlacementMode::Balanced,
        shard_count: 3,
        writable_shard_count: 2,
        has_overlap_risk: false,
        repo_is_dirty: false,
        blocked_reason: None,
    });

    assert_eq!(decision.placement, PlacementState::Worktree);
    assert_eq!(
        decision.reason,
        "multiple writable shards need isolated worktrees"
    );
    assert_eq!(decision.block_reason, None);
}

#[test]
fn placement_engine_prefers_worktree_for_overlap_risk() {
    let decision = decide_placement(PlacementDecisionInput {
        mode: PlacementMode::Balanced,
        shard_count: 1,
        writable_shard_count: 1,
        has_overlap_risk: true,
        repo_is_dirty: false,
        blocked_reason: None,
    });

    assert_eq!(decision.placement, PlacementState::Worktree);
    assert_eq!(
        decision.reason,
        "overlap risk requires an isolated worktree"
    );
    assert_eq!(decision.block_reason, None);
}

#[test]
fn placement_engine_prefers_worktree_for_multiple_shards_with_one_writer() {
    let decision = decide_placement(PlacementDecisionInput {
        mode: PlacementMode::Balanced,
        shard_count: 3,
        writable_shard_count: 1,
        has_overlap_risk: false,
        repo_is_dirty: false,
        blocked_reason: None,
    });

    assert_eq!(decision.placement, PlacementState::Worktree);
    assert_eq!(
        decision.reason,
        "multiple shards still need isolated worktrees"
    );
    assert_eq!(decision.block_reason, None);
}

#[test]
fn placement_engine_prefers_worktree_when_repo_is_dirty() {
    let decision = decide_placement(PlacementDecisionInput {
        mode: PlacementMode::Balanced,
        shard_count: 1,
        writable_shard_count: 1,
        has_overlap_risk: false,
        repo_is_dirty: true,
        blocked_reason: None,
    });

    assert_eq!(decision.placement, PlacementState::Worktree);
    assert_eq!(
        decision.reason,
        "dirty repo state requires an isolated worktree"
    );
    assert_eq!(decision.block_reason, None);
}

#[test]
fn placement_engine_prefers_worktree_in_safe_mode() {
    let decision = decide_placement(PlacementDecisionInput {
        mode: PlacementMode::Safe,
        shard_count: 1,
        writable_shard_count: 1,
        has_overlap_risk: false,
        repo_is_dirty: false,
        blocked_reason: None,
    });

    assert_eq!(decision.placement, PlacementState::Worktree);
    assert_eq!(
        decision.reason,
        "safe mode keeps execution inside a worktree"
    );
    assert_eq!(decision.block_reason, None);
}

#[test]
fn placement_engine_blocks_when_dispatch_conditions_fail() {
    let decision = decide_placement(PlacementDecisionInput {
        mode: PlacementMode::Fast,
        shard_count: 2,
        writable_shard_count: 1,
        has_overlap_risk: true,
        repo_is_dirty: false,
        blocked_reason: Some("operator approval required"),
    });

    assert_eq!(decision.placement, PlacementState::Blocked);
    assert_eq!(
        decision.reason,
        "dispatch blocked until operator intervention"
    );
    assert_eq!(
        decision.block_reason,
        Some("operator approval required".to_owned())
    );
}
