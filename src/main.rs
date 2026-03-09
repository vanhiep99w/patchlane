use clap::Parser;
use std::env;
use std::process::ExitCode;

#[derive(Debug, Parser)]
#[command(name = "patchlane")]
#[command(about = "Patchlane CLI bootstrap")]
#[command(
    long_about = "Patchlane CLI bootstrap\n\nPlanned swarm commands are not implemented yet."
)]
struct Cli;

fn main() -> ExitCode {
    if env::args_os().nth(1).is_none() {
        eprintln!(
            "{}\n\nPlanned swarm commands are not implemented yet.\nUse --help to inspect the current bootstrap surface.",
            patchlane::bootstrap_banner()
        );
        return ExitCode::from(1);
    }

    let _ = Cli::parse();
    println!("{}", patchlane::bootstrap_banner());
    ExitCode::SUCCESS
}
