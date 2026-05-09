# AGENTS.md — Junie Guidelines for Odyssée Kids

These guidelines apply to the `Odyssée Kids` project. They complement Junie's general rules and must be followed before modifying the repository.

## 1. Think Before Coding

**Do not assume. Do not hide uncertainty. Surface tradeoffs.**

Before implementing:

- State your assumptions explicitly. If a request is ambiguous, ask a targeted question.
- If multiple interpretations exist, present them instead of silently choosing one.
- If a simpler approach is enough, prefer it and explain why.
- If something is blocking or contradictory, stop, name the issue, and ask for clarification.

## 2. Simplicity First

**The minimum code that solves the need. Nothing speculative.**

- Do not add any feature that was not requested.
- Do not introduce an abstraction for single-use code.
- Do not add flexibility, configuration, or generalization that was not requested.
- Do not add error handling for scenarios that are impossible in the current context.
- If a solution becomes long or complex, return to a more direct version.

Control question: **would a senior engineer consider this solution over-engineered?** If yes, simplify it.

## 3. Surgical Changes

**Touch only what is necessary. Clean up only what your changes make unnecessary.**

When modifying existing code:

- Do not "improve" adjacent code, comments, or formatting without a direct link to the request.
- Do not refactor what is not broken.
- Respect the local style, even if you would do it differently.
- If you notice unrelated dead code or an unrelated issue, mention it instead of removing it.

When your changes create orphaned elements:

- Remove imports, variables, or functions made unused by your own changes.
- Do not remove pre-existing dead code unless explicitly requested.

Mental test: every changed line must be directly traceable to the user's request.

## 4. Goal-Driven Execution

**Define verifiable success criteria and loop until verified.**

Turn tasks into testable goals:

- "Add validation" → write or identify tests for invalid inputs, then make them pass.
- "Fix a bug" → reproduce the bug with a test or script, then verify the fix.
- "Refactor X" → identify the relevant existing tests and verify they pass after the change.

For multi-step tasks, prepare a brief plan:

```text
1. [Step] → verify: [check]
2. [Step] → verify: [check]
3. [Step] → verify: [check]
```

Weak success criteria such as "make it work" require clarification.

## 5. Project-Specific Context

- Project: `Odyssée Kids`, a local-first MVP for an interactive encyclopedia for children.
- Frontend: `Vue 3`, `Vite`, `Tailwind CSS`, `Lucide Icons`, `vue-i18n`.
- Desktop/backend: `Tauri 2` with a `Rust` backend.
- Local storage: `SQLite` via `rusqlite`.
- Optional local inference: `llama.cpp` and `stable-diffusion.cpp` sidecars.
- The product must remain local-first: do not introduce cloud calls, remote telemetry, or network dependencies without an explicit request.
- Preserve child-safety guarantees: prompts must be factual, kind, educational, and free from inappropriate content.
- The no-model mode must keep working through deterministic offline fallbacks.

## 6. Backend Areas to Respect

- `src-tauri/src/commands.rs`: orchestration of Tauri commands.
- `src-tauri/src/constants.rs`: prompt, model, and storage constants.
- `src-tauri/src/generation.rs`: prompt construction and offline fallback rendering.
- `src-tauri/src/hardware.rs`: hardware and accelerator detection.
- `src-tauri/src/models.rs`: shared DTO contracts with the frontend.
- `src-tauri/src/paths.rs`: runtime path resolution.
- `src-tauri/src/seeds.rs`: deterministic catalog data.
- `src-tauri/src/sidecars.rs`: external binary adaptation.
- `src-tauri/src/storage.rs`: SQLite schema, seeds, catalog queries, and cache.

Respect each file's responsibilities. Do not move logic between modules without a direct reason.

## 7. Verification Commands

Use PowerShell on Windows and the existing `npm` scripts:

```powershell
npm run build:web
npm run test:rust
npm run check
```

- For a frontend change, verify at minimum with `npm run build:web` unless the task is purely documentary.
- For a Rust/backend change, verify with `npm run test:rust`.
- For a cross-cutting frontend + backend change, prefer `npm run check`.
- For documentation-only changes, tests may be omitted, but state that clearly.

## 8. Local Runtime and Sidecars

- Do not assume local models or binaries exist.
- Runtime paths can be inspected with:

```powershell
npm run paths:runtime
```

- The expected layout contains `bin`, `models`, `cache/images`, and `odyssee.sqlite` under `%APPDATA%\Odyssee` on Windows.
- Do not automatically download large models and do not change expected sidecar file names without an explicit request.

## 9. Respect the Existing Working Tree

- The repository may already contain user changes unrelated to the task.
- Inspect diffs before modifying an already changed file if context is needed.
- Never restore, overwrite, or reformat changes you did not create.
- Files in `.junie` are reserved for Junie guidelines and configuration; do not use it as a temporary folder.