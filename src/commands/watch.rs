use crate::commands::CommandOutcome;
use crate::events::run_events::{derive_watch_events, empty_watch_events};
use crate::renderers::watch_renderer::render_watch_events;
use crate::store::run_store::{latest_run_dir, load_events};
use std::io;
use std::path::PathBuf;

pub fn execute() -> CommandOutcome {
    let state_root = state_root();
    let events = match latest_run_dir(&state_root) {
        Ok(run_dir) => match load_events(&run_dir) {
            Ok(events) => derive_watch_events(events),
            Err(error) if error.kind() == io::ErrorKind::NotFound => empty_watch_events(),
            Err(error) => return CommandOutcome::error(format!("error: failed to load events: {error}")),
        },
        Err(error) if error.kind() == io::ErrorKind::NotFound => empty_watch_events(),
        Err(error) => return CommandOutcome::error(format!("error: {error}")),
    };
    CommandOutcome::success(render_watch_events(&events))
}

fn state_root() -> PathBuf {
    std::env::var_os("PATCHLANE_STATE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".patchlane"))
}
