use clap::Parser;
use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    if env::args_os().nth(1).is_none() {
        eprintln!(
            "{}\n\nPlanned swarm commands are not implemented yet.\nUse --help to inspect the current bootstrap surface.",
            patchlane::bootstrap_banner()
        );
        return ExitCode::from(1);
    }

    let cli = patchlane::cli::Cli::parse();
    println!("{}", patchlane::commands::execute(cli));
    ExitCode::SUCCESS
}
