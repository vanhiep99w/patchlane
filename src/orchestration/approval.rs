use crate::orchestration::model::{
    AgentEventType, CheckpointStatus, OrchestratorState, PersistedCheckpoint, PersistedTaskEvent,
};
use crate::orchestration::store::{append_task_event, load_task_run, write_checkpoint, write_task_run};
use std::io;
use std::path::Path;

pub fn handle_approval_input(
    input: &str,
    checkpoint: PersistedCheckpoint,
    run_dir: &Path,
) -> io::Result<PersistedCheckpoint> {
    if !matches!(input.trim(), "y" | "Y" | "n" | "N") {
        append_task_event(
            run_dir,
            &PersistedTaskEvent {
                event_id: "event-invalid-input".to_owned(),
                run_id: checkpoint.run_id.clone(),
                agent_id: Some(checkpoint.requested_by.clone()),
                event_type: AgentEventType::WaitingInput,
                payload_summary: "invalid approval input; re-prompting for y/n".to_owned(),
                timestamp: checkpoint.updated_at.clone(),
            },
        )?;
        return Ok(checkpoint);
    }

    let mut run = load_task_run(run_dir)?;
    let updated = PersistedCheckpoint {
        status: if matches!(input.trim(), "y" | "Y") {
            CheckpointStatus::Approved
        } else {
            CheckpointStatus::Rejected
        },
        response: Some(input.trim().to_ascii_lowercase()),
        updated_at: "2026-03-10T00:00:01Z".to_owned(),
        ..checkpoint
    };

    run.overall_state = if updated.status == CheckpointStatus::Rejected {
        OrchestratorState::WaitingForInput
    } else {
        OrchestratorState::Running
    };
    run.blocking_reason = if updated.status == CheckpointStatus::Rejected {
        Some(format!("checkpoint {} rejected", updated.checkpoint_id))
    } else {
        None
    };
    run.updated_at = updated.updated_at.clone();

    write_checkpoint(run_dir, &updated)?;
    write_task_run(run_dir, &run)?;
    append_task_event(
        run_dir,
        &PersistedTaskEvent {
            event_id: format!("event-{}", updated.checkpoint_id),
            run_id: updated.run_id.clone(),
            agent_id: Some(updated.requested_by.clone()),
            event_type: AgentEventType::CheckpointDecision,
            payload_summary: format!("{} -> {:?}", updated.checkpoint_id, updated.status),
            timestamp: updated.updated_at.clone(),
        },
    )?;

    Ok(updated)
}
