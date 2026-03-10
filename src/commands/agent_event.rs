use crate::cli::AgentEventCommand;
use crate::commands::CommandOutcome;
use crate::orchestration::model::{
    AgentEventType, ArtifactType, CheckpointStatus, OrchestratorState, PersistedArtifact,
    PersistedTaskEvent,
};
use crate::orchestration::store::{
    append_task_event, load_task_run, load_task_snapshot, write_agent, write_artifact,
    write_checkpoint, write_task_run,
};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct AgentEventInput {
    pub run_dir: PathBuf,
    pub run_id: String,
    pub agent_id: String,
    pub event_type: AgentEventType,
    pub message: String,
    pub timestamp: String,
}

pub fn execute(command: AgentEventCommand) -> CommandOutcome {
    let event_type = match command.event_type.as_str() {
        "start" => AgentEventType::Start,
        "phase" => AgentEventType::Phase,
        "waiting-input" => AgentEventType::WaitingInput,
        "waiting-approval" => AgentEventType::WaitingApproval,
        "artifact" => AgentEventType::Artifact,
        "review-start" => AgentEventType::ReviewStart,
        "review-pass" => AgentEventType::ReviewPass,
        "review-fail" => AgentEventType::ReviewFail,
        "done" => AgentEventType::Done,
        "fail" => AgentEventType::Fail,
        _ => {
            return CommandOutcome::error(format!(
                "error: unsupported agent event type {}",
                command.event_type
            ))
        }
    };

    let input = AgentEventInput {
        run_dir: PathBuf::from(command.run_dir),
        run_id: command.run_id,
        agent_id: command.agent_id,
        event_type,
        message: command.message,
        timestamp: "2026-03-10T00:00:00Z".to_owned(),
    };

    match record_agent_event(input) {
        Ok(()) => CommandOutcome::success("agent event recorded".to_owned()),
        Err(error) => CommandOutcome::error(format!("error: {error}")),
    }
}

pub fn record_agent_event(input: AgentEventInput) -> std::io::Result<()> {
    let event = PersistedTaskEvent {
        event_id: format!("event-{}-{:?}", input.agent_id, input.event_type),
        run_id: input.run_id.clone(),
        agent_id: Some(input.agent_id.clone()),
        event_type: input.event_type,
        payload_summary: input.message.clone(),
        timestamp: input.timestamp.clone(),
    };

    persist_event_side_effects(&input.run_dir, &input, &event)?;
    append_task_event(&input.run_dir, &event)
}

fn persist_event_side_effects(
    run_dir: &PathBuf,
    input: &AgentEventInput,
    event: &PersistedTaskEvent,
) -> std::io::Result<()> {
    let snapshot = load_task_snapshot(run_dir)?;
    if let Some(mut agent) = snapshot
        .agents
        .into_iter()
        .find(|agent| agent.agent_id == input.agent_id)
    {
        match event.event_type {
            AgentEventType::Start => {
                agent.current_state = OrchestratorState::Running;
            }
            AgentEventType::Phase => {
                agent.current_phase = input.message.clone();
                agent.current_state = OrchestratorState::Running;
            }
            AgentEventType::WaitingInput => {
                agent.current_state = OrchestratorState::WaitingForInput;
            }
            AgentEventType::WaitingApproval => {
                agent.current_state = OrchestratorState::WaitingForApproval;
                let (checkpoint_id, prompt_text) = parse_waiting_approval_message(&input.message);
                if let Some(mut checkpoint) = load_task_snapshot(run_dir)?
                    .checkpoints
                    .into_iter()
                    .find(|checkpoint| checkpoint.checkpoint_id == checkpoint_id)
                {
                    checkpoint.status = CheckpointStatus::Pending;
                    checkpoint.prompt_text = prompt_text;
                    write_checkpoint(run_dir, &checkpoint)?;
                }
            }
            AgentEventType::Artifact => {
                let (artifact_type, path) = parse_artifact_message(&input.message);
                let artifact = PersistedArtifact {
                    artifact_id: format!("artifact-{}-{}", input.agent_id, agent.related_artifact_ids.len()),
                    run_id: input.run_id.clone(),
                    producing_agent_id: input.agent_id.clone(),
                    artifact_type,
                    path: path.clone(),
                    created_at: event.timestamp.clone(),
                };
                agent.related_artifact_ids.push(artifact.artifact_id.clone());
                write_artifact(run_dir, &artifact)?;
            }
            AgentEventType::ReviewStart => {
                agent.current_state = OrchestratorState::InReview;
            }
            AgentEventType::ReviewPass => {}
            AgentEventType::ReviewFail | AgentEventType::Fail => {
                agent.current_state = OrchestratorState::Failed;
            }
            AgentEventType::Done => {
                agent.current_state = OrchestratorState::Done;
            }
            AgentEventType::CheckpointDecision => {}
        }
        agent.updated_at = event.timestamp.clone();
        write_agent(run_dir, &agent)?;
    }

    let mut run = load_task_run(run_dir)?;
    match event.event_type {
        AgentEventType::WaitingApproval => {
            run.overall_state = OrchestratorState::WaitingForApproval;
            run.blocking_reason = Some(input.message.clone());
        }
        AgentEventType::WaitingInput => {
            run.overall_state = OrchestratorState::WaitingForInput;
            run.blocking_reason = Some(input.message.clone());
        }
        AgentEventType::Fail => {
            run.overall_state = OrchestratorState::Failed;
            run.blocking_reason = Some(input.message.clone());
        }
        AgentEventType::Done => {
            run.overall_state = OrchestratorState::Done;
            run.blocking_reason = None;
        }
        _ => {}
    }
    run.updated_at = event.timestamp.clone();
    write_task_run(run_dir, &run)
}

fn parse_artifact_message(message: &str) -> (ArtifactType, String) {
    let mut parts = message.splitn(2, '|');
    let kind = parts.next().unwrap_or("summary");
    let path = parts.next().unwrap_or(message).to_owned();
    let artifact_type = match kind {
        "spec" => ArtifactType::Spec,
        "plan" => ArtifactType::Plan,
        "review" => ArtifactType::Review,
        "log" => ArtifactType::Log,
        _ => ArtifactType::Summary,
    };
    (artifact_type, path)
}

fn parse_waiting_approval_message(message: &str) -> (String, String) {
    let mut parts = message.splitn(2, '|');
    let checkpoint_id = parts.next().unwrap_or(message).to_owned();
    let prompt = parts.next().unwrap_or("Approve? [y/n]").to_owned();
    (checkpoint_id, prompt)
}
