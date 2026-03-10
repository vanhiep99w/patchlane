use crate::cli::Runtime;
use crate::services::placement_engine::{placement_label, PlacementDecision};

pub struct RunOpeningBlock {
    runtime: Runtime,
    objective: String,
    placement: PlacementDecision,
}

impl RunOpeningBlock {
    pub fn new(runtime: Runtime, objective: String, placement: PlacementDecision) -> Self {
        Self {
            runtime,
            objective,
            placement,
        }
    }
}

pub fn render_opening_block(opening: &RunOpeningBlock) -> String {
    let mut placement_lines = vec![
        format!(
            "  simulated placeholder preflight -> {placement}: {reason}",
            placement = placement_label(opening.placement.placement),
            reason = opening.placement.reason,
        ),
        "  final placement pending runtime preflight inputs".to_owned(),
    ];

    if let Some(block_reason) = &opening.placement.block_reason {
        placement_lines.push(format!("  block reason: {}", block_reason));
    }

    format!(
        "Run\n  queued\n  runtime: {runtime}\n\nObjective\n  {objective}\n\nPlan\n  1. Capture the requested objective.\n  2. Prepare a placeholder execution plan.\n\nPlacement\n{placement}\n\nNext\n  waiting for planner and runtime integration",
        runtime = runtime_label(&opening.runtime),
        objective = opening.objective,
        placement = placement_lines.join("\n"),
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
            Runtime::Codex,
            "demo objective".to_owned(),
            PlacementDecision {
                placement: PlacementState::MainRepo,
                reason: "single low-risk shard can stay in main repo",
                block_reason: None,
            },
        );

        let rendered = render_opening_block(&opening);

        assert!(rendered.contains(
            "simulated placeholder preflight -> main_repo: single low-risk shard can stay in main repo"
        ));
        assert!(rendered.contains("runtime: codex"));
        assert!(rendered.contains("final placement pending runtime preflight inputs"));
    }

    #[test]
    fn run_opening_block_preserves_block_reason_visibility() {
        let opening = RunOpeningBlock::new(
            Runtime::Claude,
            "demo objective".to_owned(),
            PlacementDecision {
                placement: PlacementState::Blocked,
                reason: "dispatch blocked until operator intervention",
                block_reason: Some("operator approval required".to_owned()),
            },
        );

        let rendered = render_opening_block(&opening);

        assert!(rendered.contains(
            "simulated placeholder preflight -> blocked: dispatch blocked until operator intervention"
        ));
        assert!(rendered.contains("runtime: claude"));
        assert!(rendered.contains("operator approval required"));
    }
}
