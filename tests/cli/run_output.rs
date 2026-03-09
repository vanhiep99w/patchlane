use std::process::Command;

fn run_command(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_patchlane"))
        .args(args)
        .output()
        .expect("CLI should be executable")
}

#[test]
fn run_output_matches_the_opening_block_contract() {
    let output = run_command(&["swarm", "run", "demo objective"]);

    assert!(
        output.status.success(),
        "expected swarm run with an objective to succeed, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be valid UTF-8");
    let expected = "\
Run
  queued

Objective
  demo objective

Plan
  1. Capture the requested objective.
  2. Prepare a placeholder execution plan.

Placement
  pending placement decision

Next
  waiting for planner and runtime integration
";

    assert_eq!(stdout, expected);
    assert!(stderr.is_empty(), "successful run should not write to stderr");
}

#[test]
fn run_output_requires_an_objective_argument() {
    let output = run_command(&["swarm", "run"]);

    assert!(
        !output.status.success(),
        "expected swarm run without an objective to fail"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be valid UTF-8");

    assert!(stdout.is_empty(), "help/error path should not write to stdout");
    assert!(stderr.contains("Usage: patchlane swarm run <OBJECTIVE>"));
    assert!(stderr.contains("error: the following required arguments were not provided:"));
}

#[test]
fn run_output_rejects_multiline_objectives() {
    let output = run_command(&["swarm", "run", "demo objective\nNext"]);

    assert!(
        !output.status.success(),
        "expected multiline objective to be rejected"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be valid UTF-8");

    assert!(stdout.is_empty(), "invalid objective path should not write to stdout");
    assert_eq!(stderr, "error: objective must be a single line\n");
}
