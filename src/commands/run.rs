use crate::cli::{RunCommand, Runtime};
use crate::commands::CommandOutcome;
use crate::planner::shard_planner::plan_shards;
use crate::renderers::run_renderer::{render_opening_block, RunOpeningBlock};
use crate::runtime::launcher::{build_launch_spec, launch_worker, LaunchRequest};
use crate::services::placement_engine::{decide_placement, PlacementDecisionInput, PlacementMode};
use crate::store::run_store::{
    append_event, create_run, write_shard, PersistedEvent, PersistedRun, PersistedShard,
};
use crate::workspaces::worktree_manager::allocate_workspace;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn execute(command: RunCommand) -> CommandOutcome {
    let objective = command.objective;

    if objective.contains('\n') || objective.contains('\r') {
        return CommandOutcome::error("error: objective must be a single line".to_owned());
    }

    let run_id = generate_run_id();
    let state_root = state_root();
    let planned_shards = plan_shards(&objective);

    let persisted_shards = planned_shards
        .iter()
        .map(|shard| {
            let workspace = allocate_workspace(&state_root, &run_id, shard.id)
                .map_err(|error| {
                    format!(
                        "error: failed to allocate workspace for shard {}: {:?}",
                        shard.id, error
                    )
                })?;
            Ok(PersistedShard {
                shard_id: shard.id.to_owned(),
                runtime: runtime_label(&command.runtime).to_owned(),
                pid: None,
                state: "queued".to_owned(),
                workspace: workspace.display().to_string(),
            })
        })
        .collect::<Result<Vec<_>, String>>();

    let mut persisted_shards = match persisted_shards {
        Ok(shards) => shards,
        Err(message) => return CommandOutcome::error(message),
    };

    let run_dir = match create_run(
        &state_root,
        &PersistedRun {
            run_id: run_id.clone(),
            runtime: runtime_label(&command.runtime).to_owned(),
            objective: objective.clone(),
            shard_count: planned_shards.len(),
        },
        &persisted_shards,
    ) {
        Ok(run_dir) => run_dir,
        Err(error) => {
            return CommandOutcome::error(format!(
                "error: failed to persist run metadata: {error}"
            ));
        }
    };

    let placement = decide_placement(PlacementDecisionInput {
        mode: PlacementMode::Balanced,
        shard_count: planned_shards.len(),
        writable_shard_count: planned_shards.len(),
        has_overlap_risk: planned_shards.len() > 1,
        repo_is_dirty: false,
        blocked_reason: None,
    });

    let mut failed_count = 0usize;
    for (planned, persisted) in planned_shards.iter().zip(persisted_shards.iter_mut()) {
        let request = LaunchRequest {
            runtime: command.runtime.clone(),
            shard_id: planned.id.to_owned(),
            brief: planned.brief.clone(),
            workspace: PathBuf::from(&persisted.workspace),
            logs_dir: run_dir.join("logs"),
        };
        let spec = build_launch_spec(&request);
        let args = spec.args.iter().map(String::as_str).collect::<Vec<_>>();

        match launch_worker(&request, spec.program, &args) {
            Ok(outcome) => {
                persisted.pid = Some(outcome.pid);
                persisted.state = "launched".to_owned();
                let _ = write_shard(&run_dir, persisted);
                let _ = append_event(
                    &run_dir,
                    &PersistedEvent {
                        timestamp: timestamp_now(),
                        shard_id: Some(planned.id.to_owned()),
                        message: format!(
                            "launched local {} worker for shard {}",
                            runtime_label(&command.runtime),
                            planned.id
                        ),
                    },
                );
            }
            Err(error) => {
                failed_count += 1;
                persisted.state = "failed".to_owned();
                let _ = write_shard(&run_dir, persisted);
                let _ = append_event(
                    &run_dir,
                    &PersistedEvent {
                        timestamp: timestamp_now(),
                        shard_id: Some(planned.id.to_owned()),
                        message: format!(
                            "spawn failure for shard {} using {}: {:?}",
                            planned.id,
                            runtime_label(&command.runtime),
                            error
                        ),
                    },
                );
            }
        }
    }

    let next_step = if failed_count == 0 {
        format!(
            "launching {} local {} workers",
            planned_shards.len(),
            runtime_label(&command.runtime)
        )
    } else {
        format!("spawn failure recorded for failed: {failed_count}")
    };

    let opening = RunOpeningBlock::new(
        run_id,
        command.runtime,
        objective,
        planned_shards.len(),
        placement,
        failed_count,
        next_step,
    );

    CommandOutcome::success(render_opening_block(&opening))
}

fn generate_run_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    format!("run-{nanos}")
}

fn state_root() -> PathBuf {
    std::env::var_os("PATCHLANE_STATE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".patchlane"))
}

fn timestamp_now() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_secs();
    format!("{seconds}")
}

fn runtime_label(runtime: &Runtime) -> &'static str {
    match runtime {
        Runtime::Codex => "codex",
        Runtime::Claude => "claude",
    }
}
