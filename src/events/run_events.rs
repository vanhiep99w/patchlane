use crate::store::run_store::{PersistedEvent, PersistedRun, PersistedShard};

pub struct StatusSnapshot {
    pub run: RunSnapshot,
    pub shards: Vec<ShardSnapshot>,
    pub blockers: Vec<String>,
    pub latest_event: EventLine,
    pub suggested_next_command: String,
}

pub struct RunSnapshot {
    pub id: String,
    pub runtime: String,
    pub objective: String,
    pub state: String,
}

pub struct ShardSnapshot {
    pub id: String,
    pub runtime: String,
    pub pid: String,
    pub state: String,
    pub workspace: String,
    pub detail: String,
}

pub struct EventLine {
    pub timestamp: String,
    pub message: String,
}

pub fn derive_status_snapshot(
    run: PersistedRun,
    shards: Vec<PersistedShard>,
    events: Vec<PersistedEvent>,
) -> StatusSnapshot {
    let latest_event = events.last().map(event_line).unwrap_or_else(|| EventLine {
        timestamp: "none".to_owned(),
        message: "no recorded events".to_owned(),
    });
    let blockers = collect_blockers(&shards, &events);
    let suggested_next_command = suggested_next_command(&shards);

    StatusSnapshot {
        run: RunSnapshot {
            id: run.run_id,
            runtime: run.runtime,
            objective: run.objective,
            state: derive_run_state(&shards),
        },
        shards: shards.into_iter().map(|shard| shard_snapshot(&shard, &events)).collect(),
        blockers,
        latest_event,
        suggested_next_command,
    }
}

pub fn empty_status_snapshot() -> StatusSnapshot {
    StatusSnapshot {
        run: RunSnapshot {
            id: "none".to_owned(),
            runtime: "none".to_owned(),
            objective: "no persisted runs found".to_owned(),
            state: "idle".to_owned(),
        },
        shards: Vec::new(),
        blockers: Vec::new(),
        latest_event: EventLine {
            timestamp: "none".to_owned(),
            message: "run `patchlane swarm run --runtime <codex|claude> <objective>` to start a run"
                .to_owned(),
        },
        suggested_next_command: "patchlane swarm run --runtime <codex|claude> <objective>"
            .to_owned(),
    }
}

fn derive_run_state(shards: &[PersistedShard]) -> String {
    if shards.iter().any(|shard| shard.state == "failed") {
        "degraded".to_owned()
    } else if shards.iter().any(|shard| shard.state == "blocked") {
        "blocked".to_owned()
    } else if shards.iter().all(|shard| shard.state == "completed") {
        "completed".to_owned()
    } else if shards.iter().any(|shard| shard.state == "launched") {
        "active".to_owned()
    } else {
        "queued".to_owned()
    }
}

pub fn fixture_watch_events() -> Vec<EventLine> {
    crate::workflow::superpowers_contract::fixture_stage_event_lines()
}

pub fn derive_watch_events(events: Vec<PersistedEvent>) -> Vec<EventLine> {
    let filtered = events
        .into_iter()
        .filter(|event| !contains_transcript_noise(&event.message))
        .map(|event| event_line(&event))
        .collect::<Vec<_>>();

    if filtered.is_empty() {
        vec![EventLine {
            timestamp: "none".to_owned(),
            message: "no recorded events".to_owned(),
        }]
    } else {
        filtered
    }
}

pub fn empty_watch_events() -> Vec<EventLine> {
    vec![EventLine {
        timestamp: "none".to_owned(),
        message: "no persisted runs found".to_owned(),
    }]
}

fn shard_snapshot(shard: &PersistedShard, events: &[PersistedEvent]) -> ShardSnapshot {
    ShardSnapshot {
        id: shard.shard_id.clone(),
        runtime: shard.runtime.clone(),
        pid: shard
            .pid
            .map(|pid| pid.to_string())
            .unwrap_or_else(|| "-".to_owned()),
        state: shard.state.clone(),
        workspace: shard.workspace.clone(),
        detail: shard_detail(shard, events),
    }
}

fn shard_detail(shard: &PersistedShard, events: &[PersistedEvent]) -> String {
    if shard.state == "failed" || shard.state == "blocked" {
        return find_latest_shard_event(events, &shard.shard_id)
            .map(|event| event.message.clone())
            .unwrap_or_else(|| "no detail recorded".to_owned());
    }

    "none".to_owned()
}

fn collect_blockers(shards: &[PersistedShard], events: &[PersistedEvent]) -> Vec<String> {
    shards
        .iter()
        .filter(|shard| shard.state == "failed" || shard.state == "blocked")
        .map(|shard| {
            let detail = find_latest_shard_event(events, &shard.shard_id)
                .map(|event| event.message.clone())
                .unwrap_or_else(|| "no detail recorded".to_owned());
            format!("shard {} {}", shard.shard_id, detail)
        })
        .collect()
}

fn suggested_next_command(shards: &[PersistedShard]) -> String {
    if shards
        .iter()
        .any(|shard| shard.state == "failed" || shard.state == "blocked")
    {
        "patchlane swarm retry <shard-id>".to_owned()
    } else {
        "patchlane swarm watch".to_owned()
    }
}

fn event_line(event: &PersistedEvent) -> EventLine {
    EventLine {
        timestamp: event.timestamp.clone(),
        message: event.message.clone(),
    }
}

fn contains_transcript_noise(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    lower.contains("transcript") || lower.contains("assistant:") || lower.contains("user:")
}

fn find_latest_shard_event<'a>(
    events: &'a [PersistedEvent],
    shard_id: &str,
) -> Option<&'a PersistedEvent> {
    events
        .iter()
        .rev()
        .find(|event| event.shard_id.as_deref() == Some(shard_id))
}
