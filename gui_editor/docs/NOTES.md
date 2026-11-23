# GUI Editor Agent Notes

## Introduction

### Purpose

Keeps the agent command lines and the product owner handoff for `gui_editor`.
The text moved here from `README.md`, which holds end-user documentation only.

### Vocabulary

- **Agent:** a Claude Code subagent started with `claude --agent <name>`.
- **Handoff:** the message that passes a story and task to the next agent.

### References

- [PLAN.md](PLAN.md): the approved plan with task T-01.
- [REQUIREMENTS.md](REQUIREMENTS.md): the `GEDT-AR` requirements.
- [USER_STORIES.md](USER_STORIES.md): the `GEDT-STORY` user stories.

### Overview

The commands below run the agent pipeline from opportunity assessment to implementation.
The transcript records the product owner approval and the handoff for task T-01.

## Agent Commands

```sh
claude --agent business-analyst "Process gui_editor/docs/02_opportunity_assesmemt.md into gui_editor/docs/USER_STORIES.md"
claude --agent architect "Derive requirements from gui_editor/docs/USER_STORIES.md into gui_editor/docs/REQUIREMENTS.md"
claude --agent planner "Plan gui_editor from gui_editor/docs/USER_STORIES.md and gui_editor/docs/REQUIREMENTS.md and write to gui_editor/docs/PLAN.md"
claude --agent orchestrator "Implement task T-01 (GEDT-STORY-00019) from gui_editor/docs/PLAN.md. The plan is approved; skip business-analyst, architect, and planner. Start at the developer stage, then tester and reviewer. Read gui_editor/docs/USER_STORIES.md and gui_editor/docs/REQUIREMENTS.md for the story and AR-00024, AR-00025, AR-00030."
```

## Product Owner Handoff

```text
Product owner decision (approved)

You approved the plan as proposed, so nothing changed:
- MVP: components only.
- Increment order: INC-1 to INC-6 as listed.
- Deferred: GEDT-STORY-00013 stays deferred.
- Next action: start T-01.

Handoff to the developer agent

I can't start the developer agent from here. Pass it this:

- Story: GEDT-STORY-00019, "As a maintainer, I want a native Linux app so that I edit SADs on my workstation".
- Task: T-01, build pipeline.
- Increment: MVP.
- Requirements: GEDT-AR-00024, GEDT-AR-00025 and GEDT-AR-00030 (CI part only). GEDT-AR-00003 and GEDT-AR-00004 follow in T-02.
- Depends on: nothing.
- Size: M.
- Definition of done:
  - gui_editor/build.sh builds the gui_editor binary instead of amber-dos.
  - The Rust container image is pinned by version and digest, not rust:latest.
  - The build runs cargo build --locked against the committed Cargo.lock.
  - cargo audit and cargo deny run, and the build fails on any finding not explicitly ignored.
  - A secret scan runs and finds nothing.
  - CI runs the gui_editor, sad_xml_sql, sad2md and tui_editor test suites.
  - .cargo-home/ is in .gitignore for gui_editor and aws2xml.
- Before starting: commit the pending changes in sad_xml_sql/src/db_populate_from_xml.rs.

Next step

Hand off to developer: GEDT-STORY-00019, task T-01.

