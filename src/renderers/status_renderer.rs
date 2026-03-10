use crate::events::run_events::{StatusSnapshot, StatusView};

pub fn render_status_snapshot(snapshot: &StatusSnapshot) -> String {
    let mut lines = vec![
        "Run".to_owned(),
        format!("  {} ({})", snapshot.run.id, snapshot.run.state),
        format!("  runtime: {}", snapshot.run.runtime),
    ];

    if let Some(phase) = &snapshot.run.phase {
        lines.push(format!("  phase: {phase}"));
    }

    lines.push(format!("  objective: {}", snapshot.run.objective));

    match snapshot.view {
        StatusView::Agents => {
            lines.extend([
                String::new(),
                "Agents".to_owned(),
                "  id                 role             phase                state                runtime  detail"
                    .to_owned(),
            ]);

            for agent in &snapshot.agents {
                lines.push(format!(
                    "  {:<18} {:<16} {:<20} {:<20} {:<7} {}",
                    agent.id, agent.role, agent.phase, agent.state, agent.runtime, agent.detail
                ));
            }
        }
        StatusView::Shards => {
            lines.extend([
                String::new(),
                "Shards".to_owned(),
                "  shard  runtime  pid    state      workspace  detail".to_owned(),
            ]);

            for shard in &snapshot.shards {
                lines.push(format!(
                    "  {:<2}     {:<7} {:<6} {:<10} {}  {}",
                    shard.id, shard.runtime, shard.pid, shard.state, shard.workspace, shard.detail
                ));
            }
        }
    }

    lines.extend([String::new(), "Blockers".to_owned()]);
    if snapshot.blockers.is_empty() {
        lines.push("  none".to_owned());
    } else {
        for blocker in &snapshot.blockers {
            lines.push(format!("  - {blocker}"));
        }
    }

    lines.extend([
        String::new(),
        "Latest Event".to_owned(),
        format!("  {} {}", snapshot.latest_event.timestamp, snapshot.latest_event.message),
        String::new(),
        "Next".to_owned(),
        format!("  {}", snapshot.suggested_next_command),
    ]);

    lines.join("\n")
}
