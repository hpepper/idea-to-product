# User Stories

## Introduction

### Purpose

Story cards for the GUI editor, an editor for Software Architecture Documentation (SAD) data.
Readers are the product owner, the architect, and the developers of `gui_editor`.

### References

- Initial charter: [01_initial_charter.md](01_initial_charter.md)
- Opportunity assessment, source of the VoC entries: [02_opportunity_assesmemt.md](02_opportunity_assesmemt.md)
- SAD concepts: [SoftwareArchitectureDocumentationOverview.md](../../docs/SoftwareArchitectureDocumentationOverview.md)
- SAD XML example: [template_sad.xml](../../template_sad.xml)
- SAD XML schema: [software_architecture_documentation.dtd](../../sad_xml_sql/tests/software_architecture_documentation.dtd)

### Vocabulary

- Architecture story: As an architect I need to <change> to <quality goal>.
- Component: Entity of the documented system, for example a service or a message broker.
- Edit mode: Editor state where the user changes project data.
- Maintainer: Person who writes and updates the SAD of a project.
- Primary component: Component that a viewpacket describes.
- Project: One SAD, stored in one XML file.
- Reader: Person who uses the SAD to understand an architecture.
- SAD: Software Architecture Documentation.
- Story card: Captured idea with its root reason, before requirements exist.
- Technical debt story: As a developer I need to <change> to <quality goal>.
- User story: As a <role>, I want <goal> so that <purpose>.
- View mode: Read-only editor state for exploring the architecture.
- Viewpacket: Smallest cohesive bundle of documentation about one aspect of a component.
- VoC: Voice of the Customer.

## Product Context

### Goal

Maintainers update the SAD without hand-editing XML.
The gemba for adding a component today has seven manual XML steps,
including copying a template, incrementing an ID, and sorting the list by hand.
Each manual step is a source of invalid IDs, broken references, and invalid XML.

Proposed measurable outcomes, pending confirmation, see open question 1:

- Adding a component takes zero manual XML edits.
- Every file saved by the editor validates against the SAD DTD.

### Stakeholders

- Maintainer: Primary user in edit mode.
- Reader: Development teams and new stakeholders, primary user in view mode.

### Scope

- In scope:
  - Native Linux app, first release.
  - Local web app for a single user on the local machine, later release.
  - Edit and view of components, viewpackets, and diagrams.
  - Multiple projects, with cross-project component references.
- Out of scope:
  - Company-wide multi-user service, including authentication and roles.
  - Other storage formats than the SAD XML.
  - Native app on platforms other than Linux.
- Constraints:
  - The SAD XML file is the storage format and the source of truth.
  - Saved files stay usable by existing tools, for example `sad2md` and `sad_xml_sql`.
  - Each project is stored in its own XML file.
  - Implementation language is Rust, from the voice of the developer.
  - UI toolkit is egui, because egui supports both native and web apps.
  - Native Linux app ships before the web app, from the voice of the business.
- Assumptions:
  - Loaded project data is held in an in-memory SQLite database for search and update,
    from the voice of the developer.
    The architect confirms or replaces this design.
  - One user edits a project at a time.
  - Version history and audit come from the user's version control, for example git.
  - Project files can contain sensitive infrastructure details, for example AWS account IDs.

## Storage

### Project Files

#### GEDT-STORY-00001 - As a maintainer, I want to open a project XML file so that I can work on an existing SAD

- Description: Load a SAD XML file into the editor.
- VoC: Save in common storage format (currently XML).
- Priority: Critical
- Root reason: Existing projects hold all current architecture data.
- What if not done: The editor is usable only for new projects.
- Acceptance criteria:
  - Opening a file that validates against the SAD DTD shows all its components and viewpackets.
  - Opening an invalid file shows an error naming the file and the failing location.
  - Opening an invalid file leaves the file unchanged.
- Dependent on: GEDT-STORY-00021
- Requirements: GEDT-AR-00001, GEDT-AR-00012, GEDT-AR-00013, GEDT-AR-00027, GEDT-FR-00001,
  GEDT-FR-00002, GEDT-FR-00003

#### GEDT-STORY-00002 - As a maintainer, I want to save the project to its XML file so that other SAD tools can use my changes

- Description: Write the project back to the SAD XML file.
- VoC: Save in common storage format (currently XML).
- Priority: Critical
- Root reason: The XML file feeds document generation, so edits must reach the file.
- What if not done: Edits are lost when the editor closes.
- Acceptance criteria:
  - A saved file validates against the SAD DTD.
  - Opening, then saving without edits, keeps every component, viewpacket, and relation.
  - `sad2md` processes a file saved by the editor without errors.
- Dependent on: GEDT-STORY-00001, GEDT-STORY-00022
- Requirements: GEDT-AR-00001, GEDT-AR-00002, GEDT-AR-00030, GEDT-FR-00004, GEDT-FR-00005,
  GEDT-FR-00006, GEDT-FR-00009

#### GEDT-STORY-00003 - As a maintainer, I want the editor to save automatically so that I lose at most one minute of work

- Description: Save unsaved changes at a fixed interval.
- VoC: Automatic save every x minutes, probably every 1 minute as native app.
- Priority: Value adding
- Root reason: Crashes or forgotten saves cost the maintainer rework.
- What if not done: The maintainer can lose a full editing session.
- Acceptance criteria:
  - With unsaved changes, the editor saves within 60 seconds of the first unsaved change.
  - Without unsaved changes, the editor does not rewrite the file.
- Dependent on: GEDT-STORY-00002
- Requirements: GEDT-AR-00007, GEDT-AR-00029, GEDT-FR-00007, GEDT-FR-00008, GEDT-FR-00009

## Editing

### Components

#### GEDT-STORY-00004 - As a maintainer, I want to add a component without editing XML so that I avoid ID and ordering mistakes

- Description: Create a component through a form.
- VoC: Have an edit mode, where aspects of the architecture can be entered.
  Gemba, manually adding a new component.
- Priority: Critical
- Root reason: The manual process has seven steps, and each step can introduce an error.
- What if not done: The editor gives no gain over hand-editing XML for the most common task.
- Acceptance criteria:
  - The new component gets an ID that no other component in the project uses.
  - The saved component list is in alphabetical order by name.
  - Creating a component with an existing name in the same project shows an error.
- Dependent on: GEDT-STORY-00002
- Requirements: GEDT-FR-00010, GEDT-FR-00011, GEDT-FR-00012, GEDT-FR-00013

#### GEDT-STORY-00005 - As a maintainer, I want to edit an existing component so that the SAD stays current

- Description: Change the fields of a component.
- VoC: Type of edit views: Component.
- Priority: Critical
- Root reason: Architecture changes over time, and the SAD must follow.
- What if not done: Maintainers fall back to hand-editing XML for every change.
- Acceptance criteria:
  - Changing a component name updates every reference to that component.
  - The component ID does not change on edit.
- Dependent on: GEDT-STORY-00004
- Requirements: GEDT-FR-00013, GEDT-FR-00014, GEDT-FR-00015, GEDT-FR-00016

### Viewpackets

#### GEDT-STORY-00006 - As a maintainer, I want to create and edit viewpackets so that I can describe each aspect of a component

- Description: Create a viewpacket, set its type and style, and edit its sections.
- VoC: Type of edit views: Viewpacket.
- Priority: Critical
- Root reason: Viewpackets carry the SAD content, see the SAD overview.
- What if not done: Only components are editable, so most SAD content stays manual.
- Acceptance criteria:
  - A new viewpacket references an existing component as its primary component.
  - The user can select only viewpacket types and styles that the SAD DTD allows.
  - A new viewpacket gets an ID that no other viewpacket in the project uses.
- Dependent on: GEDT-STORY-00004
- Requirements: GEDT-FR-00011, GEDT-FR-00017, GEDT-FR-00018, GEDT-FR-00019

### Diagrams

#### GEDT-STORY-00007 - As a maintainer, I want to edit the diagram of a viewpacket so that I can show how components relate

- Description: Add and remove components and relations in a viewpacket diagram.
- VoC: Type of edit views: Diagram.
- Priority: Critical
- Root reason: Relations between components are the core of each viewpacket style.
- What if not done: Diagrams stay hand-edited, and broken relations go unnoticed.
- Acceptance criteria:
  - The user can edit diagrams of these styles:
    client-server, context, decomposition, deployment, installation, uses.
  - A relation references only components that exist in the loaded projects.
  - Removing a component from a diagram keeps the component in the project.
- Dependent on: GEDT-STORY-00006
- Requirements: GEDT-FR-00011, GEDT-FR-00020, GEDT-FR-00021, GEDT-FR-00022

#### GEDT-STORY-00008 - As a maintainer, I want to search for components while editing a diagram so that I find the right component quickly

- Description: Filter the component list by name while adding components to a diagram.
- VoC: When editing diagrams make it easy to find the components that I want to add.
- Priority: Value adding
- Root reason: Large projects have many components, and scrolling a long list is slow.
- What if not done: Adding components to diagrams is slow in large projects.
- Acceptance criteria:
  - Typing part of a component name lists only components whose name contains the text.
  - The search ignores letter case.
  - Search text containing `'`, `%`, or `_` matches those characters literally.
- Dependent on: GEDT-STORY-00007
- Requirements: GEDT-AR-00016, GEDT-AR-00028, GEDT-FR-00024

#### GEDT-STORY-00009 - As a maintainer, I want to add replicated instances as separate components so that a diagram shows each replica

- Description: Model replicas, for example ETCD1, ETCD2, and ETCD3, as separate components.
- VoC: TODO how to handle having e.g. ETCD1-3 as components in a diagram?
- Priority: Value adding
- Root reason: Clustered services run as several instances, and diagrams must show them.
- What if not done: Diagrams of clustered services are incomplete.
- Acceptance criteria:
  - A diagram shows ETCD1, ETCD2, and ETCD3 as three distinct components.
  - Each replica has its own relations.
- Dependent on: GEDT-STORY-00007
- Requirements: GEDT-FR-00023

#### GEDT-STORY-00010 - As a reader, I want to filter the client-server primary diagram so that I see only the relevant components

- Description: Hide diagram components that do not match a filter, for example API.
- VoC: Be able to filter on e.g. API in the ClientServer primary diagram.
- Priority: Value adding
- Root reason: Client-server diagrams of large systems are too dense to read whole.
- What if not done: Readers must read the full diagram to find one interaction.
- Acceptance criteria:
  - Applying the filter API shows only API components and their direct relations.
  - Clearing the filter shows the full diagram.
  - Filtering does not change the saved project.
- Dependent on: GEDT-STORY-00007
- Requirements: GEDT-FR-00025, GEDT-FR-00026

## Viewing

### Modes

#### GEDT-STORY-00011 - As a reader, I want a view mode so that I can explore the architecture without changing it

- Description: Read-only mode for exploring components, viewpackets, and diagrams.
- VoC: Have a view mode, where the user can use the app to get an understanding.
- Priority: Value adding
- Root reason: Readers need the SAD content without the risk of accidental edits.
- What if not done: Readers risk changing the project while reading.
- Acceptance criteria:
  - In view mode, no control changes project data.
  - Switching between view mode and edit mode keeps the current selection.
- Dependent on: GEDT-STORY-00001
- Requirements: GEDT-FR-00026, GEDT-FR-00027, GEDT-FR-00028

### Component Overview

#### GEDT-STORY-00012 - As a reader, I want to see all viewpackets of a component in one view so that I understand the component as a whole

- Description: List and show every viewpacket where the component is the primary component.
- VoC: Have a view that shows the combined information of a component.
- Priority: Value adding
- Root reason: The SAD splits a component over several viewpackets, which hides the whole picture.
- What if not done: Readers must find each viewpacket of a component by hand.
- Acceptance criteria:
  - Selecting a component lists every viewpacket with that component as primary.
  - The list excludes viewpackets where the component is not primary.
- Dependent on: GEDT-STORY-00006
- Requirements: GEDT-FR-00029

#### GEDT-STORY-00013 - As a reader, I want to see viewpackets that mention a component so that I know where else the component appears

- Description: List viewpackets where the component appears but is not the primary component.
- VoC: TODO do I want a view where I can also see all the viewpackets where the component is mentioned.
- Priority: Nice to have
- Root reason: Impact of a component change reaches every viewpacket that mentions the component.
- What if not done: Readers miss related viewpackets when assessing a change.
- Acceptance criteria:
  - The list shows viewpackets that reference the component in a diagram or relation.
  - The list is separate from the primary viewpacket list.
- Dependent on: GEDT-STORY-00012
- Requirements: GEDT-FR-00030

### Layout

#### GEDT-STORY-00014 - As a user, I want to split the window into panes so that I see several aspects of the architecture at once

- Description: Show different views side by side in one window.
- VoC: Be able to split the window into multiple panes.
- Priority: Value adding
- Root reason: Understanding a component needs several viewpackets at the same time.
- What if not done: Users switch back and forth between views and lose context.
- Acceptance criteria:
  - The user can show at least two views side by side in one window.
  - Each pane shows its content independently of the other panes.
- Dependent on: GEDT-STORY-00011
- Requirements: GEDT-FR-00026, GEDT-FR-00031

#### GEDT-STORY-00015 - As a user, I want linked windows across screens so that a selection in one window updates the others

- Description: Open several windows, and propagate a selection to every window.
- VoC: Multi screen support. Selecting something on one screen affects the other screens.
- Priority: Value adding
- Root reason: Users with several monitors want an overview across all of them.
- What if not done: Each window needs manual navigation to the same item.
- Acceptance criteria:
  - The user can open at least two windows on the same project.
  - Selecting a component in one window shows that component in every other window
    within 1 second.
- Dependent on: GEDT-STORY-00014
- Requirements: GEDT-FR-00032, GEDT-FR-00033

## Multiple Projects

### Project Set

#### GEDT-STORY-00016 - As a maintainer, I want to open several projects at once so that I work across related SADs

- Description: Load several project XML files in one editor session.
- VoC: Support multiple projects. Each project is stored in its own XML file.
- Priority: Value adding
- Root reason: Large systems span several SADs, for example core AWS and telemetry.
- What if not done: Maintainers switch editor sessions to work across projects.
- Acceptance criteria:
  - The user can open at least two project files in one session.
  - Saving writes each project only to its own file.
- Dependent on: GEDT-STORY-00002
- Requirements: GEDT-FR-00034, GEDT-FR-00035

### Cross-Project References

#### GEDT-STORY-00017 - As a maintainer, I want to reference a component from another project in a viewpacket so that I avoid duplicate components

- Description: Use a component of project A in a viewpacket of project B.
- VoC: Support referencing components in one project in a viewpacket in another project.
- Priority: Value adding
- Root reason: Copying shared components, for example AWS accounts, into every project creates drift.
- What if not done: Shared components are duplicated, and copies drift apart.
- Acceptance criteria:
  - A viewpacket in project B can reference a component of project A.
  - The saved reference identifies both the project and the component.
  - A reference to a project that is not loaded shows as unresolved, and the reference is kept.
- Dependent on: GEDT-STORY-00016
- Requirements: GEDT-FR-00036, GEDT-FR-00037, GEDT-FR-00038

#### GEDT-STORY-00018 - As a reader, I want one view with components from several projects so that I see the combined architecture

- Description: Show components of several loaded projects in the same view.
- VoC: Including components from multiple projects in the same view display.
- Priority: Value adding
- Root reason: Interactions between projects are invisible when each project is shown alone.
- What if not done: Readers combine views of several projects in their heads.
- Acceptance criteria:
  - A view shows components from at least two loaded projects.
  - Each component shows the project the component belongs to.
- Dependent on: GEDT-STORY-00017
- Requirements: GEDT-FR-00039, GEDT-FR-00040

## Platforms

### Native App

#### GEDT-STORY-00019 - As a maintainer, I want a native Linux app so that I edit SADs on my workstation

- Description: First release of the editor as a native Linux desktop app.
- VoC: Implement as native app on Linux.
- Priority: Critical
- Root reason: Maintainers work on Linux workstations, with project files on local storage.
- What if not done: No release reaches the maintainers.
- Acceptance criteria:
  - The app starts on the target Linux distribution, see open question 6.
  - The app opens, edits, and saves a project file on local storage.
- Dependent on: none
- Requirements: GEDT-AR-00003, GEDT-AR-00004

### Web App

#### GEDT-STORY-00020 - As a maintainer, I want a local web app so that I edit SADs in a browser

- Description: Later release of the editor as a web app for one user on the local machine.
- VoC: Implement as web-app.
- Priority: Value adding
- Root reason: A browser UI works without installing a desktop toolkit.
- What if not done: Only users of the native Linux app can edit SADs.
- Acceptance criteria:
  - The web app offers the same edit and view functions as the native app.
  - The web app works in a current Firefox and a current Chromium release.
- Dependent on: GEDT-STORY-00019, GEDT-STORY-00023, GEDT-STORY-00024
- Requirements: GEDT-AR-00003, GEDT-AR-00005, GEDT-FR-00041

## Security and Quality

### File Handling

#### GEDT-STORY-00021 - As a maintainer, I want project files from others to open safely so that a crafted file cannot harm my machine

- Description: Treat every project XML file as untrusted input.
- VoC: Security review. Project files come from other people and tools, for example `aws2xml`.
- Priority: Critical
- Root reason: XML parsers can read local files or reach the network through external entities.
- What if not done: Opening a crafted file can leak local files or trigger network requests.
- Acceptance criteria:
  - Opening a file with an external entity does not read the referenced file.
  - Opening a file does not make a network request.
  - Opening a file with deeply nested or expanding entities fails with an error,
    and the editor stays responsive.
- Dependent on: none
- Requirements: GEDT-AR-00010, GEDT-AR-00011, GEDT-AR-00012, GEDT-AR-00013, GEDT-AR-00014,
  GEDT-AR-00015, GEDT-AR-00016, GEDT-AR-00029, GEDT-AR-00031

#### GEDT-STORY-00022 - As a maintainer, I want saves that never corrupt the project file so that an interrupted save does not lose my SAD

- Description: A save either completes or leaves the previous file intact.
- VoC: Automatic save every x minutes.
- Priority: Critical
- Root reason: Autosave writes often, which raises the chance of an interrupted write.
- What if not done: A crash during save can destroy the whole project file.
- Acceptance criteria:
  - Killing the editor during a save leaves either the previous or the new file,
    and the file validates against the SAD DTD.
  - Saving keeps the file permissions of the original file.
- Dependent on: none
- Requirements: GEDT-AR-00007, GEDT-AR-00008, GEDT-AR-00009, GEDT-FR-00006

#### GEDT-STORY-00025 - As a maintainer, I want project data to stay on my machine so that infrastructure details do not leak

- Description: The editor sends no project data off the local machine.
- VoC: Security review. Project files hold details such as AWS account IDs.
- Priority: Critical
- Root reason: Architecture data helps an attacker map the infrastructure.
- What if not done: Sensitive infrastructure details can leave the machine without the user knowing.
- Acceptance criteria:
  - During open, edit, and save, the editor makes no outbound network connection.
- Dependent on: none
- Requirements: GEDT-AR-00009, GEDT-AR-00015, GEDT-AR-00023, GEDT-AR-00024

### Web App Protection

#### GEDT-STORY-00023 - As a maintainer, I want the web app reachable only from my machine so that others cannot read or change my projects

- Description: The local web app serves only the local user.
- VoC: Decision 2026-11-22: web app is local only, without authentication.
- Priority: Critical, required by GEDT-STORY-00020
- Root reason: The web app has no authentication, so network exposure gives anyone full access.
- What if not done: Other hosts, or malicious web pages in the user's browser, can read or change projects.
- Acceptance criteria:
  - A request from another host fails to connect.
  - A request with a Host header other than the local address is rejected.
  - A state-changing request from another web origin is rejected.
- Dependent on: none
- Requirements: GEDT-AR-00017, GEDT-AR-00018, GEDT-AR-00019, GEDT-AR-00020

#### GEDT-STORY-00024 - As a maintainer, I want project text shown as plain text in the web app so that a crafted file cannot run code in my browser

- Description: Text from project files never runs as script or markup in the browser.
- VoC: Security review.
- Priority: Critical, required by GEDT-STORY-00020
- Root reason: Project files are untrusted input, see GEDT-STORY-00021.
- What if not done: A crafted project file can run script with full access to the web app.
- Acceptance criteria:
  - A component name containing `<script>` text shows as literal text, and no script runs.
- Dependent on: none
- Requirements: GEDT-AR-00021

## Out of Scope

- Authentication, roles, and audit logging, until a multi-user service is in scope.
- Company-wide multi-user service. VoC: maybe much later, as a microservice solution.
- Native app on platforms other than Linux.
- Storage formats other than the SAD XML.

## Open Questions

- 1: Measurable outcome.
  The charter objectives table is empty.
  Proposed default: zero manual XML edits to add a component, and every saved file validates.
- 2: External file changes.
  Hand edits or `aws2xml` can change a file while the editor has the file open.
  Options: warn and ask before overwrite, reload, or ignore.
  Proposed default: detect the change and ask before overwriting.
- 3: Autosave interval.
  Options: fixed 1 minute, or user-configurable.
  Proposed default: fixed 1 minute in the first release.
- 4: Replica grouping.
  Replicas are separate components, see GEDT-STORY-00009.
  Options: no grouping, or an optional visual group such as an ETCD cluster.
  Proposed default: no grouping in the first release.
- 5: Client-server filter criteria.
  The VoC names only API.
  Proposed default: filter by component type.
- 6: Target Linux distribution and desktop.
  Proposed default: Fedora, current release, on Wayland and X11.
- 7: Gemba gaps.
  The assessment has empty gemba entries for adding a viewpacket and a deployment diagram.
  Proposed default: use the steps in the repository `README.md` as the current process.
- 8: Common DTD.
  The repository has `cpp_cli/ideatoproduct.dtd` and
  `sad_xml_sql/tests/software_architecture_documentation.dtd`.
  Proposed default: `software_architecture_documentation.dtd`, as referenced by `template_sad.xml`.
