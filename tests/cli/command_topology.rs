use std::process::Command;

fn assert_command(
    args: &[&str],
    expected_stdout: &str,
) {
    let output = Command::new(env!("CARGO_BIN_EXE_patchlane"))
        .args(args)
        .output()
        .expect("CLI should be executable");

    assert!(
        output.status.success(),
        "expected {:?} to succeed, stderr was: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    assert_eq!(stdout.trim(), expected_stdout);
}

#[test]
fn command_topology_recognizes_approved_swarm_commands() {
    let cases = [
        (vec!["swarm", "run"], "stub: swarm run"),
        (vec!["swarm", "status"], "stub: swarm status"),
        (vec!["swarm", "watch"], "stub: swarm watch"),
        (vec!["swarm", "pause"], "stub: swarm pause"),
        (vec!["swarm", "resume"], "stub: swarm resume"),
        (vec!["swarm", "retry"], "stub: swarm retry"),
        (vec!["swarm", "reassign"], "stub: swarm reassign"),
        (vec!["swarm", "merge", "approve"], "stub: swarm merge approve"),
        (vec!["swarm", "merge", "reject"], "stub: swarm merge reject"),
        (vec!["swarm", "stop"], "stub: swarm stop"),
        (vec!["swarm", "board"], "stub: swarm board"),
        (vec!["swarm", "web"], "stub: swarm web"),
    ];

    for (args, expected_stdout) in cases {
        assert_command(&args, expected_stdout);
    }
}
