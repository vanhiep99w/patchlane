use crate::orchestration::model::{ArtifactType, PersistedAgent, PersistedArtifact};

pub fn brainstorming_agent(run_id: &str) -> PersistedAgent {
    persisted_agent(run_id, "agent-brainstorm", "brainstorming")
}

pub fn planning_agent(run_id: &str) -> PersistedAgent {
    persisted_agent(run_id, "agent-plan", "writing-plans")
}

pub fn implementation_agent(run_id: &str) -> PersistedAgent {
    persisted_agent(run_id, "agent-implement", "subagent-driven-development")
}

pub fn spec_artifact(run_id: &str) -> PersistedArtifact {
    PersistedArtifact {
        artifact_id: format!("artifact-{run_id}-spec"),
        run_id: run_id.to_owned(),
        producing_agent_id: "agent-brainstorm".to_owned(),
        artifact_type: ArtifactType::Spec,
        path: "spec.md".to_owned(),
        created_at: "2026-03-10T00:00:00Z".to_owned(),
    }
}

pub fn plan_artifact(run_id: &str) -> PersistedArtifact {
    PersistedArtifact {
        artifact_id: format!("artifact-{run_id}-plan"),
        run_id: run_id.to_owned(),
        producing_agent_id: "agent-plan".to_owned(),
        artifact_type: ArtifactType::Plan,
        path: "plan.md".to_owned(),
        created_at: "2026-03-10T00:00:00Z".to_owned(),
    }
}

fn persisted_agent(run_id: &str, agent_id: &str, role: &str) -> PersistedAgent {
    PersistedAgent {
        agent_id: agent_id.to_owned(),
        run_id: run_id.to_owned(),
        parent_agent_id: None,
        role: role.to_owned(),
        current_phase: role.to_owned(),
        current_state: crate::orchestration::model::OrchestratorState::Queued,
        runtime: "codex".to_owned(),
        workspace_path: "workspace".to_owned(),
        pid: None,
        related_artifact_ids: Vec::new(),
        stdout_log: format!("{agent_id}-stdout.log"),
        stderr_log: format!("{agent_id}-stderr.log"),
        created_at: "2026-03-10T00:00:00Z".to_owned(),
        updated_at: "2026-03-10T00:00:00Z".to_owned(),
    }
}
