use crate::cli::RunCommandTarget;
use crate::commands::intervention_support::{run_intervention, InterventionAction};
use crate::commands::CommandOutcome;

pub fn execute(command: RunCommandTarget) -> CommandOutcome {
    run_intervention(InterventionAction::Stop, &command.run_id)
}
