## Why

The project has OpenSpec workflow skills but lacks a focused skill for building and integrating Tauri v2 applications. Adding this skill now will standardize implementation steps, reduce setup mistakes, and speed up delivery for desktop app features.

## What Changes

- Add a new reusable Codex skill dedicated to Tauri v2 application work.
- Define a clear workflow for Tauri v2 project setup, command usage, and architecture conventions.
- Include guidance for frontend/backend boundary design between web UI and Tauri Rust commands.
- Add verification steps for build/run/package flows so generated work is implementation-ready.

## Capabilities

### New Capabilities
- `tauri-v2-app-skill`: Provide structured instructions and templates for implementing or updating Tauri v2 applications in this repository.

### Modified Capabilities
- None.

## Impact

- Affected areas: `.codex/skills/` (new skill directory and documentation), project docs referencing workflow usage.
- Developer workflow: improved consistency for Tauri v2 tasks and fewer ad-hoc implementation patterns.
- Dependencies/systems: no runtime dependency changes; this is a process and documentation capability addition.
