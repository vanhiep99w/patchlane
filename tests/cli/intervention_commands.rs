use std::process::Command;

fn run_command(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_patchlane"))
        .args(args)
        .output()
        .expect("CLI should be executable")
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
        let output = run_command(&args);
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
    let output = run_command(&["swarm", "pause", "run-paused"]);
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    assert!(stdout.contains("Result\n  applied"));
    assert!(stdout.contains("target is already paused"));

    let output = run_command(&["swarm", "merge", "approve", "merge-applied"]);
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    assert!(stdout.contains("Result\n  applied"));
    assert!(stdout.contains("already reflects this decision"));
}

#[test]
fn intervention_failures_surface_explicit_failure_reasons() {
    let output = run_command(&["swarm", "resume", "run-done"]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8(output.stderr).expect("stderr should be valid UTF-8");
    assert!(stderr.contains("Result\n  failed"));
    assert!(stderr.contains("invalid state: run is succeeded and cannot resume"));

    let output = run_command(&["swarm", "reassign", "shard-running", "--runtime", "gemini"]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8(output.stderr).expect("stderr should be valid UTF-8");
    assert!(stderr.contains("Result\n  failed"));
    assert!(stderr.contains("policy denial: runtime gemini is not supported"));
}

#[test]
fn merge_commands_require_a_concrete_merge_unit_id() {
    let output = run_command(&["swarm", "merge", "reject", "run-active"]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8(output.stderr).expect("stderr should be valid UTF-8");
    assert!(stderr.contains("Result\n  failed"));
    assert!(stderr.contains("missing id: merge commands require a concrete merge unit id"));
}
