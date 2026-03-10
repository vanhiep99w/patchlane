use crate::cli::RunCommand;
use crate::commands::CommandOutcome;
use crate::renderers::run_renderer::{render_opening_block, RunOpeningBlock};
use crate::services::placement_engine::{decide_placement, PlacementDecisionInput, PlacementMode};

pub fn execute(command: RunCommand) -> CommandOutcome {
    let objective = command.objective;

    if objective.contains('\n') || objective.contains('\r') {
        return CommandOutcome::error("error: objective must be a single line".to_owned());
    }

    let placement = decide_placement(PlacementDecisionInput {
        mode: PlacementMode::Balanced,
        shard_count: 1,
        writable_shard_count: 1,
        has_overlap_risk: false,
        repo_is_dirty: false,
        blocked_reason: None,
    });
    let opening = RunOpeningBlock::new(objective, placement);

    CommandOutcome::success(render_opening_block(&opening))
}
