use crate::cli::RunCommand;
use crate::commands::CommandOutcome;
use crate::renderers::run_renderer::{render_opening_block, RunOpeningBlock};

pub fn execute(command: RunCommand) -> CommandOutcome {
    let objective = command.objective;

    if objective.contains('\n') || objective.contains('\r') {
        return CommandOutcome::error("error: objective must be a single line".to_owned());
    }

    let opening = RunOpeningBlock::new(objective);

    CommandOutcome::success(render_opening_block(&opening))
}
