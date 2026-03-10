use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn run_command(args: &[&str]) -> std::process::Output {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let state_root = std::env::temp_dir().join(format!("patchlane-command-topology-{unique}"));
    std::fs::create_dir_all(&state_root).expect("temp state root should be creatable");

    Command::new(env!("CARGO_BIN_EXE_patchlane"))
        .args(args)
        .env("PATCHLANE_STATE_ROOT", &state_root)
        .output()
        .expect("CLI should be executable")
}

fn assert_implemented_command(args: &[&str]) {
    let output = run_command(args);

    assert!(
        output.status.success(),
        "expected {:?} to succeed once implemented, stderr: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be valid UTF-8");
    assert!(
        stderr.is_empty(),
        "implemented command {:?} should not write to stderr",
        args
    );
}

fn assert_help_failure(args: &[&str], expected_help_fragments: &[&str]) {
    let output = run_command(args);
    assert!(
        !output.status.success(),
        "expected {:?} to fail with help output",
        args,
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be valid UTF-8");
    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    let combined = format!("{stdout}{stderr}");
    for fragment in expected_help_fragments {
        assert!(
            combined.contains(fragment),
            "expected help output for {:?} to contain {:?}, got: {}",
            args,
            fragment,
            combined
        );
    }
}

#[test]
fn command_topology_recognizes_approved_swarm_commands() {
    assert_implemented_command(&["task", "Design run store"]);
    assert_implemented_command(&["tui"]);

    for args in [
        vec!["swarm", "status"],
        vec!["swarm", "watch"],
        vec!["swarm", "pause", "run-active"],
        vec!["swarm", "resume", "run-paused"],
        vec!["swarm", "merge", "approve", "merge-001"],
        vec!["swarm", "merge", "reject", "merge-001"],
        vec!["swarm", "stop", "run-active"],
        vec!["swarm", "board"],
        vec!["swarm", "web"],
    ] {
        assert_implemented_command(&args);
    }

    assert_help_failure(
        &["swarm", "run"],
        &[
            "Usage: patchlane swarm run --runtime <RUNTIME> <OBJECTIVE>",
            "error: the following required arguments were not provided:",
            "--runtime <RUNTIME>",
            "<OBJECTIVE>",
        ],
    );
}

#[test]
fn top_level_help_lists_tui_command() {
    let output = run_command(&["--help"]);
    assert!(output.status.success(), "help should succeed");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    assert!(
        stdout.contains("tui"),
        "top-level help should list tui command, got: {stdout}"
    );
}

#[test]
fn command_topology_surfaces_help_for_incomplete_invocations() {
    assert_help_failure(
        &[],
        &["Usage: patchlane <COMMAND>", "Commands:", "task", "swarm"],
    );
    assert_help_failure(
        &["swarm"],
        &[
            "Usage: patchlane swarm <COMMAND>",
            "Commands:",
            "run",
            "merge",
        ],
    );
    assert_help_failure(
        &["swarm", "pause"],
        &[
            "Usage: patchlane swarm pause <TARGET_ID>",
            "error: the following required arguments were not provided:",
            "<TARGET_ID>",
        ],
    );
    assert_help_failure(
        &["swarm", "merge"],
        &[
            "Usage: patchlane swarm merge <COMMAND>",
            "Commands:",
            "approve",
            "reject",
        ],
    );
    assert_help_failure(
        &["swarm", "merge", "approve"],
        &[
            "Usage: patchlane swarm merge approve <MERGE_UNIT_ID>",
            "error: the following required arguments were not provided:",
            "<MERGE_UNIT_ID>",
        ],
    );
}
