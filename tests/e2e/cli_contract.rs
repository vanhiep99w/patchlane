use std::process::Command;

fn run_command(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_patchlane"))
        .args(args)
        .output()
        .expect("CLI should be executable")
}

#[test]
fn cli_contract_covers_run_status_watch_and_intervention_flow() {
    let objective = "Land compact status and watch surfaces";

    let run_output = run_command(&["swarm", "run", objective]);
    assert!(
        run_output.status.success(),
        "expected swarm run to succeed, stderr: {}",
        String::from_utf8_lossy(&run_output.stderr)
    );
    let run_stdout = String::from_utf8(run_output.stdout).expect("stdout should be valid UTF-8");
    assert!(run_stdout.contains("Run\n  queued"));
    assert!(run_stdout.contains(&format!("Objective\n  {objective}")));
    assert!(run_stdout.contains("Next\n  waiting for planner and runtime integration"));

    let status_output = run_command(&["swarm", "status"]);
    assert!(
        status_output.status.success(),
        "expected swarm status to succeed, stderr: {}",
        String::from_utf8_lossy(&status_output.stderr)
    );
    let status_stdout =
        String::from_utf8(status_output.stdout).expect("stdout should be valid UTF-8");
    assert!(status_stdout.contains(&format!("objective: {objective}")));
    assert!(status_stdout.contains("Latest Event"));
    assert!(status_stdout.contains("Next\n  patchlane swarm watch"));

    let watch_output = run_command(&["swarm", "watch"]);
    assert!(
        watch_output.status.success(),
        "expected swarm watch to succeed, stderr: {}",
        String::from_utf8_lossy(&watch_output.stderr)
    );
    let watch_stdout = String::from_utf8(watch_output.stdout).expect("stdout should be valid UTF-8");
    assert!(watch_stdout.contains("workflow stage: dispatching shards"));
    assert!(watch_stdout.contains("workflow stage: merging clean shards"));

    let intervention_output = run_command(&["swarm", "pause", "run-active"]);
    assert!(
        intervention_output.status.success(),
        "expected swarm pause to succeed, stderr: {}",
        String::from_utf8_lossy(&intervention_output.stderr)
    );
    let intervention_stdout =
        String::from_utf8(intervention_output.stdout).expect("stdout should be valid UTF-8");
    assert!(intervention_stdout.contains("Result\n  queued"));
    assert!(intervention_stdout.contains("Reason\n  pause will apply at the next safe checkpoint"));
}

#[test]
fn readme_documents_the_local_cli_contract() {
    let readme = include_str!("../../README.md");

    assert!(readme.contains("swarm run"));
    assert!(readme.contains("swarm status"));
    assert!(readme.contains("swarm watch"));
    assert!(readme.contains("swarm pause run-active"));
    assert!(readme.contains("cargo test"));
    assert!(readme.contains("cargo run -- swarm run"));
}
