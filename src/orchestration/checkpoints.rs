use crate::orchestration::model::{CheckpointStatus, PersistedCheckpoint};

const APPROVAL_PROMPT: &str = "Approve? [y/n]";

pub fn build_phase_checkpoints(run_id: &str, requested_by: &str) -> Vec<PersistedCheckpoint> {
    [
        ("after-brainstorming", "artifact", "artifact-spec"),
        ("after-writing-plans", "artifact", "artifact-plan"),
        ("request-more-information", "agent", "agent-input"),
        ("review-intervention", "agent", "agent-review"),
        ("before-branch-finishing", "phase", "finish-branch"),
    ]
    .into_iter()
    .map(|(phase, target_kind, target_ref)| PersistedCheckpoint {
        checkpoint_id: format!("{run_id}-{phase}"),
        run_id: run_id.to_owned(),
        phase: phase.to_owned(),
        target_kind: target_kind.to_owned(),
        target_ref: target_ref.to_owned(),
        requested_by: requested_by.to_owned(),
        status: CheckpointStatus::Pending,
        prompt_text: APPROVAL_PROMPT.to_owned(),
        response: None,
        note: None,
        created_at: "2026-03-10T00:00:00Z".to_owned(),
        updated_at: "2026-03-10T00:00:00Z".to_owned(),
    })
    .collect()
}
