use patchlane::store::run_store::{
    append_event, create_run, latest_run_dir, load_events, load_shard_attempts, load_shards,
    write_shard_attempts, PersistedEvent, PersistedRun, PersistedShard, PersistedShardAttempt,
};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_root() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("patchlane-intervention-{unique}"));
    fs::create_dir_all(&root).expect("temp root should be creatable");
    root
}

fn run_command(args: &[&str], state_root: Option<&PathBuf>, launch_mode: Option<&str>) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_patchlane"));
    command.args(args);
    if let Some(root) = state_root {
        command.env("PATCHLANE_STATE_ROOT", root);
    }
    if let Some(mode) = launch_mode {
        command.env("PATCHLANE_TEST_RUNTIME_MODE", mode);
    }
    command.output().expect("CLI should be executable")
}

#[test]
fn intervention_commands_return_only_operator_visible_results() {
    let cases = [
        (
            vec!["swarm", "pause", "run-active"],
            0,
            "Result\n  queued\n",
            "",
        ),
        (
            vec!["swarm", "resume", "run-paused"],
            0,
            "Result\n  applied\n",
            "",
        ),
        (
            vec!["swarm", "retry", "shard-failed"],
            0,
            "Result\n  queued\n",
            "",
        ),
        (
            vec!["swarm", "reassign", "shard-running", "--runtime", "codex"],
            0,
            "Result\n  acknowledged\n",
            "",
        ),
        (
            vec!["swarm", "merge", "approve", "merge-001"],
            0,
            "Result\n  acknowledged\n",
            "",
        ),
        (
            vec!["swarm", "stop", "run-active"],
            0,
            "Result\n  acknowledged\n",
            "",
        ),
    ];

    for (args, expected_code, stdout_prefix, stderr_prefix) in cases {
        let output = run_command(&args, None, None);
        assert_eq!(
            output.status.code(),
            Some(expected_code),
            "unexpected exit code for {:?}",
            args
        );

        let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
        let stderr = String::from_utf8(output.stderr).expect("stderr should be valid UTF-8");

        assert!(
            stdout.starts_with(stdout_prefix),
            "expected stdout for {:?} to start with {:?}, got {:?}",
            args,
            stdout_prefix,
            stdout
        );
        assert!(
            stderr.starts_with(stderr_prefix),
            "expected stderr for {:?} to start with {:?}, got {:?}",
            args,
            stderr_prefix,
            stderr
        );

        let combined = format!("{stdout}{stderr}");
        assert!(
            combined.contains("\n  queued\n")
                || combined.contains("\n  acknowledged\n")
                || combined.contains("\n  applied\n")
                || combined.contains("\n  failed\n"),
            "intervention result must stay within the approved vocabulary: {:?}",
            combined
        );
    }
}

#[test]
fn intervention_commands_are_idempotent_from_the_operator_perspective() {
    let output = run_command(&["swarm", "pause", "run-paused"], None, None);
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    assert!(stdout.contains("Result\n  applied"));
    assert!(stdout.contains("target is already paused"));

    let output = run_command(&["swarm", "merge", "approve", "merge-applied"], None, None);
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    assert!(stdout.contains("Result\n  applied"));
    assert!(stdout.contains("already reflects this decision"));
}

#[test]
fn intervention_failures_surface_explicit_failure_reasons() {
    let output = run_command(&["swarm", "resume", "run-done"], None, None);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8(output.stderr).expect("stderr should be valid UTF-8");
    assert!(stderr.contains("Result\n  failed"));
    assert!(stderr.contains("invalid state: run is succeeded and cannot resume"));

    let output = run_command(
        &["swarm", "reassign", "shard-running", "--runtime", "gemini"],
        None,
        None,
    );
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8(output.stderr).expect("stderr should be valid UTF-8");
    assert!(stderr.contains("Result\n  failed"));
    assert!(stderr.contains("policy denial: runtime gemini is not supported"));
}

#[test]
fn merge_commands_require_a_concrete_merge_unit_id() {
    let output = run_command(&["swarm", "merge", "reject", "run-active"], None, None);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8(output.stderr).expect("stderr should be valid UTF-8");
    assert!(stderr.contains("Result\n  failed"));
    assert!(stderr.contains("missing id: merge commands require a concrete merge unit id"));
}

#[test]
fn retry_relaunches_a_failed_persisted_shard_with_new_pid_and_attempt_history() {
    let state_root = temp_root();
    let run_dir = create_run(
        &state_root,
        &PersistedRun {
            run_id: "run-001".to_owned(),
            runtime: "codex".to_owned(),
            objective: "relaunch failed shard".to_owned(),
            shard_count: 1,
        },
        &[PersistedShard {
            shard_id: "03".to_owned(),
            runtime: "codex".to_owned(),
            pid: Some(1111),
            state: "failed".to_owned(),
            workspace: "workspace-03".to_owned(),
        }],
    )
    .expect("fixture run should persist");
    append_event(
        &run_dir,
        &PersistedEvent {
            timestamp: "2026-03-10T11:00:00Z".to_owned(),
            shard_id: Some("03".to_owned()),
            message: "spawn failure for shard 03 using codex: missing binary".to_owned(),
        },
    )
    .expect("failure event should persist");
    write_shard_attempts(
        &run_dir,
        "03",
        &[PersistedShardAttempt {
            attempt: 1,
            pid: Some(1111),
            state: "failed".to_owned(),
        }],
    )
    .expect("attempt history should persist");

    let output = run_command(&["swarm", "retry", "03"], Some(&state_root), Some("success"));
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    assert!(stdout.contains("Result\n  queued"));
    assert!(stdout.contains("Action\n  retry"));
    assert!(stdout.contains("Target\n  03"));

    let latest_run = latest_run_dir(&state_root).expect("latest run dir should load");
    let shards = load_shards(&latest_run).expect("shards should load");
    let retried = shards
        .into_iter()
        .find(|shard| shard.shard_id == "03")
        .expect("retried shard should exist");
    let new_pid = retried.pid.expect("retried shard should have new pid");
    assert_ne!(new_pid, 1111);
    assert_eq!(retried.state, "launched");

    let attempts = load_shard_attempts(&latest_run, "03").expect("attempts should load");
    assert_eq!(attempts.len(), 2);
    assert_eq!(attempts[0].attempt, 1);
    assert_eq!(attempts[0].pid, Some(1111));
    assert_eq!(attempts[0].state, "failed");
    assert_eq!(attempts[1].attempt, 2);
    assert_eq!(attempts[1].pid, Some(new_pid));
    assert_eq!(attempts[1].state, "launched");

    let events = load_events(&latest_run).expect("events should load");
    assert!(
        events
            .iter()
            .any(|event| event.message.contains("retried shard 03 with pid")),
        "retry event should be recorded, got {:?}",
        events
    );

    fs::remove_dir_all(state_root).expect("temp root should be removable");
}
