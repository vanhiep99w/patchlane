use patchlane::cli::{Cli, Runtime, TopLevelCommand};
use clap::Parser;
use std::process::Command;

#[test]
fn parses_task_command_with_objective_and_optional_runtime() {
    let cli = Cli::try_parse_from([
        "patchlane",
        "task",
        "--runtime",
        "codex",
        "Design run store",
    ])
    .expect("task command should parse");

    match cli.command {
        TopLevelCommand::Task(task) => {
            assert!(matches!(task.runtime, Some(Runtime::Codex)));
            assert_eq!(task.objective, "Design run store");
        }
        other => panic!("expected task command, got {:?}", other),
    }
}

#[test]
fn top_level_help_lists_task_command() {
    let output = Command::new(env!("CARGO_BIN_EXE_patchlane"))
        .arg("--help")
        .output()
        .expect("help command should run");

    assert!(output.status.success(), "help should succeed");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("task"), "top-level help should list task: {stdout}");
}

#[test]
fn task_command_succeeds_without_runtime() {
    let output = Command::new(env!("CARGO_BIN_EXE_patchlane"))
        .args(["task", "Design run store"])
        .output()
        .expect("task command should run");

    assert!(
        output.status.success(),
        "task command should succeed, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert_eq!(
        stdout.trim(),
        "task queued: runtime: codex objective: Design run store"
    );
}

#[test]
fn task_command_surfaces_runtime_when_provided() {
    let output = Command::new(env!("CARGO_BIN_EXE_patchlane"))
        .args(["task", "--runtime", "codex", "Design run store"])
        .output()
        .expect("task command should run");

    assert!(
        output.status.success(),
        "task command should succeed, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert_eq!(
        stdout.trim(),
        "task queued: runtime: codex objective: Design run store"
    );
}

#[test]
fn task_command_surfaces_runtime_confirmation_prompt() {
    let output = Command::new(env!("CARGO_BIN_EXE_patchlane"))
        .args(["task", "Design run store"])
        .env("PATCHLANE_TEST_RUNTIME_CONTEXT", "ambiguous")
        .output()
        .expect("task command should run");

    assert!(output.status.success(), "task command should succeed");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert_eq!(
        stdout.trim(),
        "Detected both codex and claude contexts. Use codex? [y/n]"
    );
}
