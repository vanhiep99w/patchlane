use crate::orchestration::model::TaskSnapshot;
use crate::orchestration::store::load_task_snapshot;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub fn load_runs(state_root: &Path) -> io::Result<Vec<TaskSnapshot>> {
    let tasks_root = state_root.join("tasks");
    let entries = match fs::read_dir(&tasks_root) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };

    let mut snapshots = entries
        .collect::<io::Result<Vec<_>>>()?
        .into_iter()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .map(|path| {
            load_task_snapshot(&path).map_err(|error| {
                io::Error::new(
                    error.kind(),
                    format!("failed to load task run {}: {error}", path.display()),
                )
            })
        })
        .collect::<io::Result<Vec<_>>>()?;

    snapshots.sort_by(|left, right| right.run.updated_at.cmp(&left.run.updated_at));
    Ok(snapshots)
}

pub fn resolve_log_path(state_root: &Path, run_id: &str, relative_path: &str) -> PathBuf {
    let file_name = Path::new(relative_path)
        .file_name()
        .map(|name| name.to_owned())
        .unwrap_or_else(|| "unknown.log".into());
    state_root
        .join("tasks")
        .join(run_id)
        .join("logs")
        .join(file_name)
}
