use crate::cli::{ReassignCommand, ShardCommand};
use crate::commands::intervention_support::{run_reassign_intervention, run_retry_intervention};
use crate::commands::CommandOutcome;

pub fn execute_retry(command: ShardCommand) -> CommandOutcome {
    run_retry_intervention(&command.shard_id)
}

pub fn execute_reassign(command: ReassignCommand) -> CommandOutcome {
    run_reassign_intervention(&command.shard_id, &command.runtime)
}
