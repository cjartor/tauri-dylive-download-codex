---
name: tauri-v2-app-skill
description: Implement or update Tauri v2 applications with clear scope checks, frontend/Rust command boundaries, and verification steps for run/build/package flows.
license: MIT
compatibility: Requires Tauri v2 toolchain in the target project.
metadata:
  author: cjpro
  version: "1.0"
---

Implement Tauri v2 app work with a consistent, execution-ready workflow.

## When To Use

Use this skill when the user asks to:
- Create a new Tauri v2 app or module
- Add or modify a Tauri command or invoke flow
- Troubleshoot Tauri v2 integration issues across frontend and Rust

If the request is not Tauri v2 related, use a different skill.

## Workflow

### Phase 1: Confirm Scope and Constraints

1. Confirm target app/module and expected outcome.
2. Confirm runtime assumptions: OS targets, dev/build requirements, and whether packaging is in scope.
3. Identify touched surfaces before coding:
   - Frontend files invoking commands
   - Rust command handlers and registration points
   - Tauri config/build settings if impacted

### Phase 2: Plan Implementation

1. Define command contracts before wiring code:
   - Command name
   - Input payload shape
   - Response shape and error mapping
2. Decide ownership boundary:
   - Frontend: view state, invocation orchestration, user-facing errors
   - Rust: side effects, system access, validation, typed responses
3. List concrete file touchpoints in execution order.

### Phase 3: Implement

1. Add/update Rust commands and register them in the Tauri invocation surface.
2. Add/update frontend invocation code with explicit payload typing and error handling.
3. Keep changes minimal and local to requested behavior.

### Phase 4: Verify and Report

Run only checks that are relevant and available in the target project:
- Development run check (example: `npm run tauri dev`)
- Production build/package check (example: `npm run tauri build`)
- Command-path smoke check (at least one call path from UI to Rust and back)

If any check is skipped, explicitly report:
- Which check was skipped
- Why it was skipped
- What risk remains

## Guardrails

- Always confirm scope before implementation.
- Do not blur boundaries between UI logic and Rust command logic.
- Use clear, stable command names; avoid ambiguous naming.
- Document exact file touchpoints and why each is changed.
- Prefer deterministic error messages and typed payload handling.
- If architecture uncertainty appears, pause and ask for clarification before continuing.

## Output Expectations

Responses should include:
1. Ordered implementation steps
2. Concrete files to edit
3. Done criteria and validation checklist
4. Note of any unexecuted checks and residual risk

## Dry-Run Prompt (Reference)

Sample request:
"Add a Tauri v2 command to read app version from Rust and show it in settings."

Expected behavior from this skill:
- Confirm where settings UI lives and where command registration is done.
- Define command contract (`get_app_version` -> `{ version: string }` or string).
- Identify exact frontend and Rust files to touch.
- Provide run/build/package checks and call out any skipped checks.
