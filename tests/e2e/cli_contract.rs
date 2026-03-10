use patchlane::store::run_store::{load_events, load_run, load_shards};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_root() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("patchlane-e2e-{unique}"));
    fs::create_dir_all(&root).expect("temp root should be creatable");
    root
}

fn run_command(args: &[&str], state_root: &PathBuf, launch_mode: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_patchlane"))
        .args(args)
        .env("PATCHLANE_STATE_ROOT", state_root)
        .env("PATCHLANE_TEST_RUNTIME_MODE", launch_mode)
        .output()
        .expect("CLI should be executable")
}

#[test]
fn cli_contract_covers_real_run_launch_and_persisted_artifacts() {
    let objective = "Land compact status and watch surfaces";
    let state_root = temp_root();

    let run_output = run_command(
        &["swarm", "run", "--runtime", "codex", objective],
        &state_root,
        "success",
    );
    assert!(
        run_output.status.success(),
        "expected swarm run to succeed, stderr: {}",
        String::from_utf8_lossy(&run_output.stderr)
    );
    let run_stdout = String::from_utf8(run_output.stdout).expect("stdout should be valid UTF-8");
    assert!(run_stdout.contains("Run\n  queued"));
    assert!(run_stdout.contains("run_id: run-"));
    assert!(run_stdout.contains("runtime: codex"));
    assert!(run_stdout.contains("shards: 4"));
    assert!(run_stdout.contains(&format!("Objective\n  {objective}")));
    assert!(run_stdout.contains("Next\n  launching 4 local codex workers"));

    let mut run_dirs = fs::read_dir(&state_root)
        .expect("state root should be readable")
        .collect::<Result<Vec<_>, _>>()
        .expect("state root entries should load")
        .into_iter()
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    run_dirs.sort();
    assert_eq!(run_dirs.len(), 1, "expected one persisted run directory");

    let run_dir = &run_dirs[0];
    let persisted_run = load_run(run_dir).expect("run metadata should load");
    let persisted_shards = load_shards(run_dir).expect("shards should load");
    let persisted_events = load_events(run_dir).expect("events should load");

    assert!(persisted_run.run_id.starts_with("run-"));
    assert_eq!(persisted_run.runtime, "codex");
    assert_eq!(persisted_run.objective, objective);
    assert_eq!(persisted_run.shard_count, 4);
    assert_eq!(persisted_shards.len(), 4);
    assert!(persisted_shards.iter().all(|shard| shard.state == "launched"));
    assert!(
        persisted_events
            .iter()
            .any(|event| event.message.contains("launched local codex worker")),
        "expected launched worker events, got {:?}",
        persisted_events
    );

    fs::remove_dir_all(state_root).expect("temp root should be removable");
}

#[test]
fn cli_contract_surfaces_runtime_spawn_failures_as_real_shard_failures() {
    let objective = "Launch workers with a missing runtime";
    let state_root = temp_root();

    let run_output = run_command(
        &["swarm", "run", "--runtime", "codex", objective],
        &state_root,
        "missing_binary",
    );
    assert!(
        run_output.status.success(),
        "run command should still complete with persisted failures, stderr: {}",
        String::from_utf8_lossy(&run_output.stderr)
    );
    let run_stdout = String::from_utf8(run_output.stdout).expect("stdout should be valid UTF-8");
    assert!(run_stdout.contains("failed: 4"));
    assert!(run_stdout.contains("spawn failure"));

    let run_dirs = fs::read_dir(&state_root)
        .expect("state root should be readable")
        .collect::<Result<Vec<_>, _>>()
        .expect("state root entries should load")
        .into_iter()
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    let run_dir = &run_dirs[0];

    let persisted_shards = load_shards(run_dir).expect("shards should load");
    let persisted_events = load_events(run_dir).expect("events should load");

    assert!(persisted_shards.iter().all(|shard| shard.state == "failed"));
    assert!(
        persisted_events
            .iter()
            .any(|event| event.message.contains("spawn failure")),
        "expected persisted spawn failure event"
    );

    fs::remove_dir_all(state_root).expect("temp root should be removable");
}

#[test]
fn readme_documents_the_local_cli_contract() {
    let readme = include_str!("../../README.md");

    assert!(readme.contains("swarm run"));
    assert!(readme.contains("--runtime <codex|claude>"));
    assert!(readme.contains("codex CLI"));
    assert!(readme.contains("claude CLI"));
    assert!(readme.contains("swarm status"));
    assert!(readme.contains("swarm watch"));
    assert!(readme.contains("PATCHLANE_STATE_ROOT"));
    assert!(readme.contains(".patchlane/"));
    assert!(readme.contains("events.jsonl"));
    assert!(readme.contains("logs/"));
    assert!(readme.contains("swarm pause run-active"));
    assert!(readme.contains("cargo test"));
    assert!(readme.contains("cargo run -- swarm run"));
}
