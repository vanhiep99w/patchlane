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

    assert_eq!(stderr, expected);
}
