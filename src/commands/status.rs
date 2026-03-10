use crate::commands::CommandOutcome;
use crate::events::run_events::{
    derive_legacy_status_snapshot, derive_status_snapshot, empty_status_snapshot,
};
use crate::orchestration::store::load_task_snapshot;
use crate::renderers::status_renderer::render_status_snapshot;
use crate::store::run_store::{load_events, load_run, load_shards};
use std::fs;
use std::io;
use std::path::PathBuf;

pub fn execute() -> CommandOutcome {
    let state_root = state_root();
    let task_root = state_root.join("tasks");
    let task_run_dir = match latest_task_run_dir(&task_root) {
        Ok(path) => path,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return legacy_status(&state_root),
        Err(error) => return CommandOutcome::error(format!("error: {error}")),
    };

    let snapshot = match load_task_snapshot(&task_run_dir) {
        Ok(snapshot) => snapshot,
        Err(error) => {
            return CommandOutcome::error(format!(
                "error: failed to load task run snapshot: {error}"
            ));
        }
    };

    CommandOutcome::success(render_status_snapshot(&derive_status_snapshot(snapshot)))
}

fn legacy_status(state_root: &PathBuf) -> CommandOutcome {
    let run_dir = match latest_legacy_run_dir(state_root) {
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
        Err(error) => return CommandOutcome::error(format!("error: failed to load shards: {error}")),
    };
    let events = match load_events(&run_dir) {
        Ok(events) => events,
        Err(error) => return CommandOutcome::error(format!("error: failed to load events: {error}")),
    };

    CommandOutcome::success(render_status_snapshot(&derive_legacy_status_snapshot(
        run, shards, events,
    )))
}

fn latest_legacy_run_dir(root: &PathBuf) -> io::Result<PathBuf> {
    let mut run_dirs = fs::read_dir(root)?
        .collect::<io::Result<Vec<_>>>()?
        .into_iter()
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_dir()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name != "tasks")
        })
        .collect::<Vec<_>>();
    run_dirs.sort();
    run_dirs
        .pop()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no persisted legacy runs found"))
}

fn latest_task_run_dir(root: &PathBuf) -> io::Result<PathBuf> {
    let latest = fs::read_dir(root)?
        .collect::<io::Result<Vec<_>>>()?
        .into_iter()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .filter_map(|path| {
            load_task_snapshot(&path)
                .ok()
                .map(|snapshot| (snapshot.run.updated_at, path))
        })
        .into_iter()
        .max_by(|left, right| left.0.cmp(&right.0))
        .map(|(_, path)| path);

    latest.ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no persisted task runs found"))
}

fn state_root() -> PathBuf {
    std::env::var_os("PATCHLANE_STATE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".patchlane"))
}
