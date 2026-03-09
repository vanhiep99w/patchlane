use crate::events::run_events::EventLine;

pub fn render_watch_events(events: &[EventLine]) -> String {
    events
        .iter()
        .map(|event| format!("{} {}", event.timestamp, event.message))
        .collect::<Vec<_>>()
        .join("\n")
}
