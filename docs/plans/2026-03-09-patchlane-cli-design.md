# Patchlane CLI Design

**Date:** 2026-03-09
**Source:** Approved from `PRD.md`
**Focus:** CLI product spec for the agent-native swarm tool

---

## 1. Purpose

Patchlane should present as a command-native swarm tool inside a coding-agent instance, not as a dashboard-first platform. The CLI is the primary product surface. It must let an operator submit one objective, inspect the system's plan, watch progress, and intervene only when needed.

This design intentionally prioritizes operator flow and terminal readability over backend detail. Internal architecture is only defined when it directly affects CLI behavior.

For v1 implementation, the CLI should assume a local embedded source of truth backed by SQLite so that `status`, `watch`, `resume`, and recovery can read consistent run state without requiring an external service.

## 2. Design Scope

This design covers:

- command information architecture
- `swarm run` behavior
- output format for `run`, `status`, and `watch`
- intervention semantics and guardrails
- the workflow contract between Patchlane and `superpowers`

This design does not fully specify:

- persistence schema
- runtime adapter internals
- merge engine internals
- planner algorithms beyond operator-visible behavior

## 3. Product Decisions

The approved product decisions for CLI v1 are:

- default run style is `mostly automatic`
- in-agent updates use a `hybrid` model: compact by default, detailed when watching
- intervention level is `practical`, not minimal and not fully manual
- `superpowers` acts as the internal workflow contract, while `Patchlane` remains the user-facing shell

## 4. CLI Information Architecture

The command tree should stay centered on the operator's actual loop after submitting an objective:

1. start a run
2. observe the run
3. intervene if required
4. view wider context when needed

The command surface for v1 is:

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

The CLI must not expose internal system nouns such as planner stages, projections, or runtime adapters as first-class commands. Those internals should be reflected through summaries and states, not surfaced as operator responsibilities.

### 4.1 Command Grouping

The information architecture is organized into four command groups:

- `run`: create a new run from one objective
- `observe`: inspect current state with `status` and `watch`
- `intervene`: issue practical control commands
- `overview`: open broader read-mostly views with `board` and `web`

### 4.2 Primary Entry Point

`swarm run "<objective>"` is the dominant entry point. The other commands exist to support runs that were already created. Users should not need to learn a separate setup flow before receiving value.

## 5. `swarm run` Behavior

`swarm run` should behave as "declare objective, then inspect the system's plan". It should not require the operator to orchestrate low-level execution steps manually.

### 5.1 Standard Flow

1. The operator calls `swarm run "<objective>"`.
2. The CLI immediately acknowledges creation with a run header and run id.
3. The system drafts a short internal spec and renders a compact summary of what it believes the objective means.
4. The system shards the work and prints a shard summary.
5. The system determines placement for each shard and prints a placement plan with short reasons.
6. If no critical blocker exists, execution begins automatically.
7. Compact progress updates continue in the current session.
8. At completion, the CLI prints a result summary and next action.

### 5.2 Clarification Policy

The default run style is `mostly automatic`.

That means:

- the system should continue automatically whenever the objective is sufficiently clear
- the system should ask only when ambiguity materially affects correctness, placement trust, or merge confidence
- in non-interactive mode, the system should fail or block explicitly instead of opening a clarification exchange

The CLI should not dump a long intermediate spec to the operator. If clarification is needed, it should ask one short question or mark the affected shard as blocked pending operator input.

### 5.3 Output Contract for `swarm run`

The opening block of `swarm run` should always appear in the same order:

1. `Run`: run id, mode, runtime preference
2. `Objective`: short understanding of the task
3. `Plan`: shard count and notable shard risk
4. `Placement`: `main_repo`, `worktree`, or `blocked` for each shard with one reason
5. `Next`: what the system is doing next or what it needs from the operator

This fixed order creates muscle memory and keeps runs comparable.

### 5.4 Responsiveness Rules

`swarm run` must not go silent for long periods. If planning or inspection takes time, the CLI should render compact progress states such as:

- `drafting spec`
- `scoring overlap`
- `creating worktree`
- `dispatching shard`
- `waiting for review`
- `awaiting merge decision`

The output should remain readable as an append-only session transcript. A user who scrolls up should still understand the run lifecycle without reconstructing hidden state.

## 6. Workflow Contract Under the CLI

Patchlane should use `superpowers` as the internal spec-driven workflow contract rather than inventing a new agent process from scratch.

Patchlane remains responsible for orchestration. `superpowers` provides a stable structure for how work should progress once the system accepts an objective.

### 6.1 Internal Workflow Stages

The internal workflow contract should map roughly to:

- clarify if needed
- draft a short design/spec
- write a plan
- split into assignments
- execute assignments
- review and verify results
- merge or request operator decision

Patchlane must add its own orchestration layers on top of that:

- placement decision
- repo-state inspection
- shard scheduling
- worktree lifecycle
- merge queue
- recovery and resume

### 6.2 Operator-Facing Reflection

The operator should not see raw skill names or low-level workflow jargon. Instead, CLI states should reflect workflow stages in human-readable form, such as:

- `clarifying objective`
- `drafting design`
- `writing plan`
- `splitting assignments`
- `dispatching shards`
- `reviewing outputs`
- `merging clean shards`

This keeps status understandable while preserving a stable internal process contract across runtimes.

### 6.3 Design Principle

The core rule is:

`Patchlane orchestrates; superpowers structures the work.`

## 7. Output Model

The CLI should use three distinct output surfaces rather than one overloaded log format.

### 7.1 `swarm run`

Purpose:

- start a run
- show the initial plan
- stream only milestone-level progress in the current session

It should show:

- run header
- compact objective summary
- shard summary
- placement plan
- milestone updates
- blocker or merge prompts
- final result summary

It should not show:

- token-by-token streaming
- raw agent transcript output
- noisy internal events

### 7.2 `swarm status`

Purpose:

- return a stable snapshot for one run

It should always include:

- overall run state
- shard table with runtime, state, and placement
- blocker summary
- merge queue summary
- latest event
- suggested next command

This command should let an operator decide quickly whether intervention is necessary.

### 7.3 `swarm watch`

Purpose:

- stream operationally meaningful events for a run

It should include:

- state transitions
- dispatch events
- worktree creation events
- retry and failure events
- review results
- merge queue changes

`watch` should be more detailed than `run`, but still avoid raw transcript noise.

### 7.4 Common Formatting Rules

Every visible update should answer:

1. what the system is doing
2. what is blocking progress, if anything
3. what the operator should do next, if anything

Additional mandatory rules:

- placement reasons must appear in `run` and `status`
- `blocked` is a first-class state, not a vague failure
- merge decisions must clearly state what is waiting, why it is not auto-merged, and the exact command to resolve it

## 8. Intervention Semantics

Intervention exists to preserve operator control without turning v1 into a manual orchestration console.

### 8.1 Supported Commands

- `pause <run-id|shard-id>`
- `resume <run-id|shard-id>`
- `retry <shard-id>`
- `reassign <shard-id> --runtime <codex|claude>`
- `merge approve <merge-unit-id>`
- `merge reject <merge-unit-id>`
- `stop <run-id>`

### 8.2 Command Meaning

`pause`
- requests a safe pause point and transitions through `pause requested` before `paused`

`resume`
- reactivates a run or shard that is paused or waiting for operator action

`retry`
- creates a new attempt for the shard and preserves previous attempt history

`reassign`
- keeps the shard intent but changes runtime routing

`merge approve` / `merge reject`
- acts on a concrete merge artifact or merge unit, not on the run as a whole

`stop`
- prevents further dispatch and moves the run toward a controlled terminal state

### 8.3 Guardrails

The CLI must enforce these guardrails:

- intervention should be idempotent from the operator's perspective
- commands should return one of `queued`, `acknowledged`, `applied`, or `failed`
- commands should not pretend to be synchronous when they are not
- v1 should not allow free-form shard topology edits mid-run
- every blocked state should have a clear resolution path
- command failures must state whether the reason is invalid state, missing id, policy denial, or runtime error

## 9. Acceptance Criteria for This CLI Design

The CLI design is successful if:

- one objective can start a run without teaching the user internal workflow machinery
- the current session remains the primary place to understand progress
- placement decisions are visible and justified
- intervention remains short and practical
- broader views exist but are not required for normal operation
- the workflow remains spec-driven under the hood without leaking unnecessary complexity

## 10. Open Constraints

This workspace currently contains only `PRD.md` and is not a git repository. As a result:

- this design is documented, but not committed
- no worktree-based planning or execution setup can be created yet
- implementation planning must assume repository bootstrapping is part of the work

The implementation stack is now fixed to Rust for the CLI/runtime core, with a local SQLite database as the persistence layer for operator-visible state.
