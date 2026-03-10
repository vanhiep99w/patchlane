# Patchlane CLI Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build the first Patchlane CLI slice for `swarm run/status/watch/intervene/board/web`, using the approved CLI design and a spec-driven workflow contract based on `superpowers`.

**Architecture:** Start by bootstrapping a real Rust repository with a thin CLI application, explicit run-state models, and a local SQLite source of truth. Implement the operator-visible flow first, then layer in the internal workflow contract, placement policy, persistence, and intervention semantics behind stable command outputs.

**Tech Stack:** Git repository, Rust, `clap`, `tokio`, `rusqlite`, `serde`, Rust unit/integration tests, optional lightweight TUI/web placeholders for `board` and `web`

---

### Task 1: Bootstrap the repository and planning baseline

**Files:**
- Create: `.gitignore`
- Create: `README.md`
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/lib.rs`
- Create: `tests/`
- Create: `docs/plans/2026-03-09-patchlane-cli-design.md`
- Create: `docs/plans/2026-03-09-patchlane-cli-implementation.md`

**Step 1: Initialize the repository**

Run: `git init`
Expected: repository initialized in `/home/hieptran/Desktop/Patchlane/.git`

**Step 2: Write the failing bootstrap check**

Create a simple smoke test in `tests/bootstrap.rs` that invokes the library or CLI entrypoint and fails because the crate surface does not exist yet.

**Step 3: Run the test to verify it fails**

Run: `cargo test bootstrap`
Expected: FAIL because the crate entry files are missing

**Step 4: Add the minimal project files**

Create `Cargo.toml`, the crate entry files, and the minimal CLI skeleton.

**Step 5: Run the test to verify it passes**

Run: `cargo test bootstrap`
Expected: PASS because the crate entrypoints now exist

**Step 6: Commit**

```bash
git add .gitignore README.md Cargo.toml src tests docs/plans
git commit -m "chore: bootstrap patchlane cli repository"
```

### Task 2: Define CLI command topology and parser behavior

**Files:**
- Create: `src/cli.rs`
- Create: `src/commands/mod.rs`
- Create: `tests/cli/command_topology.rs`

**Step 1: Write the failing test**

Write tests that assert the CLI recognizes these commands:

- `swarm run`
- `swarm status`
- `swarm watch`
- `swarm pause`
- `swarm resume`
- `swarm retry`
- `swarm reassign`
- `swarm merge approve`
- `swarm merge reject`
- `swarm stop`
- `swarm board`
- `swarm web`

**Step 2: Run the test to verify it fails**

Run: `cargo test command_topology`
Expected: FAIL because commands are not registered

**Step 3: Write minimal implementation**

Register the command tree and stub handlers that return deterministic placeholder output.

**Step 4: Run the test to verify it passes**

Run: `cargo test command_topology`
Expected: PASS

**Step 5: Commit**

```bash
git add src/cli.rs src/commands/mod.rs tests/cli/command_topology.rs
git commit -m "feat: add patchlane command topology"
```

### Task 3: Model run, shard, placement, and intervention state

**Files:**
- Create: `src/domain/run.rs`
- Create: `src/domain/shard.rs`
- Create: `src/domain/placement.rs`
- Create: `src/domain/intervention.rs`
- Create: `tests/domain/run_state.rs`

**Step 1: Write the failing test**

Write tests that define the minimum state model:

- run states
- shard states
- placement states: `main_repo`, `worktree`, `blocked`
- intervention results: `queued`, `acknowledged`, `applied`, `failed`

**Step 2: Run the test to verify it fails**

Run: `cargo test run_state`
Expected: FAIL because the domain model does not exist

**Step 3: Write minimal implementation**

Add explicit enums/types and state transition helpers for the operator-visible model using `serde`-friendly Rust types.

**Step 4: Run the test to verify it passes**

Run: `cargo test run_state`
Expected: PASS

**Step 5: Commit**

```bash
git add src/domain tests/domain/run_state.rs
git commit -m "feat: add run and shard state model"
```

### Task 4: Implement `swarm run` opening output contract

**Files:**
- Create: `src/commands/run.rs`
- Create: `src/renderers/run_renderer.rs`
- Create: `tests/cli/run_output.rs`

**Step 1: Write the failing test**

Write a snapshot-style test that expects `swarm run "demo objective"` to print, in order:

1. `Run`
2. `Objective`
3. `Plan`
4. `Placement`
5. `Next`

**Step 2: Run the test to verify it fails**

Run: `cargo test run_output`
Expected: FAIL because the renderer does not produce the required structure

**Step 3: Write minimal implementation**

Implement deterministic rendering for the opening block, with placeholder planning data if necessary.

**Step 4: Run the test to verify it passes**

Run: `cargo test run_output`
Expected: PASS

**Step 5: Commit**

```bash
git add src/commands/run.rs src/renderers/run_renderer.rs tests/cli/run_output.rs
git commit -m "feat: implement swarm run opening output"
```

### Task 5: Add compact progress updates and live watch events

**Files:**
- Create: `src/events/run_events.rs`
- Create: `src/commands/status.rs`
- Create: `src/commands/watch.rs`
- Create: `src/renderers/status_renderer.rs`
- Create: `src/renderers/watch_renderer.rs`
- Create: `tests/cli/status_output.rs`
- Create: `tests/cli/watch_output.rs`

**Step 1: Write the failing tests**

Write tests that assert:

- `swarm status` returns a stable snapshot with run state, shard table, blocker summary, merge queue summary, latest event, and suggested next command
- `swarm watch` prints operationally meaningful event lines without raw transcript noise

**Step 2: Run the tests to verify they fail**

Run: `cargo test status_output watch_output`
Expected: FAIL because the commands and renderers do not exist yet

**Step 3: Write minimal implementation**

Implement the status and watch commands with deterministic event fixtures and renderers.

**Step 4: Run the tests to verify they pass**

Run: `cargo test status_output watch_output`
Expected: PASS

**Step 5: Commit**

```bash
git add src/events src/commands/status.rs src/commands/watch.rs src/renderers tests/cli
git commit -m "feat: add status and watch output surfaces"
```

### Task 6: Implement placement reasoning and visibility

**Files:**
- Create: `src/services/placement_engine.rs`
- Create: `tests/services/placement_engine.rs`
- Modify: `src/commands/run.rs`
- Modify: `src/renderers/run_renderer.rs`
- Modify: `src/renderers/status_renderer.rs`

**Step 1: Write the failing test**

Write tests for placement decisions based on:

- single low-risk shard
- multiple writable shards
- dirty repo state
- explicit safe mode
- blocked conditions

The tests should assert both the placement result and the short operator-facing reason.

**Step 2: Run the test to verify it fails**

Run: `cargo test placement_engine`
Expected: FAIL because the placement engine does not exist

**Step 3: Write minimal implementation**

Implement a rule-based placement engine that returns:

- placement choice
- human-readable reason
- block reason if not dispatchable

**Step 4: Run the test to verify it passes**

Run: `cargo test placement_engine`
Expected: PASS

**Step 5: Commit**

```bash
git add src/services/placement_engine.rs src/commands/run.rs src/renderers tests/services
git commit -m "feat: add placement engine and explanations"
```

### Task 7: Introduce the `superpowers` workflow contract adapter

**Files:**
- Create: `src/workflow/superpowers_contract.rs`
- Create: `tests/workflow/superpowers_contract.rs`
- Modify: `src/events/run_events.rs`
- Modify: `src/commands/run.rs`
- Modify: `src/commands/watch.rs`

**Step 1: Write the failing test**

Write tests that map internal workflow stages to operator-visible states:

- `clarifying objective`
- `drafting design`
- `writing plan`
- `splitting assignments`
- `dispatching shards`
- `reviewing outputs`
- `merging clean shards`

**Step 2: Run the test to verify it fails**

Run: `cargo test superpowers_contract`
Expected: FAIL because there is no workflow contract adapter

**Step 3: Write minimal implementation**

Implement a workflow adapter that models these stages and emits CLI-facing events without exposing raw skill internals.

**Step 4: Run the test to verify it passes**

Run: `cargo test superpowers_contract`
Expected: PASS

**Step 5: Commit**

```bash
git add src/workflow src/events/run_events.rs src/commands/run.rs src/commands/watch.rs tests/workflow
git commit -m "feat: add superpowers workflow contract adapter"
```

### Task 8: Implement intervention commands and semantics

**Files:**
- Create: `src/commands/pause.rs`
- Create: `src/commands/resume.rs`
- Create: `src/commands/retry.rs`
- Create: `src/commands/reassign.rs`
- Create: `src/commands/stop.rs`
- Create: `src/commands/merge_approve.rs`
- Create: `src/commands/merge_reject.rs`
- Create: `tests/cli/intervention_commands.rs`

**Step 1: Write the failing test**

Write tests that assert:

- commands are idempotent from the operator perspective
- commands return only `queued`, `acknowledged`, `applied`, or `failed`
- invalid state returns explicit failure reasons
- merge commands target a concrete merge unit id

**Step 2: Run the test to verify it fails**

Run: `cargo test intervention_commands`
Expected: FAIL because the handlers do not exist

**Step 3: Write minimal implementation**

Implement command handlers against the run-state model with deterministic responses and explicit failure messages.

**Step 4: Run the test to verify it passes**

Run: `cargo test intervention_commands`
Expected: PASS

**Step 5: Commit**

```bash
git add src/commands tests/cli/intervention_commands.rs
git commit -m "feat: add intervention command semantics"
```

### Task 9: Add `board` and `web` overview placeholders

**Files:**
- Create: `src/commands/board.rs`
- Create: `src/commands/web.rs`
- Create: `tests/cli/overview_commands.rs`

**Step 1: Write the failing test**

Write tests that assert:

- `swarm board` returns a compact overview of active runs, blocked shards, and merge queue
- `swarm web` resolves to a read-mostly overview entry point

**Step 2: Run the test to verify it fails**

Run: `cargo test overview_commands`
Expected: FAIL because these commands are not implemented

**Step 3: Write minimal implementation**

Implement lightweight placeholders that honor the CLI contract without overbuilding the UI.

**Step 4: Run the test to verify it passes**

Run: `cargo test overview_commands`
Expected: PASS

**Step 5: Commit**

```bash
git add src/commands/board.rs src/commands/web.rs tests/cli/overview_commands.rs
git commit -m "feat: add overview command placeholders"
```

### Task 10: Verify the end-to-end CLI contract and document local execution

**Files:**
- Create: `tests/e2e/cli_contract.rs`
- Modify: `README.md`

**Step 1: Write the failing test**

Write an end-to-end test that covers:

- starting a run
- checking status
- watching events
- issuing an intervention command
- observing an explicit next step or completion state

**Step 2: Run the test to verify it fails**

Run: `cargo test cli_contract`
Expected: FAIL because the CLI pieces are not integrated

**Step 3: Write minimal implementation**

Integrate the command handlers, shared state fixtures, and documentation so the local CLI can be exercised consistently.

**Step 4: Run the test suite to verify it passes**

Run: `cargo test`
Expected: PASS

**Step 5: Commit**

```bash
git add README.md tests/e2e/cli_contract.rs src
git commit -m "feat: verify patchlane cli contract end to end"
```

## Notes

- Because the current workspace is not yet a git repository, Task 1 must happen before any commit commands can succeed.
- Because there is no existing codebase yet, this plan treats repository initialization and SQLite bootstrap as part of the implementation, not as external prerequisites.
- Keep `board` and `web` intentionally thin in v1. They are support surfaces, not the primary entry path.

Plan complete and saved to `docs/plans/2026-03-09-patchlane-cli-implementation.md`. Two execution options:

**1. Subagent-Driven (this session)** - I dispatch fresh subagent per task, review between tasks, fast iteration

**2. Parallel Session (separate)** - Open new session with executing-plans, batch execution with checkpoints

Which approach?
