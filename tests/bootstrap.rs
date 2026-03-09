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
}

#[test]
fn bootstrap_cli_without_args_reports_bootstrap_only_state() {
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
        "bare invocation should identify the bootstrap binary"
    );
    assert!(
        stderr.contains("Planned swarm commands are not implemented yet."),
        "bare invocation should explain the current limitation"
    );
    assert!(
        stderr.contains("Use --help"),
        "bare invocation should direct the operator to help"
    );
}
