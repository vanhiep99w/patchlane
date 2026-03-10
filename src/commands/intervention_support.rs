use crate::commands::CommandOutcome;
use crate::domain::run::RunState;
use crate::domain::shard::ShardState;

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
