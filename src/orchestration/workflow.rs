use crate::cli::{Runtime, TaskCommand};
use crate::orchestration::agent_wrapper::AgentWrapper;
use crate::orchestration::checkpoints::build_phase_checkpoints;
use crate::orchestration::model::{
    CheckpointStatus, OrchestratorState, PersistedTaskRun,
};
use crate::orchestration::phases::{
    brainstorming_agent, implementation_agent, plan_artifact, planning_agent, spec_artifact,
};
use crate::orchestration::store::{
    append_task_event, create_task_run, load_task_snapshot, write_agent, write_checkpoint,
    write_task_run,
};
use crate::runtime::launcher::{launch_agent, AgentLaunchRequest};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn execute_task_workflow(root: &Path, command: TaskCommand) -> io::Result<PathBuf> {
    let run_id = generate_run_id();
    let mut run = PersistedTaskRun {
        run_id: run_id.clone(),
        objective: command.objective,
        runtime: runtime_label(command.runtime.as_ref().unwrap_or(&Runtime::Codex)).to_owned(),
        current_phase: "brainstorming".to_owned(),
        overall_state: OrchestratorState::Running,
        blocking_reason: None,
        workspace_root: "workspace".to_owned(),
        workspace_policy: "isolated_by_default".to_owned(),
        default_isolation: true,
        created_at: "2026-03-10T00:00:00Z".to_owned(),
        updated_at: "2026-03-10T00:00:00Z".to_owned(),
    };

    let run_dir = create_task_run(root, &run)?;

    let brainstorm = brainstorming_agent(&run_id);
    let planner = planning_agent(&run_id);
    let implementer = implementation_agent(&run_id);
    write_agent(&run_dir, &brainstorm)?;
    write_agent(&run_dir, &planner)?;
    write_agent(&run_dir, &implementer)?;

    ensure_log_files(&run_dir, &[&brainstorm, &planner, &implementer])?;

    let mut brainstorm_wrapper = AgentWrapper::new(run_dir.clone(), brainstorm);
    brainstorm_wrapper.start()?;
    brainstorm_wrapper.phase("brainstorming")?;
    let spec = spec_artifact(&run_id);
    brainstorm_wrapper.artifact(spec.artifact_type, &spec.path)?;
    brainstorm_wrapper.done("brainstorming complete")?;

    let mut planner_wrapper = AgentWrapper::new(run_dir.clone(), planner);
    planner_wrapper.start()?;
    planner_wrapper.phase("writing-plans")?;
    let plan = plan_artifact(&run_id);
    planner_wrapper.artifact(plan.artifact_type, &plan.path)?;

    for mut checkpoint in build_phase_checkpoints(&run_id, "agent-plan")
        .into_iter()
        .filter(|checkpoint| {
            checkpoint.phase == "after-brainstorming" || checkpoint.phase == "after-writing-plans"
        })
    {
        checkpoint.status = if checkpoint.phase == "after-brainstorming" { CheckpointStatus::Approved } else { CheckpointStatus::Pending };
        write_checkpoint(&run_dir, &checkpoint)?;
        if checkpoint.phase == "after-writing-plans" {
            planner_wrapper.waiting_approval(&checkpoint.checkpoint_id, &checkpoint.prompt_text)?;
        }
    }

    let mut implementer_wrapper = AgentWrapper::new(run_dir.clone(), implementer);
    implementer_wrapper.start()?;
    implementer_wrapper.phase("subagent-driven-development")?;
    maybe_launch_high_level_agents(
        &run_dir,
        &run_id,
        resolved_runtime(command.runtime.as_ref()),
    )?;

    run.current_phase = "finishing-a-development-branch".to_owned();
    run.overall_state = OrchestratorState::WaitingForApproval;
    run.blocking_reason = Some("approval required".to_owned());
    write_task_run(&run_dir, &run)?;
    append_task_event(
        &run_dir,
        &crate::orchestration::model::PersistedTaskEvent {
            event_id: "event-run-finish-phase".to_owned(),
            run_id: run_id.clone(),
            agent_id: None,
            event_type: crate::orchestration::model::AgentEventType::Phase,
            payload_summary: "finishing-a-development-branch".to_owned(),
            timestamp: "2026-03-10T00:00:00Z".to_owned(),
        },
    )?;

    Ok(run_dir)
}

fn maybe_launch_high_level_agents(
    run_dir: &Path,
    run_id: &str,
    runtime: Runtime,
) -> io::Result<()> {
    if std::env::var("PATCHLANE_TEST_RUNTIME_MODE").is_ok() {
        let request = AgentLaunchRequest {
            runtime,
            run_id: run_id.to_owned(),
            role: "agent-implement".to_owned(),
            prompt: "Report orchestration progress by emitting a phase event with the provided contract.".to_owned(),
            workspace: run_dir.join("workspace-agent-implement"),
            logs_dir: run_dir.join("logs"),
            run_dir: run_dir.to_path_buf(),
        };
        if let Ok(launch) = launch_agent(&request) {
            wait_for_launched_agent_effect(run_dir, &launch.stdout_log, run_id)?;
        }
    }
    Ok(())
}

fn wait_for_launched_agent_effect(
    run_dir: &Path,
    stdout_log: &Path,
    run_id: &str,
) -> io::Result<()> {
    let expected_marker = format!("launcher-contract run_id={run_id}");
    let expected_event = format!("launcher-contract:{run_id}");
    for _ in 0..50 {
        let stdout_ready = fs::read_to_string(stdout_log)
            .map(|contents| contents.contains(&expected_marker))
            .unwrap_or(false);
        let event_ready = load_task_snapshot(run_dir)
            .map(|snapshot| {
                snapshot.events.iter().any(|event| {
                    event.agent_id.as_deref() == Some("agent-implement")
                        && event.payload_summary == expected_event
                })
            })
            .unwrap_or(false);
        if stdout_ready && event_ready {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(20));
    }
    Err(io::Error::other(
        "timed out waiting for launched agent reporting contract",
    ))
}

fn ensure_log_files(run_dir: &Path, agents: &[&crate::orchestration::model::PersistedAgent]) -> io::Result<()> {
    fs::create_dir_all(run_dir.join("logs"))?;
    for agent in agents {
        fs::write(run_dir.join("logs").join(&agent.stdout_log), b"")?;
        fs::write(run_dir.join("logs").join(&agent.stderr_log), b"")?;
    }
    Ok(())
}

fn resolved_runtime(runtime: Option<&Runtime>) -> Runtime {
    runtime.cloned().unwrap_or(Runtime::Codex)
}

fn generate_run_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_nanos();
    format!("run-{nanos}")
}

fn runtime_label(runtime: &Runtime) -> &'static str {
    match runtime {
        Runtime::Codex => "codex",
        Runtime::Claude => "claude",
    }
}
