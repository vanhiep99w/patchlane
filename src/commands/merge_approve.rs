use crate::cli::MergeDecisionCommand;
use crate::commands::intervention_support::{run_merge_intervention, MergeAction};
use crate::commands::CommandOutcome;

pub fn execute(command: MergeDecisionCommand) -> CommandOutcome {
    run_merge_intervention(MergeAction::Approve, &command.merge_unit_id)
}
