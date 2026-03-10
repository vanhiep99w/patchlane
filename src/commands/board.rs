use crate::commands::CommandOutcome;
use crate::events::run_events::{derive_board_snapshot, empty_board_snapshot};
use crate::orchestration::store::load_task_snapshot;
use crate::store::run_store::{load_run, load_shards};
use std::fs;
use std::io;
use std::path::PathBuf;

pub fn execute() -> CommandOutcome {
    let state_root = state_root();
    let task_root = state_root.join("tasks");
    let snapshots = match load_all_task_snapshots(&task_root) {
        Ok(snapshots) if snapshots.is_empty() => empty_board_snapshot(),
        Ok(snapshots) => derive_board_snapshot(&snapshots),
        Err(error) if error.kind() == io::ErrorKind::NotFound => return legacy_board(&state_root),
        Err(error) => return CommandOutcome::error(format!("error: {error}")),
    };

    if snapshots.runs.is_empty() {
        return legacy_board(&state_root);
    }

    let mut lines = vec![
        "Board".to_owned(),
        format!("  active runs: {}", snapshots.active_runs),
        format!("  blocked agents: {}", snapshots.blocked_agents),
    ];

    lines.extend([String::new(), "Active Runs".to_owned()]);
    if snapshots.runs.is_empty() {
        lines.push("  none".to_owned());
    } else {
        for run in snapshots.runs {
            lines.push(format!(
                "  {} {} {} agents objective: {}",
                run.id, run.state, run.agent_count, run.objective
            ));
        }
    }

    lines.extend([String::new(), "Blocked Agents".to_owned()]);
    if snapshots.blocked.is_empty() {
        lines.push("  none".to_owned());
    } else {
        for blocked in snapshots.blocked {
            lines.push(format!("  {blocked}"));
        }
    }

    lines.extend([
        String::new(),
        "Next".to_owned(),
        "  use `patchlane swarm status` for the latest run or `patchlane swarm watch` for event flow"
            .to_owned(),
    ]);

    CommandOutcome::success(lines.join("\n"))
}

fn legacy_board(state_root: &PathBuf) -> CommandOutcome {
    let run_dir = match latest_legacy_run_dir(state_root) {
        Ok(path) => path,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return CommandOutcome::success(
                "\
Board
  active runs: 0
  blocked shards: 0
  merge queue: unavailable

Active Runs
  none

Blocked Shards
  none

Next
  use `patchlane swarm status` for a single run or `patchlane swarm web` for a broader overview
"
                .to_owned(),
            );
        }
        Err(error) => return CommandOutcome::error(format!("error: {error}")),
    };
    let run = match load_run(&run_dir) {
        Ok(run) => run,
        Err(error) => return CommandOutcome::error(format!("error: failed to load run: {error}")),
    };
    let shards = match load_shards(&run_dir) {
        Ok(shards) => shards,
        Err(error) => return CommandOutcome::error(format!("error: failed to load shards: {error}")),
    };

    let active_runs = if shards.is_empty() { 0 } else { 1 };
    let blocked = shards
        .iter()
        .filter(|shard| shard.state == "failed" || shard.state == "blocked")
        .map(|shard| format!("shard-{} {}", shard.shard_id, shard.state))
        .collect::<Vec<_>>();
    let state = if shards.iter().any(|shard| shard.state == "failed") {
        "degraded"
    } else if shards.iter().all(|shard| shard.state == "completed") {
        "completed"
    } else if shards
        .iter()
        .any(|shard| shard.state == "launched" || shard.state == "running")
    {
        "active"
    } else {
        "queued"
    };

    let mut lines = vec![
        "Board".to_owned(),
        format!("  active runs: {active_runs}"),
        format!("  blocked shards: {}", blocked.len()),
        "  merge queue: unavailable".to_owned(),
        String::new(),
        "Active Runs".to_owned(),
    ];

    if active_runs == 0 {
        lines.push("  none".to_owned());
    } else {
        lines.push(format!(
            "  {} {} {} shards objective: {}",
            run.run_id, state, run.shard_count, run.objective
        ));
    }

    lines.extend([String::new(), "Blocked Shards".to_owned()]);
    if blocked.is_empty() {
        lines.push("  none".to_owned());
    } else {
        lines.extend(blocked.into_iter().map(|line| format!("  {line}")));
    }

    lines.extend([
        String::new(),
        "Next".to_owned(),
        "  use `patchlane swarm status` for a single run or `patchlane swarm web` for a broader overview"
            .to_owned(),
    ]);

    CommandOutcome::success(lines.join("\n"))
}

fn latest_legacy_run_dir(root: &PathBuf) -> io::Result<PathBuf> {
    let mut run_dirs = fs::read_dir(root)?
        .collect::<io::Result<Vec<_>>>()?
        .into_iter()
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_dir()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name != "tasks")
        })
        .collect::<Vec<_>>();
    run_dirs.sort();
    run_dirs
        .pop()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no persisted legacy runs found"))
}

fn load_all_task_snapshots(root: &PathBuf) -> io::Result<Vec<crate::orchestration::model::TaskSnapshot>> {
    let mut snapshots = fs::read_dir(root)?
        .collect::<io::Result<Vec<_>>>()?
        .into_iter()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .filter_map(|run_dir| load_task_snapshot(&run_dir).ok())
        .collect::<Vec<_>>();

    snapshots.sort_by(|left, right| right.run.updated_at.cmp(&left.run.updated_at));
    Ok(snapshots)
}

fn state_root() -> PathBuf {
    std::env::var_os("PATCHLANE_STATE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".patchlane"))
}
