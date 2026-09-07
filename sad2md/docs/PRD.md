# Product Requirements Document (PRD) for sad2md

## How to use this template

Copy this file to `PRD.md` in the project or feature folder, then replace every `[bracketed placeholder]`. Delete a section only if it truly doesn't apply, don't leave it blank.

If you're an AI assistant filling in or reviewing requirements in this document, follow the rules in "Requirement format and rules" exactly. They exist so a requirement can be tested, traced, and implemented without a follow-up conversation to clarify what it meant.

## Document info

| Field | Value |
| --- | --- |
| Status | Draft |
| Version | [n] |

## Overview

### Problem statement

Describe the problem in one or two paragraphs: who has it, what it costs them today, and why it's worth solving now. Don't describe the solution here.

[problem statement]

### Goals

- [Goal 1: a measurable outcome, not an activity]
- [Goal 2]

### Non-goals

State what this PRD deliberately doesn't cover, so reviewers don't assume silence means "also required."

- [Non-goal 1]
- [Non-goal 2]

## Users and stakeholders

| User class | Needs from this feature |
| --- | --- |
| [e.g. End user] | [what they need] |
| [e.g. Support team] | [what they need] |

## Requirements

### Requirement format and rules

Every requirement in this document follows these rules. An AI assistant generating or editing requirements here must apply them without being asked again.

- **Statement shape**: `<WHO> shall <WHAT> <WHERE/WHEN> <CONSTRAINT>`. Example: "The billing service shall reject a charge above the account's credit limit within 200ms."
- **Verb meaning**: "shall" or "must" means required. "will" states a fact or future intent, not a requirement. "should" or "may" means optional, a goal rather than a requirement. Don't mix "shall" and "must" in the same document, a reader may infer a priority difference that isn't there.
- **One requirement per line**: if a requirement needs "and," "or," or a second verb to state, split it into two requirements.
- **Unique ID**: `<FEAT>-<TYPE>-<nnnnn>`, where `FEAT` identifies the feature or product area and `TYPE` is `FR` (Functional Requirement) for entries in Functional requirements or `AR` (Architectural Requirement) for entries in Non-functional requirements, for example `PLTF-FR-00001`. If a requirement changes enough to need a new discussion, give it a new ID rather than reusing the old one.
- **Verifiable**: state it so a reviewer can point to a test, a demonstration, or an inspection that proves it's met. If you can't think of how to verify it, the wording is probably too vague or missing information.
- **Design-independent**: say what the system must do, not how to build it. A screen mock-up or a hypothetical implementation can accompany a requirement to clarify intent, but don't let it replace the requirement itself.
- **No vague qualifiers**: avoid words like "fast," "user-friendly," "robust," "as much as practicable," "etc.," and "and/or" without a concrete definition alongside them. Each of these needs a number, a list, or an explicit rule instead. Example: replace "the response must be fast" with "the response must complete within 300ms at the 95th percentile."

### Functional requirements

| ID | Priority | Requirement | Rationale |
| --- | --- | --- | --- |
| [FEAT]-FR-[nnnnn] | [High / Medium / Low / Out] | [WHO shall WHAT WHERE/WHEN CONSTRAINT] | [why this matters, tie back to a goal] |

### Non-functional requirements

Cover performance, security, reliability, accessibility, and any other quality attribute the product depends on. Quantify every one, "secure" and "scalable" aren't verifiable on their own.

| ID | Priority | Requirement | Rationale |
| --- | --- | --- | --- |
| [FEAT]-AR-[nnnnn] | [High / Medium / Low / Out] | [quantified quality requirement] | [why this matters] |

## Constraints and assumptions

- **Constraints**: things outside anyone's control that bound the solution, for example a mandated technology, a regulatory requirement, or a hard deadline. Record why each constraint exists, not just that it exists.
- **Assumptions**: things taken as true without direct evidence, for example expected load, or that a dependency team ships on time. If an assumption turns out false, the requirements built on it need re-review.

| Type | Description | Why it applies |
| --- | --- | --- |
| [Constraint / Assumption] | [description] | [reason] |
