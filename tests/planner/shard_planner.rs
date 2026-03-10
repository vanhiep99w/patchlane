use patchlane::planner::shard_planner::{plan_shards, PlannedShard};

fn assert_brief_contains_context(shards: &[PlannedShard], objective: &str) {
    for shard in shards {
        assert!(
            shard.brief.contains(objective),
            "shard brief should preserve objective context: {:?}",
            shard
        );
    }
}

#[test]
fn shard_planner_produces_multiple_deterministic_shards() {
    let objective = "Launch multiple workers for a single objective";

    let shards = plan_shards(objective);

    assert!(
        (2..=4).contains(&shards.len()),
        "planner should emit 2-4 shards, got {}",
        shards.len()
    );
    assert_eq!(shards.len(), 4, "planner should keep a stable shard count");
    assert_eq!(shards[0].id, "01");
    assert_eq!(shards[1].id, "02");
    assert_eq!(shards[2].id, "03");
    assert_eq!(shards[3].id, "04");
    assert_brief_contains_context(&shards, objective);
}

#[test]
fn shard_planner_is_stable_for_the_same_objective() {
    let objective = "Make Patchlane launch codex workers";

    let left = plan_shards(objective);
    let right = plan_shards(objective);

    assert_eq!(left, right, "planner output should be deterministic");
}

#[test]
fn shard_planner_emits_operator_readable_work_packets() {
    let objective = "Ship a local runtime orchestrator";

    let shards = plan_shards(objective);
    let packet_labels = shards.iter().map(|shard| shard.label).collect::<Vec<_>>();

    assert_eq!(
        packet_labels,
        vec!["analyze", "implement", "verify", "integrate"],
        "planner should emit a fixed set of work packets"
    );
}
