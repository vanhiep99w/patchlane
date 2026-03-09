use crate::cli::{Cli, MergeCommand, SwarmCommand, TopLevelCommand};

pub fn execute(cli: Cli) -> &'static str {
    match cli.command {
        TopLevelCommand::Swarm(swarm) => match swarm.command {
            SwarmCommand::Run => "stub: swarm run",
            SwarmCommand::Status => "stub: swarm status",
            SwarmCommand::Watch => "stub: swarm watch",
            SwarmCommand::Pause => "stub: swarm pause",
            SwarmCommand::Resume => "stub: swarm resume",
            SwarmCommand::Retry => "stub: swarm retry",
            SwarmCommand::Reassign => "stub: swarm reassign",
            SwarmCommand::Merge(merge) => match merge.command {
                MergeCommand::Approve => "stub: swarm merge approve",
                MergeCommand::Reject => "stub: swarm merge reject",
            },
            SwarmCommand::Stop => "stub: swarm stop",
            SwarmCommand::Board => "stub: swarm board",
            SwarmCommand::Web => "stub: swarm web",
        },
    }
}
