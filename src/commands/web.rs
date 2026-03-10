use crate::commands::CommandOutcome;

pub fn execute() -> CommandOutcome {
    CommandOutcome::success(
        "\
Web
  read-mostly overview entry point

URL
  http://127.0.0.1:4040/overview

Focus
  active runs, blockers, placement, and merge queue summaries

Next
  keep `patchlane swarm watch` in the terminal for operational events"
            .to_owned(),
    )
}
