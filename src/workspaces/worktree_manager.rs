use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum WorkspaceError {
    CreateFailed {
        run_id: String,
        shard_id: String,
        path: PathBuf,
        source: io::Error,
    },
}

pub fn allocate_workspace(
    state_root: &Path,
    run_id: &str,
    shard_id: &str,
) -> Result<PathBuf, WorkspaceError> {
    let workspace = state_root
        .join(run_id)
        .join("workspaces")
        .join(format!("shard-{shard_id}"));

    fs::create_dir_all(&workspace).map_err(|source| WorkspaceError::CreateFailed {
        run_id: run_id.to_owned(),
        shard_id: shard_id.to_owned(),
        path: workspace.clone(),
        source,
    })?;

    Ok(workspace)
}
