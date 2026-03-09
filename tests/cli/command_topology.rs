use std::process::Command;

fn run_command(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_patchlane"))
        .args(args)
        .output()
        .expect("CLI should be executable")
}

fn assert_unimplemented_command(args: &[&str], expected_stderr: &str) {
    let output = run_command(args);

    assert!(
        !output.status.success(),
        "expected {:?} to fail while stubbed",
        args
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be valid UTF-8");
    assert_eq!(stderr.trim(), expected_stderr);
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
    for args in [vec!["swarm", "status"], vec!["swarm", "watch"]] {
        assert_implemented_command(&args);
    }

    let cases = [
        (
            vec!["swarm", "pause"],
            "stub: swarm pause is not implemented",
        ),
        (
            vec!["swarm", "resume"],
            "stub: swarm resume is not implemented",
        ),
        (
            vec!["swarm", "retry"],
            "stub: swarm retry is not implemented",
        ),
        (
            vec!["swarm", "reassign"],
            "stub: swarm reassign is not implemented",
        ),
        (
            vec!["swarm", "merge", "approve"],
            "stub: swarm merge approve is not implemented",
        ),
        (
            vec!["swarm", "merge", "reject"],
            "stub: swarm merge reject is not implemented",
        ),
        (vec!["swarm", "stop"], "stub: swarm stop is not implemented"),
        (
            vec!["swarm", "board"],
            "stub: swarm board is not implemented",
        ),
        (vec!["swarm", "web"], "stub: swarm web is not implemented"),
    ];

    for (args, expected_stderr) in cases {
        assert_unimplemented_command(&args, expected_stderr);
    }

    assert_help_failure(
        &["swarm", "run"],
        &[
            "Usage: patchlane swarm run <OBJECTIVE>",
            "error: the following required arguments were not provided:",
            "<OBJECTIVE>",
        ],
    );
}

#[test]
fn command_topology_surfaces_help_for_incomplete_invocations() {
    assert_help_failure(&[], &["Usage: patchlane <COMMAND>", "Commands:", "swarm"]);
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
        &["swarm", "merge"],
        &[
            "Usage: patchlane swarm merge <COMMAND>",
            "Commands:",
            "approve",
            "reject",
        ],
    );
}
