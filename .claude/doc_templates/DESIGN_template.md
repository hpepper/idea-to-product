# DOCUMENT_NAME

## Introduction

### Purpose

<!-- 1-2 sentences: document purpose. -->

### Vocabulary

<!-- Bullet list of uncommon terms. Alphabetical. -->

### References

<!-- Bullet list of external links/references. -->

### Reading this document

Three organizational levels:

- Introduction (top)
  - Purpose - one sentence on the app.
  - Vocabulary - glossary (EKS, pipeline, namespace, etc.).
  - References - external libs/APIs.
  - Overview - system bird's-eye: 2 diagrams + 2 tables.
    - Context diagram: app vs outside world (browser, OAuth proxy, GitLab, Kubernetes).
    - Component interconnection diagram: internal pieces.
- Component sections (bulk)
  - One section per component, same pattern:
    - Short prose: what component does.
    - Decomposition table: subcomponents/files.
    - Context diagram (optional): external interactions.
    - Communication diagram + table (optional): protocols, endpoints, purposes.

### Overview

#### Primary components decomposition

<!-- Table of top-level components. Columns:
  Name - component name.
  Description - ≤10-word summary.
  Reference - link to component section.
  -->

#### Overview context diagram

<!-- Mermaid: this project = 'the work' (rectangle); connected entities = circles. Show both directions.
Table columns:
  Entity - component/connector name.
  Type - 'The Work' | Adjacent | Connector.
  Description - ≤10-word summary.
  Reference - external link.
  -->

#### Primary components interconnection

<!-- Mermaid: top-level components connected. Lines without arrows; name lines where relevant.
Table columns:
  Entity - component/connector name.
  Type - Component | Connector.
  Description - ≤10-word summary (connectors: include protocol if relevant).
  Reference - optional link to sub-section.
  -->

<!-- Repeat sections below per high-level component. Alphabetical by name. -->

## Component: [Component Name]

<!-- Short purpose summary. -->

### [Component Name] Decomposition

<!-- Optional. List subcomponents if any. -->

### [Component Name] Context diagram

<!-- Optional. Show components this one interacts with. -->

### [Component Name] communication

<!-- Optional. Show connected components + protocol. -->
