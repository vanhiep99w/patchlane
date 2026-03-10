use crate::orchestration::model::{
    AgentEventType, ArtifactType, OrchestratorState, TaskSnapshot,
};
use crate::tui::logs::tail_log;
use crate::tui::store::{load_runs, resolve_log_path};
use crossterm::event::KeyCode;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    RunList,
    AgentList,
    Detail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunListItem {
    pub run_id: String,
    pub objective: String,
    pub current_phase: String,
    pub current_state: String,
    pub blocker_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentRow {
    pub agent_id: String,
    pub role: String,
    pub phase: String,
    pub state: String,
    pub has_blocker: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimelineEntry {
    pub timestamp: String,
    pub event_type: String,
    pub payload_summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactEntry {
    pub artifact_type: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectedAgentDetail {
    pub current_phase: String,
    pub current_state: String,
    pub blockers: Vec<String>,
    pub timeline: Vec<TimelineEntry>,
    pub artifacts: Vec<ArtifactEntry>,
    pub stdout_log: String,
    pub stderr_log: String,
    pub stdout_tail: Vec<String>,
    pub stderr_tail: Vec<String>,
    pub stdout_error: Option<String>,
    pub stderr_error: Option<String>,
}

pub struct TuiApp {
    state_root: Option<PathBuf>,
    runs: Vec<TaskSnapshot>,
    selected_run: usize,
    selected_agent: usize,
    active_pane: Pane,
    should_quit: bool,
    selected_detail: Option<SelectedAgentDetail>,
}

impl TuiApp {
    pub fn from_snapshot(snapshot: TaskSnapshot) -> Self {
        Self::from_snapshots(vec![snapshot])
    }

    pub fn from_snapshots(mut snapshots: Vec<TaskSnapshot>) -> Self {
        snapshots.sort_by(|left, right| right.run.updated_at.cmp(&left.run.updated_at));
        let selected_agent = default_agent_index(&snapshots, 0);
        let mut app = Self {
            state_root: None,
            runs: snapshots,
            selected_run: 0,
            selected_agent,
            active_pane: Pane::RunList,
            should_quit: false,
            selected_detail: None,
        };
        app.sync_selected_detail();
        app
    }

    pub fn load_from_store(state_root: &Path) -> io::Result<Self> {
        let runs = load_runs(state_root)?;
        let selected_agent = default_agent_index(&runs, 0);
        let mut app = Self {
            state_root: Some(state_root.to_path_buf()),
            runs,
            selected_run: 0,
            selected_agent,
            active_pane: Pane::RunList,
            should_quit: false,
            selected_detail: None,
        };
        app.sync_selected_detail();
        Ok(app)
    }

    pub fn refresh(&mut self) -> io::Result<()> {
        if let Some(state_root) = self.state_root.clone() {
            let selected_run_id = self
                .selected_run()
                .map(|snapshot| snapshot.run.run_id.clone());
            let selected_agent_id = self.selected_agent().map(|agent| agent.agent_id);

            self.runs = load_runs(&state_root)?;
            self.selected_run = selected_run_id
                .as_deref()
                .and_then(|run_id| {
                    self.runs
                        .iter()
                        .position(|snapshot| snapshot.run.run_id == run_id)
                })
                .unwrap_or(0)
                .min(self.runs.len().saturating_sub(1));
            self.selected_agent = selected_agent_id
                .as_deref()
                .and_then(|agent_id| {
                    self.selected_run().and_then(|snapshot| {
                        snapshot
                            .agents
                            .iter()
                            .position(|agent| agent.agent_id == agent_id)
                    })
                })
                .unwrap_or_else(|| default_agent_index(&self.runs, self.selected_run));
            self.sync_selected_detail();
        }
        Ok(())
    }

    pub fn runs(&self) -> Vec<RunListItem> {
        self.runs
            .iter()
            .map(|snapshot| RunListItem {
                run_id: snapshot.run.run_id.clone(),
                objective: snapshot.run.objective.clone(),
                current_phase: snapshot.run.current_phase.clone(),
                current_state: state_label(snapshot.run.overall_state),
                blocker_count: self.run_blockers(snapshot).len(),
            })
            .collect()
    }

    pub fn selected_run(&self) -> Option<&TaskSnapshot> {
        self.runs.get(self.selected_run)
    }

    pub fn agent_rows(&self) -> Vec<AgentRow> {
        self.selected_run()
            .map(|snapshot| {
                snapshot
                    .agents
                    .iter()
                    .map(|agent| AgentRow {
                        agent_id: agent.agent_id.clone(),
                        role: agent.role.clone(),
                        phase: agent.current_phase.clone(),
                        state: state_label(agent.current_state),
                        has_blocker: !self.blockers_for_agent(snapshot, &agent.agent_id).is_empty(),
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn selected_agent(&self) -> Option<AgentRow> {
        self.agent_rows().into_iter().nth(self.selected_agent)
    }

    pub fn selected_agent_detail(&self) -> Option<SelectedAgentDetail> {
        self.selected_detail.clone()
    }

    pub fn active_pane(&self) -> Pane {
        self.active_pane
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn handle_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Tab => self.active_pane = next_pane(self.active_pane),
            KeyCode::Char('j') | KeyCode::Down => self.move_selection(1),
            KeyCode::Char('k') | KeyCode::Up => self.move_selection(-1),
            _ => {}
        }
    }

    pub fn run_blockers_for_selected(&self) -> Vec<String> {
        self.selected_run()
            .map(|snapshot| self.run_blockers(snapshot))
            .unwrap_or_default()
    }

    fn move_selection(&mut self, delta: isize) {
        match self.active_pane {
            Pane::RunList => {
                self.selected_run = move_index(self.selected_run, self.runs.len(), delta);
                self.selected_agent = default_agent_index(&self.runs, self.selected_run);
            }
            Pane::AgentList | Pane::Detail => {
                self.selected_agent = move_index(self.selected_agent, self.agent_rows().len(), delta);
            }
        }
        self.sync_selected_detail();
    }

    fn blockers_for_agent(&self, snapshot: &TaskSnapshot, agent_id: &str) -> Vec<String> {
        let Some(agent) = snapshot.agents.iter().find(|agent| agent.agent_id == agent_id) else {
            return Vec::new();
        };

        let mut blockers = snapshot
            .checkpoints
            .iter()
            .filter(|checkpoint| checkpoint.requested_by == agent_id)
            .filter(|checkpoint| {
                matches!(
                    checkpoint.status,
                    crate::orchestration::model::CheckpointStatus::Pending
                )
            })
            .map(|checkpoint| checkpoint.prompt_text.clone())
            .collect::<Vec<_>>();

        let expected_event_type = match agent.current_state {
            OrchestratorState::WaitingForApproval => Some(AgentEventType::WaitingApproval),
            OrchestratorState::WaitingForInput => Some(AgentEventType::WaitingInput),
            OrchestratorState::Failed => Some(AgentEventType::Fail),
            _ => None,
        };
        if let Some(expected_event_type) = expected_event_type {
            if let Some(event) = snapshot.events.iter().rev().find(|event| {
                event.agent_id.as_deref() == Some(agent_id) && event.event_type == expected_event_type
            }) {
                let payload = normalize_blocker_payload(expected_event_type, &event.payload_summary);
                if !blockers.iter().any(|blocker| blocker == &payload) {
                    blockers.push(payload);
                }
            }
        }

        blockers
    }

    fn run_blockers(&self, snapshot: &TaskSnapshot) -> Vec<String> {
        let mut blockers = Vec::new();
        if let Some(reason) = &snapshot.run.blocking_reason {
            blockers.push(reason.clone());
        }
        for agent in &snapshot.agents {
            blockers.extend(self.blockers_for_agent(snapshot, &agent.agent_id));
        }
        blockers
    }

    fn log_path_for(&self, run_id: &str, relative_path: &str) -> String {
        self.state_root
            .as_ref()
            .map(|root| resolve_log_path(root, run_id, relative_path).display().to_string())
            .unwrap_or_else(|| relative_path.to_owned())
    }

    fn sync_selected_detail(&mut self) {
        self.selected_detail = self.build_selected_agent_detail();
    }

    fn build_selected_agent_detail(&self) -> Option<SelectedAgentDetail> {
        let snapshot = self.selected_run()?;
        let agent = snapshot.agents.get(self.selected_agent)?;
        let stdout_log = self.log_path_for(&snapshot.run.run_id, &agent.stdout_log);
        let stderr_log = self.log_path_for(&snapshot.run.run_id, &agent.stderr_log);
        let (stdout_tail, stdout_error) = read_log_tail(&stdout_log);
        let (stderr_tail, stderr_error) = read_log_tail(&stderr_log);

        Some(SelectedAgentDetail {
            current_phase: agent.current_phase.clone(),
            current_state: state_label(agent.current_state),
            blockers: self.blockers_for_agent(snapshot, &agent.agent_id),
            timeline: snapshot
                .events
                .iter()
                .filter(|event| event.agent_id.as_deref() == Some(agent.agent_id.as_str()))
                .map(|event| TimelineEntry {
                    timestamp: event.timestamp.clone(),
                    event_type: event_label(event.event_type),
                    payload_summary: event.payload_summary.clone(),
                })
                .collect(),
            artifacts: snapshot
                .artifacts
                .iter()
                .filter(|artifact| artifact.producing_agent_id == agent.agent_id)
                .map(|artifact| ArtifactEntry {
                    artifact_type: artifact_label(artifact.artifact_type),
                    path: artifact.path.clone(),
                })
                .collect(),
            stdout_log,
            stderr_log,
            stdout_tail,
            stderr_tail,
            stdout_error,
            stderr_error,
        })
    }
}

fn move_index(current: usize, len: usize, delta: isize) -> usize {
    if len == 0 {
        return 0;
    }
    let next = current as isize + delta;
    next.clamp(0, len.saturating_sub(1) as isize) as usize
}

fn default_agent_index(runs: &[TaskSnapshot], selected_run: usize) -> usize {
    runs.get(selected_run)
        .and_then(|snapshot| {
            snapshot
                .agents
                .iter()
                .enumerate()
                .max_by(|left, right| left.1.updated_at.cmp(&right.1.updated_at))
                .map(|(index, _)| index)
        })
        .unwrap_or(0)
}

fn next_pane(pane: Pane) -> Pane {
    match pane {
        Pane::RunList => Pane::AgentList,
        Pane::AgentList => Pane::Detail,
        Pane::Detail => Pane::RunList,
    }
}

fn state_label(state: OrchestratorState) -> String {
    match state {
        OrchestratorState::Queued => "queued",
        OrchestratorState::Running => "running",
        OrchestratorState::WaitingForInput => "waiting_for_input",
        OrchestratorState::WaitingForApproval => "waiting_for_approval",
        OrchestratorState::InReview => "in_review",
        OrchestratorState::Done => "done",
        OrchestratorState::Failed => "failed",
    }
    .to_owned()
}

fn event_label(event_type: AgentEventType) -> String {
    match event_type {
        AgentEventType::Start => "start",
        AgentEventType::Phase => "phase",
        AgentEventType::WaitingInput => "waiting-input",
        AgentEventType::WaitingApproval => "waiting-approval",
        AgentEventType::Artifact => "artifact",
        AgentEventType::ReviewStart => "review-start",
        AgentEventType::ReviewPass => "review-pass",
        AgentEventType::ReviewFail => "review-fail",
        AgentEventType::Done => "done",
        AgentEventType::Fail => "fail",
        AgentEventType::CheckpointDecision => "checkpoint-decision",
    }
    .to_owned()
}

fn artifact_label(artifact_type: ArtifactType) -> String {
    match artifact_type {
        ArtifactType::Spec => "spec",
        ArtifactType::Plan => "plan",
        ArtifactType::Review => "review",
        ArtifactType::Summary => "summary",
        ArtifactType::Log => "log",
    }
    .to_owned()
}

fn read_log_tail(path: &str) -> (Vec<String>, Option<String>) {
    match tail_log(Path::new(path), 4) {
        Ok(lines) => (lines, None),
        Err(error) => (Vec::new(), Some(format!("log unavailable: {error}"))),
    }
}

fn normalize_blocker_payload(event_type: AgentEventType, payload: &str) -> String {
    if event_type == AgentEventType::WaitingApproval {
        payload
            .split_once('|')
            .map(|(_, prompt)| prompt.to_owned())
            .unwrap_or_else(|| payload.to_owned())
    } else {
        payload.to_owned()
    }
}
