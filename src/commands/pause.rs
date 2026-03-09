use crate::cli::TargetCommand;
use crate::commands::intervention_support::{run_intervention, InterventionAction};
use crate::commands::CommandOutcome;

pub fn execute(command: TargetCommand) -> CommandOutcome {
    run_intervention(InterventionAction::Pause, &command.target_id)
}
