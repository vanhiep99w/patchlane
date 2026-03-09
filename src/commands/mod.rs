mod run;
mod status;
mod watch;

use crate::cli::{Cli, MergeCommand, SwarmCommand, TopLevelCommand};

pub struct CommandOutcome {
    pub message: String,
    pub exit_code: u8,
    pub stream: OutputStream,
}

pub enum OutputStream {
    Stdout,
    Stderr,
}

pub fn execute(cli: Cli) -> CommandOutcome {
    match cli.command {
        TopLevelCommand::Swarm(swarm) => match swarm.command {
            SwarmCommand::Run(run) => run::execute(run),
            SwarmCommand::Status => status::execute(),
            SwarmCommand::Watch => watch::execute(),
            SwarmCommand::Pause => unimplemented_stub("stub: swarm pause is not implemented"),
            SwarmCommand::Resume => unimplemented_stub("stub: swarm resume is not implemented"),
            SwarmCommand::Retry => unimplemented_stub("stub: swarm retry is not implemented"),
            SwarmCommand::Reassign => unimplemented_stub("stub: swarm reassign is not implemented"),
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

impl CommandOutcome {
    pub fn success(message: String) -> Self {
        Self {
            message,
            exit_code: 0,
            stream: OutputStream::Stdout,
        }
    }

    pub fn stub(message: &'static str) -> Self {
        Self {
            message: message.to_owned(),
            exit_code: 1,
            stream: OutputStream::Stderr,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            message,
            exit_code: 1,
            stream: OutputStream::Stderr,
        }
    }
}

fn unimplemented_stub(message: &'static str) -> CommandOutcome {
    CommandOutcome::stub(message)
}
