use patchlane::store::run_store::{load_events, load_run, load_shards};
use patchlane::orchestration::store::load_task_snapshot;
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

#[test]
fn cli_contract_covers_task_workflow_and_persisted_artifacts() {
    let state_root = temp_root();
    let task_root = state_root.join("tasks");

    let output = run_command(&["task", "--runtime", "codex", "Ship orchestration flow"], &state_root, "success");
    assert!(
        output.status.success(),
        "task command should succeed, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let mut run_dirs = fs::read_dir(&task_root)
        .expect("task root should be readable")
        .collect::<Result<Vec<_>, _>>()
        .expect("task root entries should load")
        .into_iter()
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    run_dirs.sort();
    assert_eq!(run_dirs.len(), 1, "expected one persisted task run");
    assert!(task_root.is_dir(), "task runs should live under PATCHLANE_STATE_ROOT/tasks");
    assert!(
        run_dirs[0].starts_with(&task_root),
        "task run should be created under the tasks namespace: {}",
        run_dirs[0].display()
    );
    let top_level_entries = fs::read_dir(&state_root)
        .expect("state root should still be readable")
        .collect::<Result<Vec<_>, _>>()
        .expect("state root entries should load");
    assert!(
        top_level_entries.iter().any(|entry| entry.file_name() == "tasks"),
        "state root should contain a dedicated tasks namespace"
    );
    assert!(
        top_level_entries.iter().all(|entry| match entry.file_name().to_str() {
            Some(name) => !name.starts_with("run-"),
            None => true,
        }),
        "task runs must not create top-level run-* directories in the swarm namespace"
    );

    let snapshot = load_task_snapshot(&run_dirs[0]).expect("task snapshot should load");
    assert_eq!(snapshot.run.runtime, "codex");
    assert!(snapshot.run.run_id.starts_with("run-"));
    assert_ne!(snapshot.run.run_id, "run-task-001");
    assert_eq!(
        run_dirs[0].file_name().and_then(|name| name.to_str()),
        Some(snapshot.run.run_id.as_str())
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid utf-8");
    assert!(stdout.contains("task queued: runtime: codex objective: Ship orchestration flow"));
    assert!(
        snapshot
            .artifacts
            .iter()
            .any(|artifact| artifact.path.contains("spec"))
    );
    assert!(
        snapshot
            .artifacts
            .iter()
            .any(|artifact| artifact.path.contains("plan"))
    );
    assert!(
        snapshot
            .checkpoints
            .iter()
            .any(|checkpoint| checkpoint.phase == "after-brainstorming")
    );
    assert!(
        snapshot
            .checkpoints
            .iter()
            .any(|checkpoint| checkpoint.phase == "after-writing-plans")
    );
    assert!(
        snapshot
            .events
            .iter()
            .any(|event| event.payload_summary.contains("Approve? [y/n]"))
    );
    let logs_dir = run_dirs[0].join("logs");
    assert!(logs_dir.is_dir(), "logs dir should exist");
    let log_entries = fs::read_dir(&logs_dir)
        .expect("logs dir should be readable")
        .collect::<Result<Vec<_>, _>>()
        .expect("logs entries should load")
        .into_iter()
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    assert!(
        log_entries.iter().any(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with("-stdout.log"))
        }),
        "expected at least one stdout log file"
    );
    assert!(
        log_entries.iter().any(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with("-stderr.log"))
        }),
        "expected at least one stderr log file"
    );
    let launched_stdout = logs_dir.join("shard-agent-agent-implement-stdout.log");
    let launched_stderr = logs_dir.join("shard-agent-agent-implement-stderr.log");
    assert!(launched_stdout.is_file(), "expected launched agent stdout log file");
    assert!(launched_stderr.is_file(), "expected launched agent stderr log file");
    let launched_stdout_contents =
        fs::read_to_string(&launched_stdout).expect("launched stdout log should be readable");
    let launched_stderr_contents =
        fs::read_to_string(&launched_stderr).expect("launched stderr log should be readable");
    assert!(
        launched_stdout_contents.contains(&format!(
            "launcher-contract run_id={}",
            snapshot.run.run_id
        )),
        "expected launched stdout to include generated run id marker"
    );
    assert!(
        launched_stderr_contents.contains("launcher-contract stderr agent=agent-implement"),
        "expected launched stderr to include launched agent marker"
    );
    assert!(
        snapshot
            .events
            .iter()
            .any(|event| {
                event.agent_id.as_deref() == Some("agent-implement")
                    && event.event_type
                        == patchlane::orchestration::model::AgentEventType::Phase
                    && event.payload_summary
                        == format!("launcher-contract:{}", snapshot.run.run_id)
            }),
        "expected launcher-injected agent-event path to persist a launched agent event"
    );
    assert!(
        snapshot
            .agents
            .iter()
            .any(|agent| {
                agent.agent_id == "agent-implement"
                    && agent.current_phase == format!("launcher-contract:{}", snapshot.run.run_id)
                    && agent.current_state
                        == patchlane::orchestration::model::OrchestratorState::Running
            }),
        "expected launched agent-event path to update persisted agent state"
    );

    fs::remove_dir_all(state_root).expect("temp root should be removable");
}
