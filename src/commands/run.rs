use crate::cli::RunCommand;
use crate::commands::CommandOutcome;
use crate::renderers::run_renderer::{render_opening_block, RunOpeningBlock};

pub fn execute(command: RunCommand) -> CommandOutcome {
    let Some(objective) = command.objective else {
        return CommandOutcome::stub("stub: swarm run is not implemented");
    };

    let opening = RunOpeningBlock::new(objective);

    CommandOutcome::success(render_opening_block(&opening))
}
