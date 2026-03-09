use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "patchlane")]
#[command(about = "Patchlane CLI bootstrap")]
#[command(
    long_about = "Patchlane CLI bootstrap\n\nPlanned swarm commands are not implemented yet."
)]
#[command(subcommand_required = true, arg_required_else_help = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: TopLevelCommand,
}

#[derive(Debug, Subcommand)]
pub enum TopLevelCommand {
    Swarm(SwarmCommandGroup),
}

#[derive(Debug, Parser)]
#[command(subcommand_required = true, arg_required_else_help = true)]
pub struct SwarmCommandGroup {
    #[command(subcommand)]
    pub command: SwarmCommand,
}

#[derive(Debug, Subcommand)]
pub enum SwarmCommand {
    Run(RunCommand),
    Status,
    Watch,
    Pause,
    Resume,
    Retry,
    Reassign,
    Merge(MergeCommandGroup),
    Stop,
    Board,
    Web,
}

#[derive(Debug, Parser)]
pub struct RunCommand {
    pub objective: Option<String>,
}

#[derive(Debug, Parser)]
#[command(subcommand_required = true, arg_required_else_help = true)]
pub struct MergeCommandGroup {
    #[command(subcommand)]
    pub command: MergeCommand,
}

#[derive(Debug, Subcommand)]
pub enum MergeCommand {
    Approve,
    Reject,
}
