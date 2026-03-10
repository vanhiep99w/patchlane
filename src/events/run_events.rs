use crate::orchestration::model::{
    AgentEventType, CheckpointStatus, OrchestratorState, PersistedTaskEvent, TaskSnapshot,
};
use crate::store::run_store::{PersistedEvent, PersistedRun, PersistedShard};

pub struct StatusSnapshot {
    pub run: RunSnapshot,
    pub view: StatusView,
    pub agents: Vec<AgentSnapshot>,
    pub shards: Vec<ShardSnapshot>,
    pub blockers: Vec<String>,
    pub latest_event: EventLine,
    pub suggested_next_command: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StatusView {
    Agents,
    Shards,
}

pub struct RunSnapshot {
    pub id: String,
    pub runtime: String,
    pub objective: String,
    pub phase: Option<String>,
    pub state: String,
}

pub struct AgentSnapshot {
    pub id: String,
    pub role: String,
    pub phase: String,
    pub state: String,
    pub runtime: String,
    pub detail: String,
}

pub struct ShardSnapshot {
    pub id: String,
    pub runtime: String,
    pub pid: String,
    pub state: String,
    pub workspace: String,
    pub detail: String,
}

pub struct BoardSnapshot {
    pub active_runs: usize,
    pub blocked_agents: usize,
    pub runs: Vec<BoardRunLine>,
    pub blocked: Vec<String>,
}

pub struct BoardRunLine {
    pub id: String,
    pub state: String,
    pub agent_count: usize,
    pub objective: String,
}

pub struct EventLine {
    pub timestamp: String,
    pub message: String,
}

pub fn derive_status_snapshot(snapshot: TaskSnapshot) -> StatusSnapshot {
    let latest_event = snapshot
        .events
        .last()
        .map(event_line)
        .unwrap_or_else(|| EventLine {
            timestamp: "none".to_owned(),
            message: "no recorded events".to_owned(),
        });
    let blockers = collect_blockers(&snapshot);

    StatusSnapshot {
        run: RunSnapshot {
            id: snapshot.run.run_id.clone(),
            runtime: snapshot.run.runtime.clone(),
            objective: snapshot.run.objective.clone(),
            phase: Some(snapshot.run.current_phase.clone()),
            state: state_label(snapshot.run.overall_state).to_owned(),
        },
        view: StatusView::Agents,
        agents: snapshot
            .agents
            .iter()
            .map(|agent| AgentSnapshot {
                id: agent.agent_id.clone(),
                role: agent.role.clone(),
                phase: agent.current_phase.clone(),
                state: state_label(agent.current_state).to_owned(),
                runtime: agent.runtime.clone(),
                detail: agent_detail(agent.agent_id.as_str(), &snapshot),
            })
            .collect(),
        shards: Vec::new(),
        blockers,
        latest_event,
        suggested_next_command: "patchlane swarm watch".to_owned(),
    }
}

pub fn derive_legacy_status_snapshot(
    run: PersistedRun,
    shards: Vec<PersistedShard>,
    events: Vec<PersistedEvent>,
) -> StatusSnapshot {
    let latest_event = events.last().map(legacy_event_line).unwrap_or_else(|| EventLine {
        timestamp: "none".to_owned(),
        message: "no recorded events".to_owned(),
    });
    let blockers = collect_legacy_blockers(&shards, &events);
    let suggested_next_command = if blockers.is_empty() {
        "patchlane swarm watch".to_owned()
    } else {
        "patchlane swarm retry <shard-id>".to_owned()
    };

    StatusSnapshot {
        run: RunSnapshot {
            id: run.run_id,
            runtime: run.runtime,
            objective: run.objective,
            phase: None,
            state: legacy_run_state(&shards).to_owned(),
        },
        view: StatusView::Shards,
        agents: Vec::new(),
        shards: shards
            .iter()
            .map(|shard| ShardSnapshot {
                id: shard.shard_id.clone(),
                runtime: shard.runtime.clone(),
                pid: shard
                    .pid
                    .map(|pid| pid.to_string())
                    .unwrap_or_else(|| "-".to_owned()),
                state: shard.state.clone(),
                workspace: shard.workspace.clone(),
                detail: legacy_shard_detail(shard, &events),
            })
            .collect(),
        blockers,
        latest_event,
        suggested_next_command,
    }
}

pub fn derive_board_snapshot(snapshots: &[TaskSnapshot]) -> BoardSnapshot {
    BoardSnapshot {
        active_runs: snapshots
            .iter()
            .filter(|snapshot| {
                matches!(
                    snapshot.run.overall_state,
                    OrchestratorState::Running
                        | OrchestratorState::WaitingForApproval
                        | OrchestratorState::WaitingForInput
                        | OrchestratorState::InReview
                )
            })
            .count(),
        blocked_agents: snapshots
            .iter()
            .flat_map(|snapshot| snapshot.agents.iter())
            .filter(|agent| {
                matches!(
                    agent.current_state,
                    OrchestratorState::WaitingForApproval
                        | OrchestratorState::WaitingForInput
                        | OrchestratorState::Failed
                )
            })
            .count(),
        runs: snapshots
            .iter()
            .map(|snapshot| BoardRunLine {
                id: snapshot.run.run_id.clone(),
                state: state_label(snapshot.run.overall_state).to_owned(),
                agent_count: snapshot.agents.len(),
                objective: snapshot.run.objective.clone(),
            })
            .collect(),
        blocked: snapshots
            .iter()
            .flat_map(|snapshot| {
                snapshot.agents.iter().filter_map(|agent| {
                    matches!(
                        agent.current_state,
                        OrchestratorState::WaitingForApproval
                            | OrchestratorState::WaitingForInput
                            | OrchestratorState::Failed
                    )
                    .then(|| {
                        format!(
                            "{} {} {} {}",
                            snapshot.run.run_id,
                            agent.agent_id,
                            agent.role,
                            agent_detail(&agent.agent_id, snapshot)
                        )
                    })
                })
            })
            .collect(),
    }
}

pub fn empty_status_snapshot() -> StatusSnapshot {
    StatusSnapshot {
        run: RunSnapshot {
            id: "none".to_owned(),
            runtime: "none".to_owned(),
            objective: "no persisted runs found".to_owned(),
            phase: Some("none".to_owned()),
            state: "idle".to_owned(),
        },
        view: StatusView::Agents,
        agents: Vec::new(),
        shards: Vec::new(),
        blockers: Vec::new(),
        latest_event: EventLine {
            timestamp: "none".to_owned(),
            message: "run `patchlane task <objective>` to start a task run".to_owned(),
        },
        suggested_next_command: "patchlane task <objective>".to_owned(),
    }
}

pub fn derive_watch_events(events: Vec<PersistedTaskEvent>) -> Vec<EventLine> {
    if events.is_empty() {
        return empty_watch_events();
    }

    events
        .into_iter()
        .map(|event| event_line(&event))
        .collect()
}

pub fn derive_legacy_watch_events(events: Vec<PersistedEvent>) -> Vec<EventLine> {
    let filtered = events
        .into_iter()
        .filter(|event| !contains_transcript_noise(&event.message))
        .map(|event| legacy_event_line(&event))
        .collect::<Vec<_>>();

    if filtered.is_empty() {
        return empty_watch_events();
    }

    filtered
}

pub fn empty_watch_events() -> Vec<EventLine> {
    vec![EventLine {
        timestamp: "none".to_owned(),
        message: "no persisted runs found".to_owned(),
    }]
}

pub fn empty_board_snapshot() -> BoardSnapshot {
    BoardSnapshot {
        active_runs: 0,
        blocked_agents: 0,
        runs: Vec::new(),
        blocked: Vec::new(),
    }
}

fn collect_blockers(snapshot: &TaskSnapshot) -> Vec<String> {
    let mut blockers = Vec::new();

    if let Some(reason) = &snapshot.run.blocking_reason {
        blockers.push(format!("run {reason}"));
    }

    blockers.extend(snapshot.agents.iter().filter(|agent| {
        matches!(
            agent.current_state,
            OrchestratorState::WaitingForApproval
                | OrchestratorState::WaitingForInput
                | OrchestratorState::Failed
        )
    })
    .map(|agent| format!("{} {}", agent.agent_id, agent_detail(&agent.agent_id, snapshot))));

    blockers
}

fn collect_legacy_blockers(shards: &[PersistedShard], events: &[PersistedEvent]) -> Vec<String> {
    shards
        .iter()
        .filter(|shard| shard.state == "failed" || shard.state == "blocked")
        .map(|shard| {
            let detail = legacy_shard_detail(shard, events);
            format!("shard {} {}", shard.shard_id, detail)
        })
        .collect()
}

fn agent_detail(agent_id: &str, snapshot: &TaskSnapshot) -> String {
    if let Some(checkpoint) = snapshot
        .checkpoints
        .iter()
        .rev()
        .find(|checkpoint| checkpoint.requested_by == agent_id && checkpoint.status == CheckpointStatus::Pending)
    {
        return format!("waiting on {}", checkpoint.phase);
    }

    snapshot
        .events
        .iter()
        .rev()
        .find(|event| event.agent_id.as_deref() == Some(agent_id))
        .map(|event| event.payload_summary.clone())
        .unwrap_or_else(|| "none".to_owned())
}

fn event_line(event: &PersistedTaskEvent) -> EventLine {
    let prefix = event
        .agent_id
        .as_deref()
        .map(|agent_id| format!("[{agent_id}] "))
        .unwrap_or_default();
    EventLine {
        timestamp: event.timestamp.clone(),
        message: format!(
            "{prefix}{} {}",
            event_type_label(event.event_type),
            event.payload_summary
        ),
    }
}

fn legacy_event_line(event: &PersistedEvent) -> EventLine {
    EventLine {
        timestamp: event.timestamp.clone(),
        message: event.message.clone(),
    }
}

fn event_type_label(event_type: AgentEventType) -> &'static str {
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
}

fn state_label(state: OrchestratorState) -> &'static str {
    match state {
        OrchestratorState::Queued => "queued",
        OrchestratorState::Running => "running",
        OrchestratorState::WaitingForInput => "waiting_for_input",
        OrchestratorState::WaitingForApproval => "waiting_for_approval",
        OrchestratorState::InReview => "in_review",
        OrchestratorState::Done => "done",
        OrchestratorState::Failed => "failed",
    }
}

fn legacy_run_state(shards: &[PersistedShard]) -> &'static str {
    if shards.iter().any(|shard| shard.state == "failed") {
        "degraded"
    } else if shards.iter().any(|shard| shard.state == "blocked") {
        "blocked"
    } else if shards.iter().all(|shard| shard.state == "completed") {
        "completed"
    } else if shards
        .iter()
        .any(|shard| shard.state == "launched" || shard.state == "running")
    {
        "active"
    } else {
        "queued"
    }
}

fn legacy_shard_detail(shard: &PersistedShard, events: &[PersistedEvent]) -> String {
    if shard.state == "failed" || shard.state == "blocked" {
        return events
            .iter()
            .rev()
            .find(|event| event.shard_id.as_deref() == Some(shard.shard_id.as_str()))
            .map(|event| event.message.clone())
            .unwrap_or_else(|| "no detail recorded".to_owned());
    }

    "none".to_owned()
}

fn contains_transcript_noise(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    lower.contains("transcript") || lower.contains("assistant:") || lower.contains("user:")
}
