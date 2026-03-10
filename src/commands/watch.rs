use crate::commands::CommandOutcome;
use crate::events::run_events::{derive_legacy_watch_events, derive_watch_events, empty_watch_events};
use crate::orchestration::store::load_task_snapshot;
use crate::renderers::watch_renderer::render_watch_events;
use crate::store::run_store::load_events;
use std::fs;
use std::io;
use std::path::PathBuf;

pub fn execute() -> CommandOutcome {
    let state_root = state_root();
    let task_root = state_root.join("tasks");
    let run_dir = match latest_task_run_dir(&task_root) {
        Ok(path) => path,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return legacy_watch(&state_root),
        Err(error) => return CommandOutcome::error(format!("error: {error}")),
    };

    let snapshot = match load_task_snapshot(&run_dir) {
        Ok(snapshot) => snapshot,
        Err(error) => {
            return CommandOutcome::error(format!(
                "error: failed to load task run snapshot: {error}"
            ));
        }
    };

    CommandOutcome::success(render_watch_events(&derive_watch_events(snapshot.events)))
}

fn legacy_watch(state_root: &PathBuf) -> CommandOutcome {
    let events = match latest_legacy_run_dir(state_root) {
        Ok(run_dir) => match load_events(&run_dir) {
            Ok(events) => derive_legacy_watch_events(events),
            Err(error) if error.kind() == io::ErrorKind::NotFound => empty_watch_events(),
            Err(error) => return CommandOutcome::error(format!("error: failed to load events: {error}")),
        },
        Err(error) if error.kind() == io::ErrorKind::NotFound => empty_watch_events(),
        Err(error) => return CommandOutcome::error(format!("error: {error}")),
    };

    CommandOutcome::success(render_watch_events(&events))
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
