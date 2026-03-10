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
    let root = std::env::temp_dir().join(format!("patchlane-watch-{unique}"));
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
fn watch_output_reads_persisted_lifecycle_events_without_transcript_noise() {
    let state_root = temp_root();
    let run_dir = create_run(
        &state_root,
        &PersistedRun {
            run_id: "run-001".to_owned(),
            runtime: "codex".to_owned(),
            objective: "stream real worker events".to_owned(),
            shard_count: 4,
        },
        &[
            PersistedShard {
                shard_id: "01".to_owned(),
                runtime: "codex".to_owned(),
                pid: Some(4101),
                state: "launched".to_owned(),
                workspace: "workspace-01".to_owned(),
            },
            PersistedShard {
                shard_id: "02".to_owned(),
                runtime: "codex".to_owned(),
                pid: Some(4102),
                state: "running".to_owned(),
                workspace: "workspace-02".to_owned(),
            },
            PersistedShard {
                shard_id: "03".to_owned(),
                runtime: "codex".to_owned(),
                pid: None,
                state: "failed".to_owned(),
                workspace: "workspace-03".to_owned(),
            },
            PersistedShard {
                shard_id: "04".to_owned(),
                runtime: "codex".to_owned(),
                pid: Some(4104),
                state: "completed".to_owned(),
                workspace: "workspace-04".to_owned(),
            },
        ],
    )
    .expect("fixture run should persist");

    for event in [
        PersistedEvent {
            timestamp: "2026-03-10T10:00:00Z".to_owned(),
            shard_id: Some("01".to_owned()),
            message: "launched local codex worker for shard 01".to_owned(),
        },
        PersistedEvent {
            timestamp: "2026-03-10T10:01:00Z".to_owned(),
            shard_id: Some("02".to_owned()),
            message: "shard 02 running worker loop".to_owned(),
        },
        PersistedEvent {
            timestamp: "2026-03-10T10:02:00Z".to_owned(),
            shard_id: Some("03".to_owned()),
            message: "spawn failure for shard 03 using codex: missing binary".to_owned(),
        },
        PersistedEvent {
            timestamp: "2026-03-10T10:03:00Z".to_owned(),
            shard_id: Some("04".to_owned()),
            message: "shard 04 completed brief".to_owned(),
        },
        PersistedEvent {
            timestamp: "2026-03-10T10:04:00Z".to_owned(),
            shard_id: Some("04".to_owned()),
            message: "assistant: transcript summary for shard 04".to_owned(),
        },
    ] {
        append_event(&run_dir, &event).expect("event should persist");
    }

    let output = run_command(&["swarm", "watch"], &state_root);

    assert!(
        output.status.success(),
        "expected swarm watch to succeed, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be valid UTF-8");
    let expected = "\
2026-03-10T10:00:00Z launched local codex worker for shard 01
2026-03-10T10:01:00Z shard 02 running worker loop
2026-03-10T10:02:00Z spawn failure for shard 03 using codex: missing binary
2026-03-10T10:03:00Z shard 04 completed brief
";

    assert_eq!(stdout, expected);
    assert!(
        stderr.is_empty(),
        "successful watch should not write to stderr"
    );
    assert!(
        !stdout.contains("transcript"),
        "watch output should avoid transcript framing"
    );
    assert!(
        !stdout.contains("assistant:"),
        "watch output should avoid raw speaker labels"
    );
    assert!(
        !stdout.contains("user:"),
        "watch output should avoid raw speaker labels"
    );
    assert!(
        !stdout.contains("assistant: transcript summary"),
        "watch output should drop transcript-like persisted events"
    );

    fs::remove_dir_all(state_root).expect("temp root should be removable");
}
