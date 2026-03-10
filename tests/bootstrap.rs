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

#[test]
fn bootstrap_library_exposes_the_file_backed_run_store_surface() {
    let event = patchlane::store::run_store::PersistedEvent {
        timestamp: "2026-03-10T00:00:00Z".to_owned(),
        shard_id: Some("01".to_owned()),
        message: "worker launched".to_owned(),
    };

    let encoded = serde_json::to_string(&event).expect("store event should serialize");
    let decoded: patchlane::store::run_store::PersistedEvent =
        serde_json::from_str(&encoded).expect("store event should deserialize");

    assert_eq!(decoded.message, "worker launched");
    assert_eq!(patchlane::bootstrap_banner(), "Patchlane CLI bootstrap");
}
