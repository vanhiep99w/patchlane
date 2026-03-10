use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_root() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("patchlane-run-output-{unique}"));
    fs::create_dir_all(&root).expect("temp root should be creatable");
    root
}

fn run_command(args: &[&str]) -> std::process::Output {
    let state_root = temp_root();
    let output = Command::new(env!("CARGO_BIN_EXE_patchlane"))
        .args(args)
        .env("PATCHLANE_STATE_ROOT", &state_root)
        .env("PATCHLANE_TEST_RUNTIME_MODE", "success")
        .output()
        .expect("CLI should be executable");
    fs::remove_dir_all(state_root).expect("temp root should be removable");
    output
}

#[test]
fn run_output_matches_the_opening_block_contract() {
    let output = run_command(&["swarm", "run", "--runtime", "codex", "demo objective"]);

    assert!(
        output.status.success(),
        "expected swarm run with an objective to succeed, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be valid UTF-8");

    assert!(stdout.starts_with("Run\n  queued\n  run_id: run-"));
    assert!(stdout.contains("runtime: codex"));
    assert!(stdout.contains("Objective\n  demo objective"));
    assert!(stdout.contains("Plan\n  shards: 4\n  failed: 0"));
    assert!(stdout.contains("Placement\n  worktree: multiple writable shards need isolated worktrees"));
    assert!(stdout.contains("Next\n  launching 4 local codex workers"));
    assert!(
        stderr.is_empty(),
        "successful run should not write to stderr"
    );
}

#[test]
fn run_output_supports_both_runtime_variants() {
    for runtime in ["codex", "claude"] {
        let output = run_command(&["swarm", "run", "--runtime", runtime, "demo objective"]);

        assert!(
            output.status.success(),
            "expected swarm run with runtime {runtime} to succeed, stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
        assert!(stdout.contains(&format!("runtime: {runtime}")));
    }
}

#[test]
fn run_output_requires_a_runtime_argument() {
    let output = run_command(&["swarm", "run", "demo objective"]);

    assert!(
        !output.status.success(),
        "expected swarm run without a runtime to fail"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be valid UTF-8");

    assert!(
        stdout.is_empty(),
        "help/error path should not write to stdout"
    );
    assert!(stderr.contains("Usage: patchlane swarm run --runtime <RUNTIME> <OBJECTIVE>"));
    assert!(stderr.contains("error: the following required arguments were not provided:"));
    assert!(stderr.contains("--runtime <RUNTIME>"));
}

#[test]
fn run_output_rejects_unknown_runtime_values() {
    let output = run_command(&["swarm", "run", "--runtime", "gemini", "demo objective"]);

    assert!(
        !output.status.success(),
        "expected swarm run with an unsupported runtime to fail"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be valid UTF-8");

    assert!(
        stdout.is_empty(),
        "invalid runtime path should not write to stdout"
    );
    assert!(stderr.contains("invalid value 'gemini'"));
    assert!(stderr.contains("[possible values: codex, claude]"));
}

#[test]
fn run_output_rejects_multiline_objectives() {
    let output = run_command(&["swarm", "run", "--runtime", "codex", "demo objective\nNext"]);

    assert!(
        !output.status.success(),
        "expected multiline objective to be rejected"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be valid UTF-8");

    assert!(
        stdout.is_empty(),
        "invalid objective path should not write to stdout"
    );
    assert_eq!(stderr, "error: objective must be a single line\n");
}
