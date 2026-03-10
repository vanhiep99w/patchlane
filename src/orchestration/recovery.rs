use crate::orchestration::model::{
    OrchestratorState, PersistedAgent, PersistedCheckpoint, PersistedTaskEvent, PersistedTaskRun,
};
use crate::orchestration::store::load_task_snapshot;
use std::io;
use std::path::Path;

pub struct RecoveredRunState {
    pub run: PersistedTaskRun,
    pub pending_checkpoint: Option<PersistedCheckpoint>,
    pub blocked_agents: Vec<PersistedAgent>,
    pub latest_event: Option<PersistedTaskEvent>,
}

pub fn recover_run_state(run_dir: &Path) -> io::Result<RecoveredRunState> {
    let snapshot = load_task_snapshot(run_dir)?;
    Ok(RecoveredRunState {
        run: snapshot.run.clone(),
        pending_checkpoint: snapshot
            .checkpoints
            .iter()
            .filter(|checkpoint| checkpoint.status == crate::orchestration::model::CheckpointStatus::Pending)
            .max_by(|left, right| left.updated_at.cmp(&right.updated_at))
            .cloned(),
        blocked_agents: snapshot
            .agents
            .iter()
            .filter(|agent| {
                matches!(
                    agent.current_state,
                    OrchestratorState::WaitingForInput
                        | OrchestratorState::WaitingForApproval
                        | OrchestratorState::Failed
                )
            })
            .cloned()
            .collect(),
        latest_event: snapshot.events.last().cloned(),
    })
}
