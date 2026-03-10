use crate::commands::CommandOutcome;
use crate::events::run_events::fixture_status_snapshot;
use crate::renderers::status_renderer::render_status_snapshot;

pub fn execute() -> CommandOutcome {
    let snapshot = fixture_status_snapshot();
    CommandOutcome::success(render_status_snapshot(&snapshot))
}
