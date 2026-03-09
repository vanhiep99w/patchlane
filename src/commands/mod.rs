use crate::cli::{Cli, MergeCommand, SwarmCommand, TopLevelCommand};

pub struct CommandOutcome {
    pub message: &'static str,
    pub exit_code: u8,
}

pub fn execute(cli: Cli) -> CommandOutcome {
    match cli.command {
        TopLevelCommand::Swarm(swarm) => match swarm.command {
            SwarmCommand::Run => unimplemented_stub("stub: swarm run is not implemented"),
            SwarmCommand::Status => unimplemented_stub("stub: swarm status is not implemented"),
            SwarmCommand::Watch => unimplemented_stub("stub: swarm watch is not implemented"),
            SwarmCommand::Pause => unimplemented_stub("stub: swarm pause is not implemented"),
            SwarmCommand::Resume => unimplemented_stub("stub: swarm resume is not implemented"),
            SwarmCommand::Retry => unimplemented_stub("stub: swarm retry is not implemented"),
            SwarmCommand::Reassign => {
                unimplemented_stub("stub: swarm reassign is not implemented")
            }
            SwarmCommand::Merge(merge) => match merge.command {
                MergeCommand::Approve => {
                    unimplemented_stub("stub: swarm merge approve is not implemented")
                }
                MergeCommand::Reject => {
                    unimplemented_stub("stub: swarm merge reject is not implemented")
                }
            },
            SwarmCommand::Stop => unimplemented_stub("stub: swarm stop is not implemented"),
            SwarmCommand::Board => unimplemented_stub("stub: swarm board is not implemented"),
            SwarmCommand::Web => unimplemented_stub("stub: swarm web is not implemented"),
        },
    }
}

fn unimplemented_stub(message: &'static str) -> CommandOutcome {
    CommandOutcome {
        message,
        exit_code: 1,
    }
}
