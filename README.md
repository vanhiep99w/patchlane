# Patchlane CLI

Patchlane is a Rust CLI project for agent-native swarm orchestration. The current repository provides a deterministic local CLI slice for the operator loop:

- start a run with `swarm run`
- inspect the latest snapshot with `swarm status`
- stream workflow events with `swarm watch`
- issue practical intervention commands such as `swarm pause run-active`
- open broader read-mostly overview surfaces with `swarm board` and `swarm web`

## Runtime Prerequisites

Patchlane launches local worker CLIs directly from this machine. Install at least one of these before running a real swarm:

- codex CLI for `--runtime codex`
- claude CLI for `--runtime claude`

If you want persisted run state outside the current repository, set `PATCHLANE_STATE_ROOT` to a writable directory. Otherwise Patchlane stores local state under `.patchlane/` in the current working directory.

## Local Usage

Run the primary contract from the repository root:

```bash
cargo run -- swarm run --runtime codex "Land compact status and watch surfaces"
cargo run -- swarm run --runtime claude "Land compact status and watch surfaces"
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

- `swarm run --runtime <codex|claude> <OBJECTIVE>`
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

## Local Artifacts

Each run creates a directory under `PATCHLANE_STATE_ROOT` or `.patchlane/` with:

- `run.json` for top-level run metadata
- `shard-<id>.json` for shard runtime, pid, workspace, and state
- `events.jsonl` for lifecycle events consumed by `swarm watch`
- `shard-<id>-attempts.json` when `swarm retry <shard-id>` records retry history
- `logs/` with per-shard stdout/stderr log files

## Manual QA

Use a real local runtime to verify the operator loop:

```bash
cargo run -- swarm run --runtime codex "Verify local worker launch"
cargo run -- swarm status
cargo run -- swarm watch
```

Then inspect `.patchlane/` (or `PATCHLANE_STATE_ROOT`) to confirm a run directory was created with `run.json`, shard files, `events.jsonl`, and `logs/`. If a shard fails, retry it with `cargo run -- swarm retry <shard-id>` and confirm a `shard-<id>-attempts.json` file appears.

## Development

Run the full verification suite:

```bash
cargo test
```
