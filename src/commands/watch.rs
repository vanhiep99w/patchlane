use crate::commands::CommandOutcome;
use crate::events::run_events::fixture_watch_events;
use crate::renderers::watch_renderer::render_watch_events;

pub fn execute() -> CommandOutcome {
    let events = fixture_watch_events();
    CommandOutcome::success(render_watch_events(&events))
}
