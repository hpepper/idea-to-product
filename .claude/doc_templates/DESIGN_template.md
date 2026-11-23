# DOCUMENT_NAME

## Introduction

### Purpose

<!-- 1-2 sentences: document purpose and the system or feature whose design decisions it records. -->

### Vocabulary

<!-- Bullet list of uncommon terms. Alphabetical. -->

### References

<!-- Bullet list of external links/references.
Include the PRD, SRS, or FIP the decisions trace to,
and the ICD that describes component interconnection. -->

### Reading this document

Three organizational levels:

- Introduction (top)
  - Purpose - one sentence on the system and the decisions recorded.
  - Vocabulary - glossary of uncommon terms.
  - References - requirement documents, ICD, external standards.
  - Overview - scope, decision drivers, and a decision log table.
- Decision sections (bulk)
  - One section per decision, same pattern:
    - Context: the problem and the forces that shape it.
    - Drivers: requirements and quality attributes the decision must satisfy.
    - Options: each alternative with pros and cons.
    - Decision: the chosen option.
    - Rationale: why the chosen option beats the alternatives.
    - Consequences: positive, negative, and follow-up work.
- Component interconnection lives in the ICD, not here.

### Overview

#### Design scope

<!-- 2-4 sentences: what the decisions cover and what they exclude.
Link the ICD for interfaces and component interconnection. -->

#### Decision drivers

<!-- Table of requirements and quality attributes that drive the decisions. Columns:
  ID - requirement ID, for example FEAT-AR-nnnnn.
  Driver - ≤10-word summary.
  Source - link to PRD, SRS, or FIP section.
  Rows sorted alphabetically by ID.
  -->

#### Decision log

<!-- Table of all decisions. Columns:
  ID - DD-nnn.
  Title - ≤10-word summary.
  Status - Proposed | Accepted | Deprecated | Superseded by DD-nnn.
  Date - YYYY-MM-DD of last status change.
  Reference - link to decision section.
  Rows sorted alphabetically by ID.
  -->

<!-- Repeat the section below per decision. Order by ID. -->

## DD-nnn: [Decision Title]

### DD-nnn Status

<!-- Proposed | Accepted | Deprecated | Superseded by DD-nnn. Add date and decision owner. -->

### DD-nnn Context

<!-- The problem, constraints, and forces at play. State facts, not the solution. -->

### DD-nnn Drivers

<!-- Bullet list of requirement IDs and quality attributes this decision must satisfy.
Link each to the Decision drivers table. -->

### DD-nnn Options

<!-- Table of considered options. Columns:
  Option - short name.
  Pros - bullet-style phrases.
  Cons - bullet-style phrases.
  Rows sorted alphabetically by Option.
  -->

### DD-nnn Decision

<!-- One or two sentences: the chosen option, stated as an imperative. -->

### DD-nnn Rationale

<!-- Why the chosen option beats the rejected ones against the drivers. -->

### DD-nnn Consequences

<!-- Bullet list: positive outcomes, negative outcomes, risks, and follow-up work.
Note any ICD or requirement changes the decision causes. -->
