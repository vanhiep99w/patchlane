use std::process::Command;

#[test]
fn bootstrap_cli_help_reports_current_surface() {
    let output = Command::new(env!("CARGO_BIN_EXE_patchlane"))
        .arg("--help")
        .output()
        .expect("bootstrap CLI should be executable");

    assert!(output.status.success(), "help command should succeed");

    let stdout = String::from_utf8(output.stdout).expect("help output should be valid UTF-8");

    assert!(
        stdout.contains("Patchlane"),
        "help output should mention Patchlane"
    );
    assert!(
        stdout.contains("Planned swarm commands are not implemented yet."),
        "help output should describe the bootstrap limitation"
    );
    assert!(
        stdout.contains("swarm"),
        "help output should expose the swarm command tree"
    );
}

#[test]
fn bootstrap_cli_without_args_reports_command_surface_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_patchlane"))
        .output()
        .expect("bootstrap CLI should be executable");

    assert!(
        !output.status.success(),
        "bare invocation should not look like a working command surface"
    );

    let stderr =
        String::from_utf8(output.stderr).expect("bootstrap stderr output should be valid UTF-8");

    assert!(
        stderr.contains("Patchlane CLI bootstrap"),
        "bare invocation should still identify the bootstrap binary"
    );
    assert!(
        stderr.contains("Usage: patchlane <COMMAND>"),
        "bare invocation should surface top-level help"
    );
    assert!(
        stderr.contains("swarm"),
        "bare invocation should expose the swarm namespace"
    );
}
