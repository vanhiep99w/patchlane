# Local Worker Launch Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the placeholder `swarm run/status/watch` path with a real local worker launcher that spawns `codex` or `claude` CLI processes, persists run artifacts on disk, and reports real shard state.

**Architecture:** Add a file-backed run store under a local state root, a small heuristic planner that produces multiple shard briefs, and a launcher that spawns local runtime CLIs per shard. Keep v1 narrow: runtime chosen up front for the whole run, fixed small shard count, no dynamic rebalancing, best-effort supervisory controls.

**Tech Stack:** Rust, `clap`, `serde`, `serde_json`, `std::process::Command`, filesystem persistence, Rust unit/integration tests

---

### Task 1: Add runtime selection to `swarm run`

**Files:**
- Modify: `src/cli.rs`
- Modify: `src/commands/run.rs`
- Test: `tests/cli/run_output.rs`

**Step 1: Write the failing test**

Add tests that require:

- `swarm run --runtime codex "demo objective"` to parse and succeed
- `swarm run --runtime claude "demo objective"` to parse and succeed
- `swarm run "demo objective"` to fail with missing `--runtime`
- unsupported runtime values to fail during parsing

**Step 2: Run test to verify it fails**

Run: `cargo test run_output`
Expected: FAIL because `swarm run` does not require or expose runtime selection

**Step 3: Write minimal implementation**

Update the CLI model so `RunCommand` includes a required `--runtime <codex|claude>` argument and thread that value into the run command handler. Keep the existing output shape, but include the selected runtime in the opening block.

**Step 4: Run test to verify it passes**

Run: `cargo test run_output`
Expected: PASS

**Step 5: Commit**

```bash
git add src/cli.rs src/commands/run.rs tests/cli/run_output.rs
git commit -m "feat: require runtime selection for swarm run"
```

### Task 2: Create the file-backed run store

**Files:**
- Create: `src/store/mod.rs`
- Create: `src/store/run_store.rs`
- Modify: `src/lib.rs`
- Test: `tests/store/run_store.rs`
- Modify: `tests/bootstrap.rs`

**Step 1: Write the failing test**

Add tests that require the run store to:

- create a run directory under a local state root
- write `run.json`
- write one `shard-<id>.json` file per shard
- append lifecycle events to an events file
- round-trip persisted metadata back into Rust structs

**Step 2: Run test to verify it fails**

Run: `cargo test run_store`
Expected: FAIL because no file-backed run store exists

**Step 3: Write minimal implementation**

Add serde-friendly structs for persisted run metadata, shard metadata, and event records. Implement filesystem helpers that create a run directory, write metadata JSON files, and append newline-delimited event records.

**Step 4: Run test to verify it passes**

Run: `cargo test run_store`
Expected: PASS

**Step 5: Commit**

```bash
git add src/store src/lib.rs tests/store/run_store.rs tests/bootstrap.rs
git commit -m "feat: add file-backed run store"
```

### Task 3: Build the heuristic shard planner

**Files:**
- Create: `src/planner/mod.rs`
- Create: `src/planner/shard_planner.rs`
- Test: `tests/planner/shard_planner.rs`

**Step 1: Write the failing test**

Add tests that require the planner to:

- produce 2-4 shard briefs from one objective
- keep shard ids deterministic
- include the original objective context in each shard brief
- preserve a stable shard count for the same input

**Step 2: Run test to verify it fails**

Run: `cargo test shard_planner`
Expected: FAIL because no real planner exists

**Step 3: Write minimal implementation**

Implement a simple heuristic planner that emits a small fixed set of shard briefs such as analysis, implementation, verification, and integration, trimming to a minimum useful count. Keep the logic deterministic and transparent.

**Step 4: Run test to verify it passes**

Run: `cargo test shard_planner`
Expected: PASS

**Step 5: Commit**

```bash
git add src/planner tests/planner/shard_planner.rs
git commit -m "feat: add heuristic shard planner"
```

### Task 4: Add the local worker launcher

**Files:**
- Create: `src/runtime/mod.rs`
- Create: `src/runtime/launcher.rs`
- Test: `tests/runtime/launcher.rs`

**Step 1: Write the failing test**

Add tests that require the launcher to:

- build the correct `Command` invocation for `codex`
- build the correct `Command` invocation for `claude`
- create stdout/stderr log files for each shard
- capture spawn failures as structured errors

**Step 2: Run test to verify it fails**

Run: `cargo test launcher`
Expected: FAIL because there is no real process launcher

**Step 3: Write minimal implementation**

Implement a launcher that prepares `std::process::Command`, redirects stdout/stderr to per-shard log files, spawns the child process, and returns pid plus log paths. Keep prompt construction minimal and deterministic.

**Step 4: Run test to verify it passes**

Run: `cargo test launcher`
Expected: PASS

**Step 5: Commit**

```bash
git add src/runtime tests/runtime/launcher.rs
git commit -m "feat: add local worker launcher"
```

### Task 5: Add workspace allocation for shards

**Files:**
- Create: `src/workspaces/mod.rs`
- Create: `src/workspaces/worktree_manager.rs`
- Test: `tests/workspaces/worktree_manager.rs`

**Step 1: Write the failing test**

Add tests that require the workspace manager to:

- allocate a stable workspace path per shard
- create workspace directories for shards
- keep paths inside a Patchlane-managed state root
- surface filesystem errors clearly

**Step 2: Run test to verify it fails**

Run: `cargo test worktree_manager`
Expected: FAIL because shard workspaces are not allocated yet

**Step 3: Write minimal implementation**

Implement a first-pass workspace allocator that creates per-shard directories under the run root. Do not add real git worktree logic yet; keep v1 focused on isolated local directories and stable paths.

**Step 4: Run test to verify it passes**

Run: `cargo test worktree_manager`
Expected: PASS

**Step 5: Commit**

```bash
git add src/workspaces tests/workspaces/worktree_manager.rs
git commit -m "feat: add shard workspace allocator"
```

### Task 6: Integrate `swarm run` with planning, persistence, and launching

**Files:**
- Modify: `src/commands/run.rs`
- Modify: `src/renderers/run_renderer.rs`
- Modify: `src/events/run_events.rs`
- Modify: `src/lib.rs`
- Test: `tests/e2e/cli_contract.rs`

**Step 1: Write the failing test**

Extend the e2e contract to require:

- `swarm run --runtime codex "<objective>"` creates a real run id
- run output includes runtime and shard count
- launched shards are persisted to the run store
- failure to spawn a runtime surfaces as a real shard failure

**Step 2: Run test to verify it fails**

Run: `cargo test cli_contract`
Expected: FAIL because `swarm run` still returns placeholder-only data

**Step 3: Write minimal implementation**

Replace the placeholder path in `run::execute` with real orchestration:

- create run metadata
- plan shards
- allocate shard workspaces
- spawn one worker per shard
- persist shard metadata and initial events
- render the opening block from real run data

Keep the implementation synchronous enough to be testable, but persist everything needed for later status/watch reads.

**Step 4: Run test to verify it passes**

Run: `cargo test cli_contract`
Expected: PASS

**Step 5: Commit**

```bash
git add src/commands/run.rs src/renderers/run_renderer.rs src/events/run_events.rs src/lib.rs tests/e2e/cli_contract.rs
git commit -m "feat: launch real local worker runs"
```

### Task 7: Make `swarm status` read persisted run state

**Files:**
- Modify: `src/commands/status.rs`
- Modify: `src/renderers/status_renderer.rs`
- Modify: `src/events/run_events.rs`
- Test: `tests/cli/status_output.rs`

**Step 1: Write the failing test**

Update status tests to require:

- reading the latest persisted run metadata
- listing real shard runtime, pid, workspace, and state
- showing explicit failures or blockers from stored state
- suggesting the next appropriate command based on persisted status

**Step 2: Run test to verify it fails**

Run: `cargo test status_output`
Expected: FAIL because status still uses fixture-only data

**Step 3: Write minimal implementation**

Load the latest run from the file-backed run store, derive an operator-facing snapshot from persisted shard metadata, and render that snapshot. Keep output compact and deterministic for tests.

**Step 4: Run test to verify it passes**

Run: `cargo test status_output`
Expected: PASS

**Step 5: Commit**

```bash
git add src/commands/status.rs src/renderers/status_renderer.rs src/events/run_events.rs tests/cli/status_output.rs
git commit -m "feat: read persisted state for swarm status"
```

### Task 8: Make `swarm watch` read persisted lifecycle events

**Files:**
- Modify: `src/commands/watch.rs`
- Modify: `src/renderers/watch_renderer.rs`
- Modify: `src/events/run_events.rs`
- Test: `tests/cli/watch_output.rs`

**Step 1: Write the failing test**

Update watch tests to require:

- reading persisted lifecycle events from the latest run
- including launched, running, failed, and completed shard events
- continuing to avoid transcript noise

**Step 2: Run test to verify it fails**

Run: `cargo test watch_output`
Expected: FAIL because watch still uses static workflow fixtures

**Step 3: Write minimal implementation**

Load event records from the run store, map them to operator-facing watch lines, and keep the renderer append-only and concise. Preserve the contract that watch surfaces operational state instead of raw transcript text.

**Step 4: Run test to verify it passes**

Run: `cargo test watch_output`
Expected: PASS

**Step 5: Commit**

```bash
git add src/commands/watch.rs src/renderers/watch_renderer.rs src/events/run_events.rs tests/cli/watch_output.rs
git commit -m "feat: stream persisted worker events"
```

### Task 9: Implement retry for real launched shards

**Files:**
- Modify: `src/commands/reassign.rs`
- Modify: `src/commands/intervention_support.rs`
- Modify: `src/store/run_store.rs`
- Test: `tests/cli/intervention_commands.rs`

**Step 1: Write the failing test**

Update intervention tests to require:

- `retry <shard-id>` to relaunch a failed shard
- retried shards to receive a new pid
- previous attempts to remain visible in persisted metadata

**Step 2: Run test to verify it fails**

Run: `cargo test intervention_commands`
Expected: FAIL because retry still returns fixture-only responses

**Step 3: Write minimal implementation**

Wire retry into the real run store and launcher. Record a new attempt for the shard, preserve prior attempt history, spawn a fresh process, and emit corresponding events.

**Step 4: Run test to verify it passes**

Run: `cargo test intervention_commands`
Expected: PASS

**Step 5: Commit**

```bash
git add src/commands/reassign.rs src/commands/intervention_support.rs src/store/run_store.rs tests/cli/intervention_commands.rs
git commit -m "feat: retry real launched shards"
```

### Task 10: Document runtime prerequisites and manual QA

**Files:**
- Modify: `README.md`
- Test: `tests/e2e/cli_contract.rs`

**Step 1: Write the failing test**

Update the README assertions to require:

- documenting that `codex` and `claude` CLIs must be installed locally
- documenting `swarm run --runtime <codex|claude>`
- documenting where run artifacts and logs are stored

**Step 2: Run test to verify it fails**

Run: `cargo test cli_contract`
Expected: FAIL because README does not document the real runtime path

**Step 3: Write minimal implementation**

Update `README.md` with:

- runtime prerequisites
- example commands for both runtimes
- local artifact/log locations
- manual QA steps for a real launched run

**Step 4: Run test suite to verify it passes**

Run: `cargo test`
Expected: PASS

**Step 5: Commit**

```bash
git add README.md tests/e2e/cli_contract.rs
git commit -m "docs: add local runtime launch instructions"
```
