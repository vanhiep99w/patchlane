use crate::cli::Runtime;
use crate::services::placement_engine::{placement_label, PlacementDecision};

pub struct RunOpeningBlock {
    run_id: String,
    runtime: Runtime,
    objective: String,
    shard_count: usize,
    placement: PlacementDecision,
    failed_count: usize,
    next_step: String,
}

impl RunOpeningBlock {
    pub fn new(
        run_id: String,
        runtime: Runtime,
        objective: String,
        shard_count: usize,
        placement: PlacementDecision,
        failed_count: usize,
        next_step: String,
    ) -> Self {
        Self {
            run_id,
            runtime,
            objective,
            shard_count,
            placement,
            failed_count,
            next_step,
        }
    }
}

pub fn render_opening_block(opening: &RunOpeningBlock) -> String {
    let mut placement_lines = vec![
        format!(
            "  {placement}: {reason}",
            placement = placement_label(opening.placement.placement),
            reason = opening.placement.reason,
        ),
        "  final placement pending runtime preflight inputs".to_owned(),
    ];

    if let Some(block_reason) = &opening.placement.block_reason {
        placement_lines.push(format!("  block reason: {}", block_reason));
    }

    format!(
        "Run\n  queued\n  run_id: {run_id}\n  runtime: {runtime}\n\nObjective\n  {objective}\n\nPlan\n  shards: {shard_count}\n  failed: {failed_count}\n\nPlacement\n{placement}\n\nNext\n  {next_step}",
        run_id = opening.run_id,
        runtime = runtime_label(&opening.runtime),
        objective = opening.objective,
        shard_count = opening.shard_count,
        failed_count = opening.failed_count,
        placement = placement_lines.join("\n"),
        next_step = opening.next_step,
    )
}

fn runtime_label(runtime: &Runtime) -> &'static str {
    match runtime {
        Runtime::Codex => "codex",
        Runtime::Claude => "claude",
    }
}

#[cfg(test)]
mod tests {
    use super::{render_opening_block, RunOpeningBlock};
    use crate::cli::Runtime;
    use crate::domain::placement::PlacementState;
    use crate::services::placement_engine::PlacementDecision;

    #[test]
    fn run_opening_block_marks_placement_as_simulated() {
        let opening = RunOpeningBlock::new(
            "run-001".to_owned(),
            Runtime::Codex,
            "demo objective".to_owned(),
            4,
            PlacementDecision {
                placement: PlacementState::MainRepo,
                reason: "single low-risk shard can stay in main repo",
                block_reason: None,
            },
            0,
            "launching 4 local codex workers".to_owned(),
        );

        let rendered = render_opening_block(&opening);

        assert!(rendered.contains("run_id: run-001"));
        assert!(rendered.contains("runtime: codex"));
        assert!(rendered.contains("shards: 4"));
        assert!(rendered.contains("failed: 0"));
        assert!(rendered.contains("main_repo: single low-risk shard can stay in main repo"));
    }

    #[test]
    fn run_opening_block_preserves_block_reason_visibility() {
        let opening = RunOpeningBlock::new(
            "run-002".to_owned(),
            Runtime::Claude,
            "demo objective".to_owned(),
            4,
            PlacementDecision {
                placement: PlacementState::Blocked,
                reason: "dispatch blocked until operator intervention",
                block_reason: Some("operator approval required".to_owned()),
            },
            2,
            "spawn failure recorded for failed: 2".to_owned(),
        );

        let rendered = render_opening_block(&opening);

        assert!(rendered.contains("runtime: claude"));
        assert!(rendered.contains("failed: 2"));
        assert!(rendered.contains("operator approval required"));
    }
}
