pub mod cli;
pub mod commands;
pub mod domain;
pub mod events;
pub mod renderers;
pub mod services;
pub mod store;
pub mod workflow;

pub fn bootstrap_banner() -> &'static str {
    "Patchlane CLI bootstrap"
}
