use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrchestratorState {
    Queued,
    Running,
    WaitingForInput,
    WaitingForApproval,
    InReview,
    Done,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckpointStatus {
    Pending,
    Approved,
    Rejected,
    Expired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentEventType {
    Start,
    Phase,
    WaitingInput,
    WaitingApproval,
    Artifact,
    ReviewStart,
    ReviewPass,
    ReviewFail,
    Done,
    Fail,
    CheckpointDecision,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactType {
    Spec,
    Plan,
    Review,
    Summary,
    Log,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedTaskRun {
    pub run_id: String,
    pub objective: String,
    pub runtime: String,
    pub current_phase: String,
    pub overall_state: OrchestratorState,
    pub blocking_reason: Option<String>,
    pub workspace_root: String,
    pub workspace_policy: String,
    pub default_isolation: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedAgent {
    pub agent_id: String,
    pub run_id: String,
    #[serde(default)]
    pub parent_agent_id: Option<String>,
    pub role: String,
    pub current_phase: String,
    pub current_state: OrchestratorState,
    pub runtime: String,
    pub workspace_path: String,
    #[serde(default)]
    pub pid: Option<u32>,
    #[serde(default)]
    pub related_artifact_ids: Vec<String>,
    pub stdout_log: String,
    pub stderr_log: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedCheckpoint {
    pub checkpoint_id: String,
    pub run_id: String,
    pub phase: String,
    pub target_kind: String,
    pub target_ref: String,
    pub requested_by: String,
    pub status: CheckpointStatus,
    pub prompt_text: String,
    #[serde(default)]
    pub response: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedArtifact {
    pub artifact_id: String,
    pub run_id: String,
    pub producing_agent_id: String,
    pub artifact_type: ArtifactType,
    pub path: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedTaskEvent {
    pub event_id: String,
    pub run_id: String,
    #[serde(default)]
    pub agent_id: Option<String>,
    pub event_type: AgentEventType,
    pub payload_summary: String,
    pub timestamp: String,
}

#[derive(Debug, Clone)]
pub struct TaskSnapshot {
    pub run: PersistedTaskRun,
    pub agents: Vec<PersistedAgent>,
    pub checkpoints: Vec<PersistedCheckpoint>,
    pub artifacts: Vec<PersistedArtifact>,
    pub events: Vec<PersistedTaskEvent>,
}
