## Context

This repository uses OpenSpec artifacts to drive implementation, but there is no dedicated skill that guides Tauri v2 application delivery end-to-end. Contributors currently rely on ad-hoc instructions, which causes inconsistent project layout, command invocation, and verification coverage across Tauri tasks.

## Goals / Non-Goals

**Goals:**
- Define a reusable skill focused on Tauri v2 application development and integration in this workspace.
- Standardize expected workflow steps: discovery, implementation, validation, and handoff.
- Capture concrete conventions for Tauri v2 command interfaces, frontend/Rust boundaries, and packaging checks.
- Ensure generated guidance is directly actionable for future `/opsx:apply` execution.

**Non-Goals:**
- Building or shipping a full Tauri application as part of this change.
- Changing runtime product behavior outside of adding skill guidance.
- Replacing existing OpenSpec lifecycle skills (`propose`, `apply`, `archive`).

## Decisions

### Decision: Add a standalone skill under `.codex/skills/` for Tauri v2 work
The change will introduce a dedicated skill package with a `SKILL.md` that describes trigger conditions, workflow, guardrails, and output expectations.

Alternative considered:
- Extend an existing generic implementation skill. Rejected because Tauri v2 has specific architecture and tooling concerns (Rust commands, config, bundling) that deserve explicit instructions.

### Decision: Encode workflow as phase-based instructions
The skill will guide users through phases: scope confirmation, project/bootstrap checks, implementation patterns, and verification (dev/build/package).

Alternative considered:
- Keep instructions as a short checklist only. Rejected because it does not provide enough decision support for boundary design and validation standards.

### Decision: Include validation criteria tied to Tauri v2 command surface
The skill will require verification of command registration, request/response payload handling, frontend invocation paths, and packaging commands where applicable.

Alternative considered:
- Validate only compile success. Rejected because compile-only checks miss integration failures between web UI and Rust commands.

## Risks / Trade-offs

- [Risk] Skill becomes too prescriptive for varied Tauri app structures.  
  Mitigation: define required outcomes and examples, while allowing project-specific adaptation.
- [Risk] Guidance drifts from actual Tauri v2 CLI/config behavior over time.  
  Mitigation: keep version assumptions explicit and update skill when toolchain changes.
- [Risk] Additional documentation overhead.  
  Mitigation: keep the skill concise and task-oriented, with clear entry/exit criteria.

## Migration Plan

1. Add the new skill folder and `SKILL.md` with Tauri v2-focused instructions.
2. Link the skill in repository guidance where skills are enumerated.
3. Use the skill for the next relevant Tauri implementation task and confirm usability.
4. Iterate wording if gaps are found during first practical use.

Rollback strategy:
- Remove the new skill directory and related references if it causes confusion or conflicts with existing workflows.

## Open Questions

- Should the skill enforce a single frontend stack assumption, or remain framework-agnostic?
- Should package signing/notarization steps be mandatory or optional in baseline guidance?
