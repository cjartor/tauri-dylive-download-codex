## ADDED Requirements

### Requirement: Skill defines Tauri v2 workflow entry criteria
The system SHALL provide a `tauri-v2-app-skill` capability that is triggered for requests involving creation, update, or troubleshooting of Tauri v2 applications, and SHALL require confirmation of scope before implementation begins.

#### Scenario: Trigger on Tauri v2 implementation request
- **WHEN** a user asks to build or modify a Tauri v2 application feature
- **THEN** the skill workflow is selected and starts with scope and constraints clarification

### Requirement: Skill prescribes Tauri v2 architecture boundaries
The system SHALL instruct implementers to separate frontend UI concerns from Rust command handlers, and MUST define how command names, payload contracts, and error handling are coordinated across that boundary.

#### Scenario: Define command contract before wiring UI calls
- **WHEN** a task requires frontend-to-Rust interaction in a Tauri v2 app
- **THEN** the skill requires explicit command contract definition including command identifier, input shape, and response/error behavior

### Requirement: Skill requires verification for run and build workflows
The system SHALL require validation steps that cover local run behavior and production build/package behavior for affected Tauri v2 changes.

#### Scenario: Validate a completed Tauri v2 change
- **WHEN** implementation is completed using the skill
- **THEN** the output includes verification steps for development execution and release build/package commands, plus reporting of any unexecuted checks

### Requirement: Skill outputs actionable implementation guidance
The system SHALL produce concrete implementation steps, file targets, and acceptance checks rather than only conceptual advice.

#### Scenario: Produce implementation-ready guidance
- **WHEN** the skill responds to a Tauri v2 development request
- **THEN** the guidance includes ordered actions, expected file touchpoints, and clear done criteria suitable for direct execution
