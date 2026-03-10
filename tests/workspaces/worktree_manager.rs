use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use patchlane::workspaces::worktree_manager::{
    allocate_workspace, allocate_workspace_for_subtask, SubtaskRisk, WorkspaceError,
    WorkspacePolicy,
};

fn temp_root() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("patchlane-workspaces-{unique}"));
    fs::create_dir_all(&root).expect("temp root should be creatable");
    root
}

#[test]
fn worktree_manager_allocates_a_stable_workspace_path_per_shard() {
    let root = temp_root();

    let first = allocate_workspace(&root, "run-001", "01").expect("workspace should allocate");
    let second = allocate_workspace(&root, "run-001", "01").expect("workspace should allocate");

    assert_eq!(first, second, "workspace path should be stable for the same shard");
    assert!(
        first.ends_with("run-001/workspaces/shard-01"),
        "workspace path should include run and shard identity: {}",
        first.display()
    );

    fs::remove_dir_all(root).expect("temp root should be removable");
}

#[test]
fn worktree_manager_creates_workspace_directories_inside_state_root() {
    let root = temp_root();

    let workspace = allocate_workspace(&root, "run-002", "02").expect("workspace should allocate");

    assert!(workspace.is_dir(), "workspace directory should exist");
    assert!(
        workspace.starts_with(&root),
        "workspace should stay inside the Patchlane-managed state root"
    );

    fs::remove_dir_all(root).expect("temp root should be removable");
}

#[test]
fn worktree_manager_surfaces_filesystem_errors_clearly() {
    let root = temp_root();
    let file_path = root.join("occupied");
    fs::write(&file_path, "not a directory").expect("file path should be creatable");

    let error = allocate_workspace(&file_path, "run-003", "03")
        .expect_err("allocating inside a file path should fail");

    match error {
        WorkspaceError::CreateFailed {
            run_id,
            shard_id,
            path,
            source: _,
        } => {
            assert_eq!(run_id, "run-003");
            assert_eq!(shard_id, "03");
            assert!(path.ends_with("run-003/workspaces/shard-03"));
        }
    }

    fs::remove_dir_all(root).expect("temp root should be removable");
}

#[test]
fn worktree_manager_uses_session_root_for_low_risk_subtasks() {
    let root = temp_root();

    let allocation = allocate_workspace_for_subtask(
        &root,
        "run-004",
        "brainstorm",
        WorkspacePolicy::isolated_by_default(),
        SubtaskRisk::Low,
    )
    .expect("workspace should allocate");

    assert!(
        allocation.path.ends_with("run-004/session-root"),
        "low-risk subtask should stay in session root: {}",
        allocation.path.display()
    );
    assert!(!allocation.isolated, "low-risk work should not be isolated");

    fs::remove_dir_all(root).expect("temp root should be removable");
}

#[test]
fn worktree_manager_uses_isolated_workspace_for_risky_subtasks() {
    let root = temp_root();

    let allocation = allocate_workspace_for_subtask(
        &root,
        "run-005",
        "implementer",
        WorkspacePolicy::isolated_by_default(),
        SubtaskRisk::High,
    )
    .expect("workspace should allocate");

    assert!(
        allocation.path.ends_with("run-005/workspaces/shard-implementer"),
        "risky subtask should get isolated workspace: {}",
        allocation.path.display()
    );
    assert!(allocation.isolated, "high-risk work should be isolated");

    fs::remove_dir_all(root).expect("temp root should be removable");
}
