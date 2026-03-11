# Patchlane CLI

Patchlane is a Rust CLI project for agent-native swarm orchestration. The current repository provides a deterministic local CLI slice for the operator loop:

- start a run with `swarm run`
- inspect the latest snapshot with `swarm status`
- stream workflow events with `swarm watch`
- issue practical intervention commands such as `swarm pause run-active`
- open broader read-mostly overview surfaces with `swarm board` and `swarm web`
- start an orchestrated task run with `task`
- monitor task runs and agents with the `tui` read-only observability surface

## Runtime Prerequisites

Patchlane launches local worker CLIs directly from this machine. Install at least one of these before running a real swarm or task:

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

Most intervention commands still use deterministic fixture ids so the local contract stays stable, but `swarm retry <shard-id>` now operates on persisted shard state from the latest local run:

- run ids: `run-active`, `run-paused`, `run-done`, `run-stopped`
- shard ids: `shard-queued`, `shard-assigned`, `shard-running`, `shard-blocked`, `shard-failed`, `shard-done`
- merge unit ids: `merge-001`, `merge-applied`, `merge-runtime-error`

## Task Orchestration Flow

Run an orchestrated task with approval checkpoints:

```bash
cargo run -- task --runtime codex "Ship orchestration flow"
cargo run -- tui
```

The `task` command runs a multi-phase agent workflow:

1. **Brainstorming** - a brainstorming agent produces a spec artifact
2. **Approval checkpoint** - operator sees `Approve? [y/n]` after brainstorming completes
3. **Writing plans** - a planning agent produces a plan artifact
4. **Approval checkpoint** - operator sees `Approve? [y/n]` after writing plans completes
5. **Execution** - implementation agents run in parallel

Approval prompts pause the orchestrator and wait for operator input before proceeding to the next phase. The run persists its state across restarts so recovery picks up from the last pending checkpoint.

## Commands

The CLI currently exposes these command groups:

- `task --runtime <codex|claude> <OBJECTIVE>`
- `tui`
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

## Task Run-Store Layout

Each `task` run creates a directory under `PATCHLANE_STATE_ROOT/tasks/<run-id>/` with:

- `run.json` for top-level run metadata and overall state
- `agents/` with one JSON file per agent recording role, phase, state, pid, and log paths
- `checkpoints/` with one JSON file per checkpoint recording phase, status, and approval prompt
- `artifacts/` with one JSON file per artifact recording type and path
- `events.jsonl` for ordered lifecycle events consumed by `swarm watch` and the TUI
- `logs/` with per-agent stdout and stderr log files

## Restart Recovery

When Patchlane restarts after an interruption it recovers the run state from the persisted store:

- The latest pending checkpoint is surfaced so the operator can re-issue the approval prompt
- Agents in `waiting_for_input` or `waiting_for_approval` states are reported as blocked agents
- The most recent event is shown as the latest event, including `checkpoint-decision` events
- Blocked state is reported in status output under the `Blockers` section

## TUI

The `tui` command opens a read-only observability surface over persisted state:

```bash
cargo run -- tui
```

- Lists all task runs with their current state
- Shows agents for the selected run with phase and state timelines
- Highlights blocked agents and approval waits
- Tails log output from persisted per-agent log files

The TUI is a read-only observer. It does not issue control commands or approve checkpoints.

## Local Artifacts

Each swarm run creates a directory under `PATCHLANE_STATE_ROOT` or `.patchlane/` with:

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
