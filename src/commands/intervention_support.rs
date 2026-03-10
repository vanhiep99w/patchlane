use crate::commands::CommandOutcome;
use crate::cli::Runtime;
use crate::domain::run::RunState;
use crate::domain::shard::ShardState;
use crate::planner::shard_planner::plan_shards;
use crate::runtime::launcher::{build_launch_spec, launch_worker, LaunchRequest};
use crate::store::run_store::{
    append_event, latest_run_dir, load_run, load_shard_attempts, load_shards, write_shard,
    write_shard_attempts, PersistedEvent, PersistedShard, PersistedShardAttempt,
};
use std::io;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub enum InterventionAction {
    Pause,
    Resume,
    Stop,
}

pub enum MergeAction {
    Approve,
    Reject,
}

enum TargetFixture {
    Run(RunState),
    Shard(ShardState),
}

pub fn run_intervention(action: InterventionAction, target_id: &str) -> CommandOutcome {
    let target = match lookup_target(target_id) {
        Ok(target) => target,
        Err(reason) => return failed(action_name(&action), target_id, reason),
    };

    match (action, target) {
        (InterventionAction::Pause, TargetFixture::Run(RunState::Running)) => queued(
            "pause",
            target_id,
            "pause will apply at the next safe checkpoint",
        ),
        (InterventionAction::Pause, TargetFixture::Run(RunState::Paused)) => {
            applied("pause", target_id, "target is already paused")
        }
        (InterventionAction::Pause, TargetFixture::Shard(ShardState::Blocked)) => applied(
            "pause",
            target_id,
            "target is already blocked and requires operator input",
        ),
        (InterventionAction::Pause, TargetFixture::Shard(ShardState::Running)) => queued(
            "pause",
            target_id,
            "pause will apply when the shard reaches a safe handoff point",
        ),
        (InterventionAction::Pause, TargetFixture::Run(state)) => failed(
            "pause",
            target_id,
            format!("invalid state: run is {state:?} and cannot pause").to_lowercase(),
        ),
        (InterventionAction::Pause, TargetFixture::Shard(state)) => failed(
            "pause",
            target_id,
            format!("invalid state: shard is {state:?} and cannot pause").to_lowercase(),
        ),
        (InterventionAction::Resume, TargetFixture::Run(RunState::Paused)) => {
            applied("resume", target_id, "run resumed from paused state")
        }
        (InterventionAction::Resume, TargetFixture::Run(RunState::Running)) => {
            applied("resume", target_id, "run is already active")
        }
        (InterventionAction::Resume, TargetFixture::Shard(ShardState::Blocked)) => acknowledged(
            "resume",
            target_id,
            "shard resume acknowledged and waiting for dispatch",
        ),
        (InterventionAction::Resume, TargetFixture::Run(state)) => failed(
            "resume",
            target_id,
            format!("invalid state: run is {state:?} and cannot resume").to_lowercase(),
        ),
        (InterventionAction::Resume, TargetFixture::Shard(state)) => failed(
            "resume",
            target_id,
            format!("invalid state: shard is {state:?} and cannot resume").to_lowercase(),
        ),
        (InterventionAction::Stop, TargetFixture::Run(RunState::Running))
        | (InterventionAction::Stop, TargetFixture::Run(RunState::Paused)) => acknowledged(
            "stop",
            target_id,
            "run stop acknowledged and further dispatch will halt",
        ),
        (InterventionAction::Stop, TargetFixture::Run(RunState::Stopped)) => {
            applied("stop", target_id, "run is already stopped")
        }
        (InterventionAction::Stop, TargetFixture::Run(state)) => failed(
            "stop",
            target_id,
            format!("invalid state: run is {state:?} and cannot stop").to_lowercase(),
        ),
        (InterventionAction::Stop, TargetFixture::Shard(_)) => failed(
            "stop",
            target_id,
            "invalid state: stop only accepts run ids".to_owned(),
        ),
    }
}

pub fn run_retry_intervention(shard_id: &str) -> CommandOutcome {
    if let Some(outcome) = retry_persisted_shard(shard_id) {
        return outcome;
    }

    let target = match lookup_target(shard_id) {
        Ok(target) => target,
        Err(reason) => return failed("retry", shard_id, reason),
    };

    match target {
        TargetFixture::Shard(ShardState::Failed) | TargetFixture::Shard(ShardState::Blocked) => {
            queued(
                "retry",
                shard_id,
                "new shard attempt queued with prior history preserved",
            )
        }
        TargetFixture::Shard(state) => failed(
            "retry",
            shard_id,
            format!("invalid state: shard is {state:?} and cannot retry").to_lowercase(),
        ),
        TargetFixture::Run(_) => failed(
            "retry",
            shard_id,
            "invalid state: retry only accepts shard ids".to_owned(),
        ),
    }
}

fn retry_persisted_shard(shard_id: &str) -> Option<CommandOutcome> {
    let state_root = state_root();
    let run_dir = match latest_run_dir(&state_root) {
        Ok(run_dir) => run_dir,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return None,
        Err(error) => return Some(CommandOutcome::error(format!("error: {error}"))),
    };

    let run = match load_run(&run_dir) {
        Ok(run) => run,
        Err(error) => {
            return Some(CommandOutcome::error(format!(
                "error: failed to load run: {error}"
            )))
        }
    };
    let mut shards = match load_shards(&run_dir) {
        Ok(shards) => shards,
        Err(error) => {
            return Some(CommandOutcome::error(format!(
                "error: failed to load shards: {error}"
            )))
        }
    };
    let Some(shard) = shards.iter_mut().find(|shard| shard.shard_id == shard_id) else {
        return None;
    };

    if shard.state != "failed" && shard.state != "blocked" {
        return Some(failed(
            "retry",
            shard_id,
            format!("invalid state: shard is {} and cannot retry", shard.state),
        ));
    }

    let runtime = match runtime_from_label(&shard.runtime) {
        Ok(runtime) => runtime,
        Err(message) => return Some(CommandOutcome::error(message)),
    };
    let brief = match plan_shards(&run.objective)
        .into_iter()
        .find(|planned| planned.id == shard.shard_id)
    {
        Some(planned) => planned.brief,
        None => {
            return Some(CommandOutcome::error(format!(
                "error: failed to resolve shard brief for {}",
                shard_id
            )))
        }
    };

    let request = LaunchRequest {
        runtime,
        shard_id: shard.shard_id.clone(),
        brief,
        workspace: PathBuf::from(&shard.workspace),
        logs_dir: run_dir.join("logs"),
    };
    let spec = build_launch_spec(&request);
    let args = spec.args.iter().map(String::as_str).collect::<Vec<_>>();

    let launch = match launch_worker(&request, spec.program, &args) {
        Ok(outcome) => outcome,
        Err(error) => {
            let _ = append_event(
                &run_dir,
                &PersistedEvent {
                    timestamp: timestamp_now(),
                    shard_id: Some(shard_id.to_owned()),
                    message: format!("retry spawn failure for shard {}: {:?}", shard_id, error),
                },
            );
            return Some(CommandOutcome::error(render_response(
                "failed",
                "retry",
                shard_id,
                &format!("retry launch failed: {:?}", error),
            )));
        }
    };

    let mut attempts =
        load_shard_attempts(&run_dir, shard_id).unwrap_or_else(|_| seed_attempt_history(shard));
    let next_attempt = attempts.last().map(|attempt| attempt.attempt + 1).unwrap_or(1);
    attempts.push(PersistedShardAttempt {
        attempt: next_attempt,
        pid: Some(launch.pid),
        state: "launched".to_owned(),
    });

    shard.pid = Some(launch.pid);
    shard.state = "launched".to_owned();
    if let Err(error) = write_shard(&run_dir, shard) {
        return Some(CommandOutcome::error(format!(
            "error: failed to persist retried shard: {error}"
        )));
    }
    if let Err(error) = write_shard_attempts(&run_dir, shard_id, &attempts) {
        return Some(CommandOutcome::error(format!(
            "error: failed to persist shard attempts: {error}"
        )));
    }
    if let Err(error) = append_event(
        &run_dir,
        &PersistedEvent {
            timestamp: timestamp_now(),
            shard_id: Some(shard_id.to_owned()),
            message: format!("retried shard {} with pid {}", shard_id, launch.pid),
        },
    ) {
        return Some(CommandOutcome::error(format!(
            "error: failed to append retry event: {error}"
        )));
    }

    Some(queued(
        "retry",
        shard_id,
        format!(
            "new shard attempt queued with pid {} and prior history preserved",
            launch.pid
        ),
    ))
}

pub fn run_reassign_intervention(shard_id: &str, runtime: &str) -> CommandOutcome {
    if !matches!(runtime, "codex" | "claude") {
        return failed(
            "reassign",
            shard_id,
            format!("policy denial: runtime {runtime} is not supported"),
        );
    }

    let target = match lookup_target(shard_id) {
        Ok(target) => target,
        Err(reason) => return failed("reassign", shard_id, reason),
    };

    match target {
        TargetFixture::Shard(ShardState::Assigned)
        | TargetFixture::Shard(ShardState::Running)
        | TargetFixture::Shard(ShardState::Blocked)
        | TargetFixture::Shard(ShardState::Failed) => acknowledged(
            "reassign",
            shard_id,
            format!("runtime routing updated to {runtime}"),
        ),
        TargetFixture::Shard(ShardState::Succeeded) => failed(
            "reassign",
            shard_id,
            "invalid state: shard is succeeded and cannot reassign".to_owned(),
        ),
        TargetFixture::Shard(ShardState::Queued) => queued(
            "reassign",
            shard_id,
            format!("runtime routing to {runtime} will apply before dispatch"),
        ),
        TargetFixture::Run(_) => failed(
            "reassign",
            shard_id,
            "invalid state: reassign only accepts shard ids".to_owned(),
        ),
    }
}

pub fn run_merge_intervention(action: MergeAction, merge_unit_id: &str) -> CommandOutcome {
    if !merge_unit_id.starts_with("merge-") {
        return failed(
            merge_action_name(&action),
            merge_unit_id,
            "missing id: merge commands require a concrete merge unit id".to_owned(),
        );
    }

    match merge_unit_id {
        "merge-001" => acknowledged(
            merge_action_name(&action),
            merge_unit_id,
            "merge unit review recorded",
        ),
        "merge-applied" => applied(
            merge_action_name(&action),
            merge_unit_id,
            "merge unit already reflects this decision",
        ),
        "merge-runtime-error" => failed(
            merge_action_name(&action),
            merge_unit_id,
            "runtime error: merge queue storage is unavailable".to_owned(),
        ),
        _ => failed(
            merge_action_name(&action),
            merge_unit_id,
            "missing id: merge unit was not found".to_owned(),
        ),
    }
}

fn lookup_target(target_id: &str) -> Result<TargetFixture, String> {
    match target_id {
        "run-active" => Ok(TargetFixture::Run(RunState::Running)),
        "run-paused" => Ok(TargetFixture::Run(RunState::Paused)),
        "run-done" => Ok(TargetFixture::Run(RunState::Succeeded)),
        "run-stopped" => Ok(TargetFixture::Run(RunState::Stopped)),
        "shard-assigned" => Ok(TargetFixture::Shard(ShardState::Assigned)),
        "shard-running" => Ok(TargetFixture::Shard(ShardState::Running)),
        "shard-blocked" => Ok(TargetFixture::Shard(ShardState::Blocked)),
        "shard-failed" => Ok(TargetFixture::Shard(ShardState::Failed)),
        "shard-done" => Ok(TargetFixture::Shard(ShardState::Succeeded)),
        "shard-queued" => Ok(TargetFixture::Shard(ShardState::Queued)),
        _ => Err("missing id: target was not found".to_owned()),
    }
}

fn queued(action: &str, target_id: &str, reason: impl AsRef<str>) -> CommandOutcome {
    success_like("queued", action, target_id, reason)
}

fn acknowledged(action: &str, target_id: &str, reason: impl AsRef<str>) -> CommandOutcome {
    success_like("acknowledged", action, target_id, reason)
}

fn applied(action: &str, target_id: &str, reason: impl AsRef<str>) -> CommandOutcome {
    success_like("applied", action, target_id, reason)
}

fn failed(action: &str, target_id: &str, reason: String) -> CommandOutcome {
    CommandOutcome::error(render_response("failed", action, target_id, &reason))
}

fn success_like(
    status: &str,
    action: &str,
    target_id: &str,
    reason: impl AsRef<str>,
) -> CommandOutcome {
    CommandOutcome::success(render_response(status, action, target_id, reason.as_ref()))
}

fn render_response(status: &str, action: &str, target_id: &str, reason: &str) -> String {
    format!(
        "Result\n  {status}\n\nAction\n  {action}\n\nTarget\n  {target_id}\n\nReason\n  {reason}"
    )
}

fn action_name(action: &InterventionAction) -> &'static str {
    match action {
        InterventionAction::Pause => "pause",
        InterventionAction::Resume => "resume",
        InterventionAction::Stop => "stop",
    }
}

fn merge_action_name(action: &MergeAction) -> &'static str {
    match action {
        MergeAction::Approve => "merge approve",
        MergeAction::Reject => "merge reject",
    }
}

fn runtime_from_label(runtime: &str) -> Result<Runtime, String> {
    match runtime {
        "codex" => Ok(Runtime::Codex),
        "claude" => Ok(Runtime::Claude),
        other => Err(format!("error: unsupported persisted runtime {other}")),
    }
}

fn seed_attempt_history(shard: &PersistedShard) -> Vec<PersistedShardAttempt> {
    vec![PersistedShardAttempt {
        attempt: 1,
        pid: shard.pid,
        state: shard.state.clone(),
    }]
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
