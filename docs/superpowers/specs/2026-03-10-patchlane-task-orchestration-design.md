# Patchlane Task Orchestration And Minimal TUI Design

**Date:** 2026-03-10
**Status:** Draft for approval

## Goal

Build a new `patchlane task "<objective>"` entrypoint that runs a spec-driven workflow based on `superpowers`, uses the current runtime family (`codex` or `claude`) for swarm execution, persists orchestration state and artifacts locally, pauses for terminal approvals at key checkpoints, and exposes a minimal TUI that can list agents and show realtime detail for a selected agent.

## Scope

This design covers the first end-to-end version of:

- task kickoff through `patchlane task`
- runtime-aware orchestration for `brainstorming -> writing-plans -> subagent-driven-development -> finishing-a-development-branch`
- terminal checkpoint approvals with `Approve? [y/n]`
- persisted run, agent, checkpoint, artifact, event, and log state
- a minimal TUI for swarm observability

This version deliberately keeps the main orchestrator context thin and moves detailed inspection into persisted state plus the TUI.

## Product Boundary

The first deliverable is not just a process launcher. It is a local orchestration system with explicit control points:

1. User runs `patchlane task [--runtime <codex|claude>] "<objective>"`
2. Patchlane resolves runtime
3. Patchlane creates a run/session and workspace policy
4. Patchlane launches `brainstorming`
5. Patchlane persists the generated spec artifact
6. Patchlane pauses in the terminal and asks `Approve? [y/n]`
7. If approved, Patchlane launches `writing-plans`
8. Patchlane persists the generated implementation plan
9. Patchlane pauses in the terminal and asks `Approve? [y/n]`
10. If approved, Patchlane launches `subagent-driven-development`
11. Patchlane tracks high-level agent execution and review state
12. Patchlane completes via `finishing-a-development-branch`

The orchestrator must remain intentionally thin. It should know what phase the run is in, what is blocked, and what needs user action. It should not depend on parsing long agent transcripts to determine control flow.

## Runtime Model

Runtime selection follows this policy:

- Prefer explicit `--runtime`
- If `--runtime` is absent, auto-detect the originating runtime when possible
- If auto-detection is ambiguous, ask for confirmation
- If the originating runtime is `codex`, swarm agents use `codex`
- If the originating runtime is `claude`, swarm agents use `claude`

Patchlane should model runtime at the run level for v1. A single run uses one runtime family consistently.

## Workspace Model

Workspace isolation is the default policy, but not an unconditional rule.

- Runs should start with an isolated session/workspace policy
- Small low-risk tasks do not have to create a new worktree
- Large or risky tasks can allocate isolated worktrees/workspaces
- The orchestrator or high-level agent may decide whether a subtask needs separate writable isolation

This keeps the system safe by default without paying unnecessary workspace cost for trivial work.

## State Model

The local run store is the source of truth for orchestration and observability.

### Core persisted entities

- `run`
- `agent`
- `checkpoint`
- `artifact`
- `event`
- `log`

### Run fields

At minimum:

- run id
- objective
- runtime
- current phase
- overall state
- timestamps
- root workspace/session path
- current blocking reason if any

### Agent fields

At minimum:

- agent id
- run id
- parent agent id if applicable
- role (`brainstorming`, `writing-plans`, `implementer`, `spec-reviewer`, `code-quality-reviewer`, `final-reviewer`, etc.)
- current phase
- current state
- runtime
- workspace path
- pid if local process-backed
- related artifact ids
- timestamps

### Checkpoint fields

At minimum:

- checkpoint id
- run id
- phase
- target artifact or step
- status (`pending`, `approved`, `rejected`, `expired`)
- prompt text shown to user
- response (`y` / `n`)
- optional note
- timestamps

### Artifact fields

At minimum:

- artifact id
- run id
- producing agent id
- type (`spec`, `plan`, `review`, `summary`, etc.)
- filesystem path
- timestamps

### Event fields

At minimum:

- event id
- run id
- agent id (optional for run-level events)
- event type
- event payload summary
- timestamp

### Logs

Each high-level agent should have stdout/stderr logs persisted separately for later tailing in the TUI.

## Orchestrator State Vocabulary

The main orchestrator keeps only a small state vocabulary:

- `queued`
- `running`
- `waiting_for_input`
- `waiting_for_approval`
- `in_review`
- `done`
- `failed`

This vocabulary is enough for the main control loop and avoids bloating context with detailed operational chatter.

## Agent Reporting Contract

High-level agents must report state through a Patchlane-controlled wrapper contract, not by stdout parsing.

Recommended shape:

- `patchlane agent-event start ...`
- `patchlane agent-event phase ...`
- `patchlane agent-event waiting-input ...`
- `patchlane agent-event waiting-approval ...`
- `patchlane agent-event artifact ...`
- `patchlane agent-event review-start ...`
- `patchlane agent-event review-pass ...`
- `patchlane agent-event review-fail ...`
- `patchlane agent-event done ...`
- `patchlane agent-event fail ...`

### Why direct event reporting

This is required because:

- stdout/stderr should remain free for natural logs and transcripts
- parsing free-form logs for state is brittle
- the TUI needs a stable event stream
- orchestration must survive process interruption and resume from persisted state

### Scope of the wrapper in v1

The wrapper is mandatory only for high-level agents:

- `brainstorming`
- `writing-plans`
- `implementer`
- `spec reviewer`
- `code quality reviewer`
- `final reviewer`
- checkpoint or approval waiter agents if they exist as explicit units

Lower-level details can remain in raw logs for now.

## Checkpoints And User Control

Checkpoint approvals happen in the terminal as the primary control surface.

### Approval behavior

At key checkpoints, Patchlane prints:

`Approve? [y/n]`

If the user answers:

- `y`: the run automatically continues to the next step
- `n`: the run enters a blocking state appropriate to context (`waiting_for_input` or failed checkpoint)

### Checkpoint classes in v1

- after `brainstorming`
- after `writing-plans`
- when a high-level agent requests more information
- when a review loop requires user intervention
- before branch-finishing actions if needed

### Partial blocking behavior

If only one agent needs input, only that agent pauses. Independent agents may continue if they are not blocked by the same dependency.

### Persisted approval data

Every approval decision is persisted with:

- who requested it
- what artifact or step it refers to
- exact decision
- timestamp
- optional note

## Execution Model

Execution uses a hybrid task splitting policy.

- Default split unit is the task from the implementation plan
- Large tasks may be split into smaller subtasks if the orchestrator or high-level execution agent determines that is necessary
- `subagent-driven-development` remains the execution protocol, but Patchlane wraps its high-level progress and checkpoints in persisted state

This preserves the quality gates of the `superpowers` flow while giving Patchlane stable external visibility.

## Minimal TUI In The First Delivery

The TUI is not a full feature-rich console in v1, but it is part of the first product boundary because the user must be able to observe live swarm work without depending on the main orchestrator context.

### Minimum TUI capabilities

- list runs
- list agents for a selected run
- show current state for each agent
- show current phase for each agent
- highlight blockers and approval waits
- show event timeline for a selected agent
- tail stdout/stderr logs for a selected agent in realtime
- show artifact paths related to the selected agent

### Required interaction model

When the user selects an agent in the TUI, they should be able to inspect:

- what phase it is in right now
- whether it is waiting, running, done, or failed
- recent state transitions
- raw log output in realtime
- related artifacts like spec/plan/review paths

### Deliberate limitation in v1

The TUI does not need perfect structured tool-lifecycle detail yet. If exact tool-level visibility is unavailable, the TUI should still provide useful live insight through:

- high-level events
- phase state
- log tailing

A richer tool-event stream can be added in a later phase.

## Delivery Phasing

### Phase 1

- `patchlane task`
- runtime resolution
- run store
- high-level agent event contract
- terminal checkpoint approvals
- artifact/log persistence
- swarm execution of the approved workflow

### Phase 1.5

- minimal TUI reading directly from the persisted run store
- run list
- agent list
- selected-agent detail view
- realtime log tailing
- event timeline

### Later phase

- richer tool activity capture
- more advanced filtering and search
- grouped subtree views
- more detailed intervention controls inside TUI

## Error Handling

The orchestrator must explicitly persist failures and waits instead of silently stalling.

At minimum:

- runtime detection failure -> ask for confirmation or mark blocked
- checkpoint rejection -> persist reject state and stop or wait for input
- agent wrapper reporting failure -> mark affected agent failed
- artifact write failure -> persist failed artifact event
- log path creation failure -> persist failed agent launch event
- orchestrator restart -> rebuild current run state from persisted store

## Testing Strategy

The first implementation is successful when tests prove:

- `patchlane task` follows `brainstorming -> approval -> writing-plans -> approval -> execution`
- `Approve? [y/n]` decisions correctly move run state forward or into blocking state
- high-level agents emit wrapper events into the store
- spec, plan, approval, event, and log artifacts are persisted
- runtime resolution follows the agreed policy
- selected-agent state can be reconstructed from the store for TUI rendering
- selected-agent logs can be tailed from persisted files
- orchestrator restart can recover enough state to continue or accurately report blocking conditions

## Non-Goals For This Version

- perfect low-level tool-by-tool tracing for every spawned agent
- fully autonomous no-checkpoint execution
- multi-runtime mixing inside one run
- a feature-complete dashboard before the core orchestration model is stable

## Recommendation

Implement the orchestration model and the minimal TUI against the same persisted store. The orchestrator should write state; the TUI should read it. This keeps the control plane small, the observability model stable, and future UI work decoupled from execution logic.
