use std::process::Command;

fn run_command(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_patchlane"))
        .args(args)
        .output()
        .expect("CLI should be executable")
}

#[test]
fn status_output_matches_the_snapshot_contract() {
    let output = run_command(&["swarm", "status"]);

    assert!(
        output.status.success(),
        "expected swarm status to succeed, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be valid UTF-8");
    let expected = "\
Run
  active
  objective: Land compact status and watch surfaces

Shards
  shard  state        branch                      owner    blockers
  01     done         feat/opening-block          agent-a  none
  02     running      feat/status-snapshot        agent-b  none
  03     blocked      feat/watch-events           agent-c  waiting on review

Blockers
  1 active blocker
  - shard 03 waiting on review from maintainer

Merge Queue
  1 ready, 1 pending
  - ready: shard 01 feat/opening-block
  - pending: shard 02 feat/status-snapshot

Latest Event
  2026-03-09T10:15:00Z review requested for shard 03 by maintainer

Next
  patchlane swarm watch
";

    assert_eq!(stdout, expected);
    assert!(
        stderr.is_empty(),
        "successful status should not write to stderr"
    );
}
