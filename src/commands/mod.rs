mod intervention_support;
mod merge_approve;
mod merge_reject;
mod pause;
mod reassign;
mod resume;
mod run;
mod status;
mod stop;
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
            SwarmCommand::Pause(command) => pause::execute(command),
            SwarmCommand::Resume(command) => resume::execute(command),
            SwarmCommand::Retry(command) => reassign::execute_retry(command),
            SwarmCommand::Reassign(command) => reassign::execute_reassign(command),
            SwarmCommand::Merge(merge) => match merge.command {
                MergeCommand::Approve(command) => merge_approve::execute(command),
                MergeCommand::Reject(command) => merge_reject::execute(command),
            },
            SwarmCommand::Stop(command) => stop::execute(command),
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
