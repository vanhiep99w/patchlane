# Local Worker Launch Design

**Date:** 2026-03-10
**Status:** Approved

## Goal

Make `swarm run` launch real local `codex` or `claude` CLI worker processes on this machine, immediately split one objective into multiple workers, and expose real run state through `status` and `watch`.

## Decisions

- Runtime is selected up front per run: `codex` or `claude`
- Workers are spawned as local child processes on this machine
- Patchlane launches multiple workers immediately, not a single-worker pilot
- v1 uses a file-backed local run store instead of SQLite
- Sharding uses a fixed small shard count and heuristic work-packet briefs
- No dynamic rebalancing in v1

## Architecture

Patchlane becomes a local orchestrator. `swarm run --runtime <codex|claude> "<objective>"` creates a run id, writes run metadata into a local state directory, generates shard briefs, prepares shard workspaces, and spawns one local CLI process per shard.

Each shard gets:

- a shard brief file
- a workspace path
- stdout/stderr log files
- persisted metadata including pid, runtime, timestamps, and state

`swarm status` reads stored run and shard metadata to show current state. `swarm watch` reads append-only event files to stream lifecycle updates such as launched, running, failed, blocked, and completed.

## Components

- `planner`: splits one objective into a small number of shard briefs
- `launcher`: spawns local `codex` or `claude` processes with shard-specific prompts
- `run store`: file-backed persisted metadata for runs, shards, events, logs, and pids
- `worktree manager`: allocates isolated workspaces for writable shards
- `status/watch readers`: reconstruct operator-facing state from stored metadata and events

## Data Flow

1. Operator runs `swarm run --runtime <codex|claude> "<objective>"`
2. Patchlane creates a run directory under a local state root
3. Planner writes shard briefs
4. Worktree manager allocates shard workspaces
5. Launcher spawns one worker process per shard
6. Patchlane records events and process metadata
7. `swarm status` summarizes persisted run state
8. `swarm watch` streams persisted lifecycle events

## Error Handling

- Worker start failure marks the shard `failed` with the process error
- Non-zero worker exit marks the shard `failed` with the exit code
- Worktree allocation failure marks the shard `blocked`
- Worker inactivity beyond a timeout window marks the shard `blocked`
- `retry` relaunches the same shard brief as a fresh process
- `pause` and `stop` are best-effort supervisory controls in v1
- `merge approve` and `merge reject` remain thin until real merge artifacts exist

## Testing

The phase is successful when:

- `swarm run --runtime codex "<objective>"` and `--runtime claude` launch real local worker processes
- multiple shard briefs are created for one objective
- each shard has persisted metadata, logs, and workspace information
- `swarm status` reflects real launched shard state
- `swarm watch` emits real lifecycle events
- worker launch failures are explicit and actionable
