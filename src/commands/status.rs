use crate::commands::CommandOutcome;
use crate::events::run_events::{derive_status_snapshot, empty_status_snapshot};
use crate::renderers::status_renderer::render_status_snapshot;
use crate::store::run_store::{latest_run_dir, load_events, load_run, load_shards};
use std::io;
use std::path::PathBuf;

pub fn execute() -> CommandOutcome {
    let state_root = state_root();
    let run_dir = match latest_run_dir(&state_root) {
        Ok(path) => path,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return CommandOutcome::success(render_status_snapshot(&empty_status_snapshot()));
        }
        Err(error) => return CommandOutcome::error(format!("error: {error}")),
    };
    let run = match load_run(&run_dir) {
        Ok(run) => run,
        Err(error) => return CommandOutcome::error(format!("error: failed to load run: {error}")),
    };
    let shards = match load_shards(&run_dir) {
        Ok(shards) => shards,
        Err(error) => {
            return CommandOutcome::error(format!("error: failed to load shards: {error}"));
        }
    };
    let events = match load_events(&run_dir) {
        Ok(events) => events,
        Err(error) => {
            return CommandOutcome::error(format!("error: failed to load events: {error}"));
        }
    };
    let snapshot = derive_status_snapshot(run, shards, events);
    CommandOutcome::success(render_status_snapshot(&snapshot))
}

fn state_root() -> PathBuf {
    std::env::var_os("PATCHLANE_STATE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".patchlane"))
}
