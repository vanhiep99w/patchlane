use crate::commands::agent_event::{record_agent_event, AgentEventInput};
use crate::orchestration::model::{
    AgentEventType, ArtifactType, OrchestratorState, PersistedAgent, PersistedArtifact,
};
use crate::orchestration::store::{write_agent, write_artifact};
use std::io;
use std::path::{Path, PathBuf};

pub struct AgentWrapper {
    run_dir: PathBuf,
    agent: PersistedAgent,
}

impl AgentWrapper {
    pub fn new(run_dir: PathBuf, agent: PersistedAgent) -> Self {
        Self { run_dir, agent }
    }

    pub fn start(&mut self) -> io::Result<()> {
        self.emit_state(
            AgentEventType::Start,
            Some("started"),
            Some(OrchestratorState::Running),
        )
    }

    pub fn phase(&mut self, phase: &str) -> io::Result<()> {
        self.agent.current_phase = phase.to_owned();
        self.emit_state(
            AgentEventType::Phase,
            Some(phase),
            Some(OrchestratorState::Running),
        )
    }

    pub fn artifact(&mut self, artifact_type: ArtifactType, path: &str) -> io::Result<()> {
        let artifact = PersistedArtifact {
            artifact_id: format!("artifact-{}-{}", self.agent.agent_id, self.agent.related_artifact_ids.len()),
            run_id: self.agent.run_id.clone(),
            producing_agent_id: self.agent.agent_id.clone(),
            artifact_type,
            path: path.to_owned(),
            created_at: timestamp_now(),
        };
        self.agent.related_artifact_ids.push(artifact.artifact_id.clone());
        write_artifact(&self.run_dir, &artifact)?;
        write_agent(&self.run_dir, &self.agent)?;
        self.emit_event(
            AgentEventType::Artifact,
            &format!("{}|{}", artifact_type_label(artifact_type), path),
        )
    }

    pub fn waiting_approval(&mut self, checkpoint_id: &str, prompt: &str) -> io::Result<()> {
        self.agent.current_state = OrchestratorState::WaitingForApproval;
        self.emit_state(
            AgentEventType::WaitingApproval,
            Some(&format!("{checkpoint_id}|{prompt}")),
            Some(OrchestratorState::WaitingForApproval),
        )
    }

    pub fn waiting_input(&mut self, prompt: &str) -> io::Result<()> {
        self.agent.current_state = OrchestratorState::WaitingForInput;
        self.emit_state(
            AgentEventType::WaitingInput,
            Some(prompt),
            Some(OrchestratorState::WaitingForInput),
        )
    }

    pub fn review_start(&mut self, summary: &str) -> io::Result<()> {
        self.agent.current_state = OrchestratorState::InReview;
        self.emit_state(
            AgentEventType::ReviewStart,
            Some(summary),
            Some(OrchestratorState::InReview),
        )
    }

    pub fn review_pass(&mut self, summary: &str) -> io::Result<()> {
        self.emit_event(AgentEventType::ReviewPass, summary)
    }

    pub fn review_fail(&mut self, summary: &str) -> io::Result<()> {
        self.agent.current_state = OrchestratorState::Failed;
        self.emit_state(
            AgentEventType::ReviewFail,
            Some(summary),
            Some(OrchestratorState::Failed),
        )
    }

    pub fn done(&mut self, summary: &str) -> io::Result<()> {
        self.agent.current_state = OrchestratorState::Done;
        self.emit_state(
            AgentEventType::Done,
            Some(summary),
            Some(OrchestratorState::Done),
        )
    }

    pub fn fail(&mut self, message: &str) -> io::Result<()> {
        self.agent.current_state = OrchestratorState::Failed;
        self.emit_state(
            AgentEventType::Fail,
            Some(message),
            Some(OrchestratorState::Failed),
        )
    }

    fn emit_state(
        &mut self,
        event_type: AgentEventType,
        payload: Option<&str>,
        state: Option<OrchestratorState>,
    ) -> io::Result<()> {
        if let Some(state) = state {
            self.agent.current_state = state;
        }
        self.agent.updated_at = timestamp_now();
        write_agent(&self.run_dir, &self.agent)?;
        self.emit_event(event_type, payload.unwrap_or(self.agent.current_phase.as_str()))
    }

    fn emit_event(&self, event_type: AgentEventType, payload: &str) -> io::Result<()> {
        record_agent_event(AgentEventInput {
            run_dir: Path::new(&self.run_dir).to_path_buf(),
            run_id: self.agent.run_id.clone(),
            agent_id: self.agent.agent_id.clone(),
            event_type,
            message: payload.to_owned(),
            timestamp: timestamp_now(),
        })
    }
}

fn timestamp_now() -> String {
    "2026-03-10T00:00:00Z".to_owned()
}

fn artifact_type_label(artifact_type: ArtifactType) -> &'static str {
    match artifact_type {
        ArtifactType::Spec => "spec",
        ArtifactType::Plan => "plan",
        ArtifactType::Review => "review",
        ArtifactType::Summary => "summary",
        ArtifactType::Log => "log",
    }
}
