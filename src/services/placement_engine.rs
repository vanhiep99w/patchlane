use crate::domain::placement::PlacementState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacementMode {
    Fast,
    Balanced,
    Safe,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlacementDecisionInput<'a> {
    pub mode: PlacementMode,
    pub shard_count: usize,
    pub writable_shard_count: usize,
    pub has_overlap_risk: bool,
    pub repo_is_dirty: bool,
    pub blocked_reason: Option<&'a str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlacementDecision {
    pub placement: PlacementState,
    pub reason: &'static str,
    pub block_reason: Option<String>,
}

pub fn decide_placement(input: PlacementDecisionInput<'_>) -> PlacementDecision {
    if let Some(block_reason) = input.blocked_reason {
        return PlacementDecision {
            placement: PlacementState::Blocked,
            reason: "dispatch blocked until operator intervention",
            block_reason: Some(block_reason.to_owned()),
        };
    }

    if matches!(input.mode, PlacementMode::Safe) {
        return PlacementDecision {
            placement: PlacementState::Worktree,
            reason: "safe mode keeps execution inside a worktree",
            block_reason: None,
        };
    }

    if input.repo_is_dirty {
        return PlacementDecision {
            placement: PlacementState::Worktree,
            reason: "dirty repo state requires an isolated worktree",
            block_reason: None,
        };
    }

    if input.writable_shard_count > 1 {
        return PlacementDecision {
            placement: PlacementState::Worktree,
            reason: "multiple writable shards need isolated worktrees",
            block_reason: None,
        };
    }

    if input.has_overlap_risk {
        return PlacementDecision {
            placement: PlacementState::Worktree,
            reason: "overlap risk requires an isolated worktree",
            block_reason: None,
        };
    }

    if input.shard_count > 1 {
        return PlacementDecision {
            placement: PlacementState::Worktree,
            reason: "multiple shards still need isolated worktrees",
            block_reason: None,
        };
    }

    PlacementDecision {
        placement: PlacementState::MainRepo,
        reason: "single low-risk shard can stay in main repo",
        block_reason: None,
    }
}

pub fn placement_label(placement: PlacementState) -> &'static str {
    match placement {
        PlacementState::MainRepo => "main_repo",
        PlacementState::Worktree => "worktree",
        PlacementState::Blocked => "blocked",
    }
}
