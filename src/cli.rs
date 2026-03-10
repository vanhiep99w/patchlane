use clap::{Parser, Subcommand, ValueEnum};

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
    AgentEvent(AgentEventCommand),
    Task(TaskCommand),
    Tui,
    Swarm(SwarmCommandGroup),
}

#[derive(Debug, Parser)]
pub struct AgentEventCommand {
    #[arg(value_name = "EVENT_TYPE")]
    pub event_type: String,
    #[arg(long, value_name = "RUN_DIR")]
    pub run_dir: String,
    #[arg(long, value_name = "RUN_ID")]
    pub run_id: String,
    #[arg(long, value_name = "AGENT_ID")]
    pub agent_id: String,
    #[arg(long, value_name = "MESSAGE")]
    pub message: String,
}

#[derive(Debug, Parser)]
pub struct TaskCommand {
    #[arg(long, value_name = "RUNTIME")]
    pub runtime: Option<Runtime>,
    #[arg(value_name = "OBJECTIVE")]
    pub objective: String,
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
    Pause(TargetCommand),
    Resume(TargetCommand),
    Retry(ShardCommand),
    Reassign(ReassignCommand),
    Merge(MergeCommandGroup),
    Stop(RunCommandTarget),
    Board,
    Web,
}

#[derive(Debug, Parser)]
pub struct RunCommand {
    #[arg(long, value_name = "RUNTIME")]
    pub runtime: Runtime,
    #[arg(value_name = "OBJECTIVE")]
    pub objective: String,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum Runtime {
    Codex,
    Claude,
}

#[derive(Debug, Parser)]
pub struct TargetCommand {
    #[arg(value_name = "TARGET_ID")]
    pub target_id: String,
}

#[derive(Debug, Parser)]
pub struct ShardCommand {
    #[arg(value_name = "SHARD_ID")]
    pub shard_id: String,
}

#[derive(Debug, Parser)]
pub struct RunCommandTarget {
    #[arg(value_name = "RUN_ID")]
    pub run_id: String,
}

#[derive(Debug, Parser)]
pub struct ReassignCommand {
    #[arg(value_name = "SHARD_ID")]
    pub shard_id: String,
    #[arg(long, value_name = "RUNTIME")]
    pub runtime: String,
}

#[derive(Debug, Parser)]
#[command(subcommand_required = true, arg_required_else_help = true)]
pub struct MergeCommandGroup {
    #[command(subcommand)]
    pub command: MergeCommand,
}

#[derive(Debug, Subcommand)]
pub enum MergeCommand {
    Approve(MergeDecisionCommand),
    Reject(MergeDecisionCommand),
}

#[derive(Debug, Parser)]
pub struct MergeDecisionCommand {
    #[arg(value_name = "MERGE_UNIT_ID")]
    pub merge_unit_id: String,
}
