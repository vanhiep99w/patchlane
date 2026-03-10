use std::process::Command;

fn run_command(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_patchlane"))
        .args(args)
        .output()
        .expect("CLI should be executable")
}

#[test]
fn board_command_returns_a_compact_read_mostly_overview() {
    let output = run_command(&["swarm", "board"]);

    assert!(
        output.status.success(),
        "expected swarm board to succeed, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be valid UTF-8");
    let expected = "\
Board
  active runs: 1
  blocked shards: 1
  merge queue: 1 ready, 1 pending

Active Runs
  run-001 running 3 shards objective: Land compact status and watch surfaces

Blocked Shards
  shard-03 waiting on maintainer review

Next
  use `patchlane swarm status` for a single run or `patchlane swarm web` for a broader overview
";

    assert_eq!(stdout, expected);
    assert!(
        stderr.is_empty(),
        "successful board command should not write to stderr"
    );
}

#[test]
fn web_command_resolves_to_a_read_mostly_overview_entry_point() {
    let output = run_command(&["swarm", "web"]);

    assert!(
        output.status.success(),
        "expected swarm web to succeed, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be valid UTF-8");
    let expected = "\
Web
  read-mostly overview entry point

URL
  http://127.0.0.1:4040/overview

Focus
  active runs, blockers, placement, and merge queue summaries

Next
  keep `patchlane swarm watch` in the terminal for operational events
";

    assert_eq!(stdout, expected);
    assert!(
        stderr.is_empty(),
        "successful web command should not write to stderr"
    );
}
