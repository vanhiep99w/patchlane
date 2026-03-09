# Patchlane CLI

Patchlane is a Rust CLI project for agent-native swarm orchestration.

The current repository state is only a bootstrap baseline. Today it provides:

- a library entrypoint that exposes the bootstrap banner
- a thin CLI binary wired with `clap` whose help output explicitly states that planned swarm commands are not implemented yet
- initial design and implementation plan documents under `docs/plans/`

Planned future commands include `swarm run`, `status`, `watch`, intervention flows, and broader board/web views, but those are not part of Task 1.

## Development

```bash
. "$HOME/.cargo/env" && cargo test bootstrap
```
