# Patchlane CLI

Patchlane is a Rust CLI project for agent-native swarm orchestration. The current repository provides a deterministic local CLI slice for the operator loop:

- start a run with `swarm run`
- inspect the latest snapshot with `swarm status`
- stream workflow events with `swarm watch`
- issue practical intervention commands such as `swarm pause run-active`
- open broader read-mostly overview surfaces with `swarm board` and `swarm web`

## Local Usage

Run the primary contract from the repository root:

```bash
cargo run -- swarm run "Land compact status and watch surfaces"
cargo run -- swarm status
cargo run -- swarm watch
cargo run -- swarm pause run-active
cargo run -- swarm board
cargo run -- swarm web
```

Intervention commands currently use deterministic fixture ids so the local contract stays stable while persistence is still placeholder-driven:

- run ids: `run-active`, `run-paused`, `run-done`, `run-stopped`
- shard ids: `shard-queued`, `shard-assigned`, `shard-running`, `shard-blocked`, `shard-failed`, `shard-done`
- merge unit ids: `merge-001`, `merge-applied`, `merge-runtime-error`

## Commands

The CLI currently exposes these command groups:

- `swarm run <OBJECTIVE>`
- `swarm status`
- `swarm watch`
- `swarm pause <TARGET_ID>`
- `swarm resume <TARGET_ID>`
- `swarm retry <SHARD_ID>`
- `swarm reassign <SHARD_ID> --runtime <codex|claude>`
- `swarm merge approve <MERGE_UNIT_ID>`
- `swarm merge reject <MERGE_UNIT_ID>`
- `swarm stop <RUN_ID>`
- `swarm board`
- `swarm web`

## Development

Run the full verification suite:

```bash
cargo test
```
