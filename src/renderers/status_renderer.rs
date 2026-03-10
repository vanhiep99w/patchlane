use crate::events::run_events::StatusSnapshot;

pub fn render_status_snapshot(snapshot: &StatusSnapshot) -> String {
    let mut lines = vec![
        "Run".to_owned(),
        format!("  {}", snapshot.run.state),
        format!("  objective: {}", snapshot.run.objective),
        String::new(),
        "Placement".to_owned(),
        format!(
            "  {}: {}",
            snapshot.placement.state, snapshot.placement.reason
        ),
    ];

    if let Some(block_reason) = snapshot.placement.block_reason {
        lines.push(format!("  block reason: {}", block_reason));
    }

    lines.extend([
        String::new(),
        "Shards".to_owned(),
        "  shard  state        branch                      owner    blockers".to_owned(),
    ]);

    for shard in &snapshot.shards {
        lines.push(format!(
            "  {:<2}     {:<12} {:<27} {:<8} {}",
            shard.id, shard.state, shard.branch, shard.owner, shard.blockers
        ));
    }

    lines.extend([
        String::new(),
        "Blockers".to_owned(),
        format!("  {}", snapshot.blockers.headline),
    ]);

    for blocker in &snapshot.blockers.items {
        lines.push(format!("  - {}", blocker));
    }

    lines.extend([
        String::new(),
        "Merge Queue".to_owned(),
        format!("  {}", snapshot.merge_queue.headline),
    ]);

    for ready in &snapshot.merge_queue.ready {
        lines.push(format!("  - ready: {}", ready));
    }

    for pending in &snapshot.merge_queue.pending {
        lines.push(format!("  - pending: {}", pending));
    }

    lines.extend([
        String::new(),
        "Latest Event".to_owned(),
        format!(
            "  {} {}",
            snapshot.latest_event.timestamp, snapshot.latest_event.message
        ),
        String::new(),
        "Next".to_owned(),
        format!("  {}", snapshot.suggested_next_command),
    ]);

    lines.join("\n")
}
