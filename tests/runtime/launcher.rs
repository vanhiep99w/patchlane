use patchlane::cli::Runtime;
use patchlane::runtime::launcher::{
    build_launch_spec, launch_worker, LaunchRequest, RuntimeLaunchError,
};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_root() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("patchlane-launcher-{unique}"));
    fs::create_dir_all(&root).expect("temp root should be creatable");
    root
}

fn request(runtime: Runtime, root: &PathBuf) -> LaunchRequest {
    LaunchRequest {
        runtime,
        shard_id: "01".to_owned(),
        brief: "Implement the main shard".to_owned(),
        workspace: root.join("workspace-01"),
        logs_dir: root.join("logs"),
    }
}

#[test]
fn launcher_builds_the_expected_codex_invocation() {
    let root = temp_root();
    let spec = build_launch_spec(&request(Runtime::Codex, &root));

    assert_eq!(spec.program, "codex");
    assert_eq!(
        spec.args,
        vec!["exec", "--skip-git-repo-check", "Implement the main shard"]
    );

    fs::remove_dir_all(root).expect("temp root should be removable");
}

#[test]
fn launcher_builds_the_expected_claude_invocation() {
    let root = temp_root();
    let spec = build_launch_spec(&request(Runtime::Claude, &root));

    assert_eq!(spec.program, "claude");
    assert_eq!(spec.args, vec!["-p", "Implement the main shard"]);

    fs::remove_dir_all(root).expect("temp root should be removable");
}

#[test]
fn launcher_creates_log_files_for_each_shard() {
    let root = temp_root();
    let outcome = launch_worker(
        &request(Runtime::Codex, &root),
        "sh",
        &["-c", "printf worker-started"],
    )
    .expect("launcher should succeed");

    assert!(outcome.pid > 0);
    assert!(outcome.stdout_log.is_file(), "stdout log should exist");
    assert!(outcome.stderr_log.is_file(), "stderr log should exist");

    fs::remove_dir_all(root).expect("temp root should be removable");
}

#[test]
fn launcher_surfaces_spawn_failures_as_structured_errors() {
    let root = temp_root();
    let error = launch_worker(
        &request(Runtime::Claude, &root),
        "__patchlane_missing_binary__",
        &["-p", "Implement the main shard"],
    )
    .expect_err("missing binary should fail");

    match error {
        RuntimeLaunchError::SpawnFailed {
            program,
            shard_id,
            source: _,
        } => {
            assert_eq!(program, "__patchlane_missing_binary__");
            assert_eq!(shard_id, "01");
        }
    }

    fs::remove_dir_all(root).expect("temp root should be removable");
}
