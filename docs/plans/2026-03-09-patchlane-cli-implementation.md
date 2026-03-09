# Patchlane CLI Implementation Plan

**Date:** 2026-03-09
**Status:** Approved baseline copied into the Task 1 worktree

## Goal

Bootstrap a real Rust repository for Patchlane with a thin CLI application, a minimal library surface, and planning documents that anchor the next implementation tasks.

## Task 1 Scope

- initialize the repository baseline in the worktree
- add a failing bootstrap smoke test first
- confirm the red phase with `cargo test bootstrap`
- add the minimal crate surface needed to pass the bootstrap check
- keep the CLI skeleton intentionally thin

## Immediate Follow-on Tasks

- define the `swarm` command topology with `clap`
- add explicit run and shard state models
- implement deterministic `run`, `status`, and `watch` output contracts
- layer in placement, persistence, and intervention behavior incrementally
