use clap::Parser;
use clap::error::ErrorKind;
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = match patchlane::cli::Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) => {
            let exit_code = match error.kind() {
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => 0,
                _ => 2,
            };
            error.print().expect("clap errors should be printable");
            return ExitCode::from(exit_code);
        }
    };

    let outcome = patchlane::commands::execute(cli);
    eprintln!("{}", outcome.message);
    ExitCode::from(outcome.exit_code)
}
