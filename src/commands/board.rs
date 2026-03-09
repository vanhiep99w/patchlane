use crate::commands::CommandOutcome;

pub fn execute() -> CommandOutcome {
    CommandOutcome::success(
        "\
Board
  active runs: 1
  blocked shards: 1
  merge queue: 1 ready, 1 pending

Active Runs
  run-001 running 3 shards objective: Land compact status and watch surfaces

Blocked Shards
  shard-03 waiting on maintainer review

Next
  use `patchlane swarm status` for a single run or `patchlane swarm web` for a broader overview"
            .to_owned(),
    )
}
