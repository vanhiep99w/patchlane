use crate::cli::{Runtime, TaskCommand};
use crate::commands::CommandOutcome;
use crate::orchestration::runtime::{
    apply_runtime_confirmation, resolve_runtime, DetectionContext, RuntimeResolutionState,
};
use crate::orchestration::workflow::execute_task_workflow;
use std::path::PathBuf;

pub fn execute(command: TaskCommand) -> CommandOutcome {
    let resolved = match resolve_runtime(command.runtime.clone(), DetectionContext::from_env()) {
        Ok(resolution) => resolution,
        Err(error) => return CommandOutcome::error(format!("error: {}", error.message)),
    };

    if resolved.state == RuntimeResolutionState::WaitingForConfirmation {
        return CommandOutcome::success(
            resolved
                .confirmation_prompt
                .unwrap_or_else(|| "Use detected runtime? [y/n]".to_owned()),
        );
    }

    let objective = command.objective.clone();
    let command = TaskCommand {
        runtime: Some(resolved.runtime.clone()),
        objective: objective.clone(),
    };
    let state_root = state_root();
    match execute_task_workflow(&state_root, command) {
        Ok(_) => {
            let runtime_segment = apply_runtime_segment(&resolved.runtime);
            CommandOutcome::success(format!(
                "task queued:{runtime_segment} objective: {}",
                objective
            ))
        }
        Err(error) => CommandOutcome::error(format!("error: {error}")),
    }
}

#[allow(dead_code)]
fn confirm_runtime_for_tests(input: &str, runtime: Runtime) -> CommandOutcome {
    match apply_runtime_confirmation(input, resolve_runtime(Some(runtime), DetectionContext::codex()).expect("explicit runtime resolves")) {
        Ok(resolution) => CommandOutcome::success(format!(
            "task queued: runtime: {} objective: confirmed",
            runtime_label(&resolution.runtime)
        )),
        Err(error) => CommandOutcome::error(format!("error: {}", error.message)),
    }
}

fn apply_runtime_segment(runtime: &Runtime) -> String {
    format!(" runtime: {}", runtime_label(runtime))
}

fn state_root() -> PathBuf {
    std::env::var_os("PATCHLANE_STATE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".patchlane"))
        .join("tasks")
}

fn runtime_label(runtime: &Runtime) -> &'static str {
    match runtime {
        Runtime::Codex => "codex",
        Runtime::Claude => "claude",
    }
}
