pub mod cli;
pub mod commands;
pub mod domain;

pub fn bootstrap_banner() -> &'static str {
    "Patchlane CLI bootstrap"
}
