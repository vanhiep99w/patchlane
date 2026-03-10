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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubtaskRisk {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkspacePolicy {
    pub default_isolation: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceAllocation {
    pub path: PathBuf,
    pub isolated: bool,
}

impl WorkspacePolicy {
    pub fn isolated_by_default() -> Self {
        Self {
            default_isolation: true,
        }
    }
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

pub fn allocate_workspace_for_subtask(
    state_root: &Path,
    run_id: &str,
    subtask_id: &str,
    policy: WorkspacePolicy,
    risk: SubtaskRisk,
) -> Result<WorkspaceAllocation, WorkspaceError> {
    if policy.default_isolation && matches!(risk, SubtaskRisk::Medium | SubtaskRisk::High) {
        return allocate_workspace(state_root, run_id, subtask_id).map(|path| WorkspaceAllocation {
            path,
            isolated: true,
        });
    }

    let session_root = state_root.join(run_id).join("session-root");
    fs::create_dir_all(&session_root).map_err(|source| WorkspaceError::CreateFailed {
        run_id: run_id.to_owned(),
        shard_id: subtask_id.to_owned(),
        path: session_root.clone(),
        source,
    })?;

    Ok(WorkspaceAllocation {
        path: session_root,
        isolated: false,
    })
}
