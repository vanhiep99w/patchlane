pub struct StatusSnapshot {
    pub run: RunSnapshot,
    pub placement: PlacementSnapshot,
    pub shards: Vec<ShardSnapshot>,
    pub blockers: BlockerSummary,
    pub merge_queue: MergeQueueSummary,
    pub latest_event: EventLine,
    pub suggested_next_command: &'static str,
}

pub struct RunSnapshot {
    pub state: &'static str,
    pub objective: &'static str,
}

pub struct PlacementSnapshot {
    pub state: &'static str,
    pub reason: &'static str,
    pub block_reason: Option<&'static str>,
}

pub struct ShardSnapshot {
    pub id: &'static str,
    pub state: &'static str,
    pub branch: &'static str,
    pub owner: &'static str,
    pub blockers: &'static str,
}

pub struct BlockerSummary {
    pub headline: &'static str,
    pub items: Vec<&'static str>,
}

pub struct MergeQueueSummary {
    pub headline: &'static str,
    pub ready: Vec<&'static str>,
    pub pending: Vec<&'static str>,
}

pub struct EventLine {
    pub timestamp: &'static str,
    pub message: String,
}

pub fn fixture_status_snapshot() -> StatusSnapshot {
    StatusSnapshot {
        run: RunSnapshot {
            state: "active",
            objective: "Land compact status and watch surfaces",
        },
        placement: PlacementSnapshot {
            state: "worktree",
            reason: "multiple writable shards need isolated worktrees",
            block_reason: None,
        },
        shards: vec![
            ShardSnapshot {
                id: "01",
                state: "done",
                branch: "feat/opening-block",
                owner: "agent-a",
                blockers: "none",
            },
            ShardSnapshot {
                id: "02",
                state: "running",
                branch: "feat/status-snapshot",
                owner: "agent-b",
                blockers: "none",
            },
            ShardSnapshot {
                id: "03",
                state: "blocked",
                branch: "feat/watch-events",
                owner: "agent-c",
                blockers: "waiting on review",
            },
        ],
        blockers: BlockerSummary {
            headline: "1 active blocker",
            items: vec!["shard 03 waiting on review from maintainer"],
        },
        merge_queue: MergeQueueSummary {
            headline: "1 ready, 1 pending",
            ready: vec!["shard 01 feat/opening-block"],
            pending: vec!["shard 02 feat/status-snapshot"],
        },
        latest_event: EventLine {
            timestamp: "2026-03-09T10:18:00Z",
            message: "merge queue ready for shard 01 feat/opening-block".to_owned(),
        },
        suggested_next_command: "patchlane swarm watch",
    }
}

pub fn fixture_watch_events() -> Vec<EventLine> {
    crate::workflow::superpowers_contract::fixture_stage_event_lines()
}
