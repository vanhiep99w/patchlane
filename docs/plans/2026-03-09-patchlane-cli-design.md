# Patchlane CLI Design

**Date:** 2026-03-09
**Status:** Approved baseline copied into the Task 1 worktree

## Purpose

Patchlane should present as a command-native swarm tool inside a coding-agent instance. The CLI is the primary product surface for submitting an objective, inspecting the plan, watching progress, and intervening only when needed.

## Initial Command Surface

The approved v1 command surface is:

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

## Design Baseline

- `swarm run` is the primary entrypoint.
- Operator-visible output should favor terminal readability and compact progress states.
- Patchlane orchestrates; `superpowers` structures the internal workflow.
- Local SQLite-backed run state is the v1 source of truth.
