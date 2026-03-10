use patchlane::store::run_store::{append_event, create_run, PersistedEvent, PersistedRun, PersistedShard};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_root() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("patchlane-status-{unique}"));
    fs::create_dir_all(&root).expect("temp root should be creatable");
    root
}

fn run_command(args: &[&str], state_root: &PathBuf) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_patchlane"))
        .args(args)
        .env("PATCHLANE_STATE_ROOT", state_root)
        .output()
        .expect("CLI should be executable")
}

#[test]
fn status_output_reads_the_latest_persisted_run_snapshot() {
    let state_root = temp_root();
    create_fixture_run(&state_root, "run-001", "claude", "completed");
    let latest_run_dir =
        create_fixture_run(&state_root, "run-002", "codex", "failed");
    append_event(
        &latest_run_dir,
        &PersistedEvent {
            timestamp: "2026-03-10T10:05:00Z".to_owned(),
            shard_id: Some("02".to_owned()),
            message: "spawn failure for shard 02 using codex: missing binary".to_owned(),
        },
    )
    .expect("failure event should persist");

    let output = run_command(&["swarm", "status"], &state_root);
    assert!(
        output.status.success(),
        "expected swarm status to succeed, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be valid UTF-8");
    let expected = "\
Run
  run-002 (degraded)
  runtime: codex
  objective: demo objective for run-002

Shards
  shard  runtime  pid    state      workspace  detail
  01     codex   4242   launched   run-002/workspace-01  none
  02     codex   -      failed     run-002/workspace-02  spawn failure for shard 02 using codex: missing binary

Blockers
  - shard 02 spawn failure for shard 02 using codex: missing binary

Latest Event
  2026-03-10T10:05:00Z spawn failure for shard 02 using codex: missing binary

Next
  patchlane swarm retry <shard-id>
";
    assert_eq!(stdout, expected);
    assert!(
        stderr.is_empty(),
        "successful status should not write to stderr"
    );

    fs::remove_dir_all(state_root).expect("temp root should be removable");
}

#[test]
fn status_output_suggests_retry_for_blocked_persisted_shards() {
    let state_root = temp_root();
    let latest_run_dir = create_fixture_run(&state_root, "run-003", "claude", "blocked");
    append_event(
        &latest_run_dir,
        &PersistedEvent {
            timestamp: "2026-03-10T10:07:00Z".to_owned(),
            shard_id: Some("02".to_owned()),
            message: "worker inactivity timeout for shard 02".to_owned(),
        },
    )
    .expect("blocked event should persist");

    let output = run_command(&["swarm", "status"], &state_root);
    assert!(
        output.status.success(),
        "expected swarm status to succeed, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be valid UTF-8");
    let expected = "\
Run
  run-003 (blocked)
  runtime: claude
  objective: demo objective for run-003

Shards
  shard  runtime  pid    state      workspace  detail
  01     claude  4242   launched   run-003/workspace-01  none
  02     claude  -      blocked    run-003/workspace-02  worker inactivity timeout for shard 02

Blockers
  - shard 02 worker inactivity timeout for shard 02

Latest Event
  2026-03-10T10:07:00Z worker inactivity timeout for shard 02

Next
  patchlane swarm retry <shard-id>
";
    assert_eq!(stdout, expected);
    assert!(
        stderr.is_empty(),
        "successful status should not write to stderr"
    );

    fs::remove_dir_all(state_root).expect("temp root should be removable");
}

#[test]
fn status_output_treats_running_shards_as_active_progress() {
    let state_root = temp_root();
    let latest_run_dir = create_run(
        &state_root,
        &PersistedRun {
            run_id: "run-004".to_owned(),
            runtime: "codex".to_owned(),
            objective: "demo objective for run-004".to_owned(),
            shard_count: 2,
        },
        &[
            PersistedShard {
                shard_id: "01".to_owned(),
                runtime: "codex".to_owned(),
                pid: Some(5252),
                state: "running".to_owned(),
                workspace: "run-004/workspace-01".to_owned(),
            },
            PersistedShard {
                shard_id: "02".to_owned(),
                runtime: "codex".to_owned(),
                pid: Some(5353),
                state: "running".to_owned(),
                workspace: "run-004/workspace-02".to_owned(),
            },
        ],
    )
    .expect("fixture run should persist");
    append_event(
        &latest_run_dir,
        &PersistedEvent {
            timestamp: "2026-03-10T10:09:00Z".to_owned(),
            shard_id: Some("02".to_owned()),
            message: "shard 02 running worker loop".to_owned(),
        },
    )
    .expect("running event should persist");

    let output = run_command(&["swarm", "status"], &state_root);
    assert!(
        output.status.success(),
        "expected swarm status to succeed, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    assert!(
        stdout.starts_with("Run\n  run-004 (active)\n"),
        "running shards should keep the run active, got {stdout:?}"
    );

    fs::remove_dir_all(state_root).expect("temp root should be removable");
}

fn create_fixture_run(
    state_root: &PathBuf,
    run_id: &str,
    runtime: &str,
    second_shard_state: &str,
) -> PathBuf {
    create_run(
        state_root,
        &PersistedRun {
            run_id: run_id.to_owned(),
            runtime: runtime.to_owned(),
            objective: format!("demo objective for {run_id}"),
            shard_count: 2,
        },
        &[
            PersistedShard {
                shard_id: "01".to_owned(),
                runtime: runtime.to_owned(),
                pid: Some(4242),
                state: "launched".to_owned(),
                workspace: format!("{run_id}/workspace-01"),
            },
            PersistedShard {
                shard_id: "02".to_owned(),
                runtime: runtime.to_owned(),
                pid: None,
                state: second_shard_state.to_owned(),
                workspace: format!("{run_id}/workspace-02"),
            },
        ],
    )
    .expect("fixture run should persist")
}
