# GUI Editor Requirements

## Introduction

### Purpose

Technical requirements for `gui_editor`, derived from the story cards in
[USER_STORIES.md](USER_STORIES.md).
Readers are the planner, the developers, and the testers of `gui_editor`.

### Scope

- In scope: native Linux app, later local web app, editing and viewing of SAD XML projects.
- Out of scope: multi-user service, authentication of users, storage formats other than SAD XML.
- Included files: the editor loads Component and Team elements and writes back Component elements only,
  see GEDT-FR-00042 to GEDT-FR-00045.
  The product owner states this scope may change in the future.
- Adopted defaults for the business-analyst open questions:
  - 2: Detect external file changes and ask before overwriting, see GEDT-FR-00009.
  - 3: Fixed autosave interval of 60 seconds, see GEDT-FR-00007.
  - 4: No replica grouping in the first release, see GEDT-FR-00023.
  - 6: Fedora, current release, on Wayland and X11, see GEDT-AR-00004.
  - 8: `software_architecture_documentation.dtd` is the SAD DTD, see GEDT-AR-00001.

### Vocabulary

- Address: Element ID in the form `a.b.c.d`, each part a decimal number from 0 to 255.
- Core: Project model, XML reader, XML writer, and in-memory SQLite database.
- DTD rules: Element order, required elements, and required attributes of the SAD DTD,
  plus the field rules of GEDT-AR-00013.
- FileId: Number in `<Document><FileId>` that identifies a project.
- Included file: SAD XML file named by an `<Include>` element of the including project file.
- SAD: Software Architecture Documentation.
- SAD DTD: `sad_xml_sql/tests/software_architecture_documentation.dtd`, version 0.1.0.
- Session: One run of the editor, with one or more loaded projects.
- Skipped edit: Edit to a Component or Team element loaded from an included file,
  which no save writes to that file.
- SRS: Software Requirements Specification.
- STS: Software Testing Specification.

### References

- Story cards: [USER_STORIES.md](USER_STORIES.md)
- Opportunity assessment: [02_opportunity_assesmemt.md](02_opportunity_assesmemt.md)
- SAD DTD: [software_architecture_documentation.dtd](../../sad_xml_sql/tests/software_architecture_documentation.dtd)
- SAD concepts: [SoftwareArchitectureDocumentationOverview.md](../../docs/SoftwareArchitectureDocumentationOverview.md)
- Shared model library: [sad_xml_sql](../../sad_xml_sql/src/lib.rs)
- Requirement format: [requirements-writing skill](../../.claude/skills/requirements-writing/SKILL.md)

### Overview

- [Traceability matrix](#traceability-matrix): story to requirement mapping.
- [Architecture and interface overview](#architecture-and-interface-overview): components and context.
- [Requirements](#requirements): functional and architectural requirements.
- [Interfaces](#interfaces): protocols, formats, and error handling.
- [Design decisions](#design-decisions): decisions with rejected alternatives.
- [Risks](#risks): technical risks and mitigations.
- [Open questions](#open-questions): questions for the product owner.

## Traceability matrix

| Story | Requirements |
| --- | --- |
| GEDT-STORY-00001 | GEDT-AR-00001, GEDT-AR-00012, GEDT-AR-00013, GEDT-AR-00027, GEDT-FR-00001, GEDT-FR-00002, GEDT-FR-00003, GEDT-FR-00042 |
| GEDT-STORY-00002 | GEDT-AR-00001, GEDT-AR-00002, GEDT-AR-00030, GEDT-AR-00035, GEDT-FR-00004, GEDT-FR-00005, GEDT-FR-00006, GEDT-FR-00009, GEDT-FR-00043, GEDT-FR-00044, GEDT-FR-00045, GEDT-FR-00046, GEDT-FR-00047, GEDT-FR-00048 |
| GEDT-STORY-00003 | GEDT-AR-00007, GEDT-AR-00029, GEDT-FR-00007, GEDT-FR-00008, GEDT-FR-00009, GEDT-FR-00047, GEDT-FR-00048 |
| GEDT-STORY-00004 | GEDT-FR-00010, GEDT-FR-00011, GEDT-FR-00012, GEDT-FR-00013 |
| GEDT-STORY-00005 | GEDT-FR-00013, GEDT-FR-00014, GEDT-FR-00015, GEDT-FR-00016, GEDT-FR-00045 |
| GEDT-STORY-00006 | GEDT-FR-00011, GEDT-FR-00017, GEDT-FR-00018, GEDT-FR-00019 |
| GEDT-STORY-00007 | GEDT-FR-00011, GEDT-FR-00020, GEDT-FR-00021, GEDT-FR-00022 |
| GEDT-STORY-00008 | GEDT-AR-00016, GEDT-AR-00028, GEDT-FR-00024 |
| GEDT-STORY-00009 | GEDT-FR-00023 |
| GEDT-STORY-00010 | GEDT-FR-00025, GEDT-FR-00026 |
| GEDT-STORY-00011 | GEDT-FR-00026, GEDT-FR-00027, GEDT-FR-00028 |
| GEDT-STORY-00012 | GEDT-FR-00029 |
| GEDT-STORY-00013 | GEDT-FR-00030 |
| GEDT-STORY-00014 | GEDT-FR-00026, GEDT-FR-00031 |
| GEDT-STORY-00015 | GEDT-FR-00032, GEDT-FR-00033 |
| GEDT-STORY-00016 | GEDT-FR-00034, GEDT-FR-00035 |
| GEDT-STORY-00017 | GEDT-FR-00036, GEDT-FR-00037, GEDT-FR-00038 |
| GEDT-STORY-00018 | GEDT-FR-00039, GEDT-FR-00040 |
| GEDT-STORY-00019 | GEDT-AR-00003, GEDT-AR-00004 |
| GEDT-STORY-00020 | GEDT-AR-00003, GEDT-AR-00005, GEDT-FR-00041 |
| GEDT-STORY-00021 | GEDT-AR-00010, GEDT-AR-00011, GEDT-AR-00012, GEDT-AR-00013, GEDT-AR-00014, GEDT-AR-00015, GEDT-AR-00016, GEDT-AR-00029, GEDT-AR-00031, GEDT-AR-00035 |
| GEDT-STORY-00022 | GEDT-AR-00007, GEDT-AR-00008, GEDT-AR-00009, GEDT-FR-00006 |
| GEDT-STORY-00023 | GEDT-AR-00017, GEDT-AR-00018, GEDT-AR-00019, GEDT-AR-00020 |
| GEDT-STORY-00024 | GEDT-AR-00021 |
| GEDT-STORY-00025 | GEDT-AR-00009, GEDT-AR-00015, GEDT-AR-00023, GEDT-AR-00024 |

Traceability gaps:

- Secondary: GEDT-AR-00025, supply-chain hygiene.
  No story asks for GEDT-AR-00025; the repository security policy mandates the requirement.
- Secondary: GEDT-AR-00032, program code in Rust.
  No story asks for GEDT-AR-00032; the product owner mandates the requirement as a process parameter.
- Secondary: GEDT-AR-00033, scripts in Bash.
  No story asks for GEDT-AR-00033; the product owner mandates the requirement as a process parameter.
- Secondary: GEDT-AR-00034, official Rust build image.
  No story asks for GEDT-AR-00034; the product owner mandates the requirement as a process parameter.
- Orphaned requirements: none.
- Stories without a requirement: none.
- Removed: GEDT-AR-00006 and GEDT-AR-00022, together with the web app architecture decision.
  The IDs stay retired.

## Architecture and interface overview

The native app runs the egui UI and the core in one process.
The core reuses and extends the `sad_xml_sql` library, shared with `sad2md` and `tui_editor`.
The web app architecture is not decided; the web app is outside the first release.

```mermaid
flowchart LR
  user[Maintainer or reader]
  tools[sad2md, tui_editor, aws2xml, git]
  files[(SAD XML project files)]
  subgraph gui_editor
    ui[egui UI]
    core[Core: sad_xml_sql, in-memory SQLite]
  end
  user --> ui
  ui --> core
  core -->|read, atomic write| files
  tools -->|read, write| files
```

| Component | Status | Responsibility |
| --- | --- | --- |
| Core | Changed, `sad_xml_sql` | XML read and write, DTD rule check, in-memory SQLite model, ID allocation. |
| egui UI | New | Edit mode, view mode, panes, windows, diagrams, search, filters. |
| SAD XML project files | Existing | Source of truth, one file per project. |

## Requirements

Priority follows the highest priority among the traced stories.
Requirements traced only to GEDT-STORY-00023 or GEDT-STORY-00024 are Critical for the web release.

### Storage

| ID | Requirement | Traces to | Priority | Verification |
| --- | --- | --- | --- | --- |
| GEDT-AR-00001 | The core shall use SAD XML files that follow the SAD DTD as the only persistent project store. | 00001, 00002 | Critical | Inspection |
| GEDT-AR-00002 | `sad2md` and `sad_xml_sql` shall process every file saved by the editor without error. | 00002 | Critical | Test |
| GEDT-AR-00007 | When a save is interrupted at any point, the project file shall contain either the complete previous content or the complete new content. | 00003, 00022 | Critical | Test, kill during save |
| GEDT-AR-00008 | A save shall keep the permission mode bits of the existing project file. | 00022 | Critical | Test |
| GEDT-AR-00009 | The editor shall write project data only to project files and to temporary files in the same directory, with permission mode bits no wider than the project file. | 00022, 00025 | Critical | Test, inspection |
| GEDT-AR-00030 | The core shall extend the shared `sad_xml_sql` library, keeping the `sad2md` and `tui_editor` test suites passing. | 00002 | Critical | Test |
| GEDT-AR-00035 | The editor shall write an included file only when the resolved path, after symlink resolution at write time, is inside the directory of the including project file. | 00002, 00021 | Critical | Test, traversal and symlink swap fixtures |
| GEDT-FR-00001 | When the user selects a SAD XML file, the editor shall load every Component, ViewPacket, ComponentRelation, Behavior, Team, Requirement, and Include element into the session. | 00001 | Critical | Test |
| GEDT-FR-00002 | When a file fails XML parsing or the DTD rules, the editor shall reject the whole file with an error naming the file path, line, column, and failed rule. | 00001 | Critical | Test |
| GEDT-FR-00003 | The editor shall open a project file without writing to the file. | 00001 | Critical | Test, compare hash and mtime |
| GEDT-FR-00004 | On a save request, the editor shall write each modified project to the file the project was loaded from. | 00002 | Critical | Test |
| GEDT-FR-00005 | A save without edits shall preserve every element, attribute, and text value that the SAD DTD defines in the loaded file. | 00002 | Critical | Test, canonical XML compare |
| GEDT-FR-00006 | Before writing, the editor shall check the project against the DTD rules and, on failure, keep the existing file and show the failing element ID. | 00002, 00022 | Critical | Test |
| GEDT-FR-00007 | With unsaved changes, the editor shall save the project within 60 seconds of the first unsaved change. | 00003 | Value adding | Test |
| GEDT-FR-00008 | Without unsaved changes, the editor shall not write the project file. | 00003 | Value adding | Test |
| GEDT-FR-00009 | Before each save, including autosave, the editor shall ask the user before overwriting a file that changed on disk since the last load or save. | 00002, 00003 | Critical | Test |
| GEDT-FR-00042 | From an included file, the core shall load only the Component and Team elements into the editor session, keeping the default read of the shared `sad_xml_sql` reader unchanged. | 00001 | Critical | Test, `sad2md` golden tests |
| GEDT-FR-00043 | When a save writes an included file, the editor shall write to that file only the Component elements loaded from that file. | 00002 | Critical | Test |
| GEDT-FR-00044 | When a save writes an included file, the editor shall keep the Document, Include, Team, ViewPacket, ComponentRelation, Behavior, and Requirement elements of that file unchanged. | 00002 | Critical | Test, canonical XML compare |
| GEDT-FR-00045 | When the user selects Save, directly or through exit, while the session holds a skipped edit, the editor shall show a warning that names the address and the source file of each element with a skipped edit. | 00002, 00005 | Critical | Test: direct Save shows a non-modal warning; exit shows a modal warning that lets the user cancel the exit; autosave shows no warning |
| GEDT-FR-00046 | The core shall return every save failure as an error value that names the project file and the failure cause. | 00002 | Critical | Test, read-only directory and full disk fixtures |
| GEDT-FR-00047 | When a save fails, the editor shall show the user the error of GEDT-FR-00046. | 00002, 00003 | Critical | Test |
| GEDT-FR-00048 | After a failed save, the editor shall keep the project marked as modified. | 00002, 00003 | Critical | Test |

TOOD fix this AI generated shit show. Update the requirements skill to adhere to a correct req doc formatting, including where to put requirement notes.

Included files:

- Source: product owner decision, quoted: "only components and teams are read from an include file and only those are written back.
  This may change in the future."
- The product owner answer to open question 9 corrects the write-back part, quoted: "GEDT-FR-00043 is wrong, it should state: When a save writes an included file, the editor shall write to that file only the Component loaded from that file."
  GEDT-FR-00043 reads "Component" as "Component elements".
  The read part stays: GEDT-FR-00042 loads Component and Team elements.
- No save writes Team elements to an included file.
  GEDT-FR-00044 keeps the Team elements of an included file unchanged, including Team `Member`.
- Stability of GEDT-FR-00042 to GEDT-FR-00045: Low.
  The product owner states the scope may change.
- GEDT-FR-00001 applies to the project file the user selects.
  GEDT-FR-00042 limits the editor session to Component and Team elements of an included file.
- GEDT-FR-00042 follows the product owner answer to open question 5.
  The shared `sad_xml_sql` reader gets an option that loads only Component and Team elements from an included file.
  The option skips the viewpackets, relations, behaviors, and nested Include elements of an included file.
  Only the editor sets the option.
  The default read stays unchanged, so `sad2md` output and its golden tests stay unchanged, GEDT-AR-00030.
- GEDT-FR-00043 depends on GEDT-AR-00014 and GEDT-AR-00035.
  `PLAN.md` task T-07 adds the include path check; a later save task writes included files.
  Until then, a save writes no included file.
- GEDT-FR-00044 follows the product owner answer to open question 6.
  The SAD DTD requires `ViewPacket+` in every file, so a file with only Component elements fails the DTD rules.
- GEDT-FR-00044 keeps Team elements as the included file defines them on disk, because the `sad_xml_sql` model does not hold Team `Member`.
- Architect default, pending product owner review in open question 13: GEDT-FR-00009 applies to each included file before a save writes that file.
  The prompt names the included file.
  When the user declines, the save leaves that included file as the file is on disk.
  When the user confirms, GEDT-FR-00044 keeps the other elements as the included file holds them on disk at write time.
  The save therefore overwrites only the Component elements of the included file.
- GEDT-FR-00045 follows the product owner answers to open questions 7 and 11.
  The editor allows edits to Component and Team elements of included files.
  Until a save writes included files, every save skips both.
  After that, every save still skips the Team edits, GEDT-FR-00043.
- GEDT-FR-00045 covers Team edits of included files, so no edit is lost silently.
- Autosave, GEDT-FR-00007, skips the skipped edits without a warning.
  The user gets the warning on the next explicit Save or exit.
- Product owner decision: a skipped edit does not count as an unsaved change for GEDT-FR-00007 and GEDT-FR-00008.
  Autosave therefore does not rewrite an unchanged file every 60 seconds.
- Product owner decision: GEDT-FR-00045 shows the warning on an explicit Save or exit, also when GEDT-FR-00008 leaves every file unwritten.
- Product owner answer to open question 12: the warning of GEDT-FR-00045 is non-modal on a direct Save.
  On exit, the warning is modal and lets the user cancel the exit, because a non-modal warning closes with the window.
- Skipped edits stay in the session until the session ends, so each explicit Save or exit repeats the warning.
- A crash before the next explicit Save or exit loses the skipped edits without a warning, like every other unsaved change.
- GEDT-AR-00035 repeats the GEDT-AR-00014 check at write time, because a symlink can change between load and save.

Save failures:

- GEDT-FR-00006 covers a DTD rule failure before the write;
  GEDT-FR-00046 to GEDT-FR-00048 cover every other save failure,
  for example a failed open, write, flush, or rename.
- GEDT-FR-00046 applies to `tui_editor` through the shared core, GEDT-AR-00030.
- GEDT-FR-00048 keeps the next autosave attempt,
  because GEDT-FR-00008 writes only with unsaved changes.
- Source: tester finding in `tui_editor/src/work.rs` line 314.
  Product owner decision: a new task under GEDT-STORY-00002 fixes the silent save failure.
- Product owner decision, open question 10: `tui_editor` keeps the save target `tui_out.xml`.
  `PLAN.md` task T-48, which changes the `tui_editor` save target to the loaded project file, is deferred and unscheduled.
  T-48 is a `tui_editor` task outside the `gui_editor` scope, with no GEDT story or requirement.

GEDT-FR-00005 note:

- The product owner accepts the losses in [Known save limitations](#known-save-limitations) for the first release.
- GEDT-FR-00005 stays the target; the accepted losses are known exceptions,
  and the round-trip tests exclude only those items.

#### Known save limitations

The `sad_xml_sql` reader reads none of the elements below from any file,
so a save of the including project file drops them.
Included files are not written yet, so included files lose nothing today.
The product owner accepts these losses as a known limitation,
in the answer to open question 8.

| Dropped item | Conflict with GEDT-STORY-00002 criterion | Conflict with requirements |
| --- | --- | --- |
| Team `Member` | None; the criterion names components, viewpackets, and relations only. | GEDT-FR-00001, GEDT-FR-00005; the including project file only, because GEDT-FR-00044 keeps Team elements of an included file unchanged. |
| Top-level `Requirement` | None; the criterion names components, viewpackets, and relations only. | GEDT-FR-00001, GEDT-FR-00005. |
| ViewPacket `ChildViewPacketId`, `ParentViewPacketId`, `SiblingViewPacketId` | Yes: each viewpacket stays, but the viewpacket loses the links to other viewpackets. | GEDT-FR-00005. |
| ViewPacket `DiagramKey` | Yes: each viewpacket and each ComponentRelation stays, but a relation whose `Key` matches only a dropped `DiagramKey` no longer belongs to any viewpacket diagram. | GEDT-FR-00005. |
| XML comments, in any file | None. | None; the SAD DTD defines no comments, so GEDT-FR-00005 does not cover comments. |

Behavior `DiagramKey` is not a loss: `PLAN.md` task T-40, reviewed and approved, writes Behavior `DiagramKey` back exactly as in the source.
The accepted loss is ViewPacket `DiagramKey` only, which the `sad_xml_sql` reader never reads.

Consequences of the accepted losses:

- The GEDT-STORY-00002 criterion for a save without edits is knowingly not fully met.
  The criterion reads: "Opening, then saving without edits, keeps every component, viewpacket, and relation."
  The ViewPacket `DiagramKey` and viewpacket link losses cause the gap.
  The business analyst aligns the story criterion when the product owner asks.
- GEDT-FR-00001 and GEDT-FR-00005 keep the full target; the dropped items are accepted exceptions.
- Team `Member` in included files is not lost, because no save writes Team elements to an included file,
  product owner answer to open question 9.

### Components

| ID | Requirement | Traces to | Priority | Verification |
| --- | --- | --- | --- | --- |
| GEDT-FR-00010 | In edit mode, the editor shall create a component from a form with Name and Summary. | 00004 | Critical | Demonstration |
| GEDT-FR-00011 | The editor shall assign each new Component, ViewPacket, and ComponentRelation an address unused by any element of the same type in the loaded projects, with the project FileId as first part. | 00004, 00006, 00007 | Critical | Test |
| GEDT-FR-00012 | The editor shall write Component elements in ascending case-insensitive order of Name, with ties ordered by address. | 00004 | Critical | Test |
| GEDT-FR-00013 | The editor shall reject a create or rename that gives a component a Name already used in the same project, with an error naming the existing component. | 00004, 00005 | Critical | Test |
| GEDT-FR-00014 | In edit mode, the editor shall let the user change the Name attribute and every child element of a component defined by the SAD DTD. | 00005 | Critical | Demonstration |
| GEDT-FR-00015 | The editor shall keep the component address unchanged when the user edits the component. | 00005 | Critical | Test |
| GEDT-FR-00016 | After a component rename, every view shall show the new Name within the same session. | 00005 | Critical | Test |

### Viewpackets

| ID | Requirement | Traces to | Priority | Verification |
| --- | --- | --- | --- | --- |
| GEDT-FR-00017 | The editor shall create a viewpacket only with a primary ComponentId that references a component in the loaded projects. | 00006 | Critical | Test |
| GEDT-FR-00018 | The editor shall offer only the ViewType and ViewStyle values listed by `sad_xml_sql::create_styles`. | 00006 | Critical | Test |
| GEDT-FR-00019 | In edit mode, the editor shall let the user change every child element and attribute of a viewpacket defined by the SAD DTD, except the address. | 00006 | Critical | Demonstration |

Rationale for GEDT-FR-00018: the SAD DTD declares ViewType and ViewStyle as CDATA,
so the allowed set comes from `sad_xml_sql`, the list that `sad2md` processes.

### Diagrams

| ID | Requirement | Traces to | Priority | Verification |
| --- | --- | --- | --- | --- |
| GEDT-FR-00020 | In edit mode, the editor shall add and remove ComponentRelation elements in the diagram of a viewpacket with style ClientServer, Decomposition, Deployment, Install, or Uses, and in context diagrams. | 00007 | Critical | Demonstration |
| GEDT-FR-00021 | The editor shall accept a relation endpoint only when the endpoint references a component in the loaded projects. | 00007 | Critical | Test |
| GEDT-FR-00022 | Removing a component from a diagram shall remove the relations of that diagram only, keeping the Component element. | 00007 | Critical | Test |
| GEDT-FR-00023 | The diagram editor shall draw each component address as a separate node with its own relations. | 00009 | Value adding | Test, ETCD1 to ETCD3 fixture |
| GEDT-FR-00024 | While adding components to a diagram, the editor shall list only components whose Name contains the search text, ignoring letter case and matching every character literally. | 00008 | Value adding | Test, includes `'`, `%`, `_` |
| GEDT-FR-00025 | With a filter text applied to a ClientServer primary diagram, the editor shall show only components whose Name contains the filter text, plus their direct relations. | 00010 | Value adding | Test |

### Viewing

| ID | Requirement | Traces to | Priority | Verification |
| --- | --- | --- | --- | --- |
| GEDT-FR-00026 | Filters, selections, pane layout, and mode changes shall leave the project data and its modified flag unchanged. | 00010, 00011, 00014 | Value adding | Test |
| GEDT-FR-00027 | In view mode, the editor shall reject every request that changes project data. | 00011 | Value adding | Test, core level |
| GEDT-FR-00028 | Switching between view mode and edit mode shall keep the current selection. | 00011 | Value adding | Test |
| GEDT-FR-00029 | On component selection, the editor shall list every viewpacket with that component as primary ComponentId, and no other viewpacket. | 00012 | Value adding | Test |
| GEDT-FR-00030 | On component selection, the editor shall list, apart from the primary list, every viewpacket whose ComponentId list or diagram relations reference the component as non-primary. | 00013 | Nice to have | Test |
| GEDT-FR-00031 | The editor shall show at least two panes side by side in one window, each pane with its own view and selection. | 00014 | Value adding | Demonstration |
| GEDT-FR-00032 | The editor shall open at least two windows on the same session. | 00015 | Value adding | Demonstration |
| GEDT-FR-00033 | A component selection in one window shall show the component in every other window within 1 second. | 00015 | Value adding | Test |

### Multiple projects

| ID | Requirement | Traces to | Priority | Verification |
| --- | --- | --- | --- | --- |
| GEDT-FR-00034 | The editor shall load at least two project files in one session. | 00016 | Value adding | Test |
| GEDT-FR-00035 | A save shall write the data of each project only to the file of that project. | 00016 | Value adding | Test |
| GEDT-FR-00036 | The editor shall let a viewpacket or relation in one project reference a component of another loaded project. | 00017 | Value adding | Test |
| GEDT-FR-00037 | The editor shall save a cross-project reference as the component address, whose first part is the FileId of the owning project. | 00017 | Value adding | Test |
| GEDT-FR-00038 | The editor shall mark a reference to a project that is not loaded as unresolved, and shall keep the reference on save. | 00017 | Value adding | Test |
| GEDT-FR-00039 | The editor shall show components of at least two loaded projects in one view. | 00018 | Value adding | Demonstration |
| GEDT-FR-00040 | Each component in a multi-project view shall show the Title of the owning project. | 00018 | Value adding | Demonstration |

### Platforms

| ID | Requirement | Traces to | Priority | Verification |
| --- | --- | --- | --- | --- |
| GEDT-AR-00003 | The editor shall use the egui toolkit. | 00019, 00020 | Critical | Inspection |
| GEDT-AR-00004 | The native app shall open, edit, and save a project on the current Fedora release, on Wayland and on X11. | 00019 | Critical | Demonstration |
| GEDT-AR-00005 | The web app shall run in the current Firefox release and the current Chromium release. | 00020 | Value adding | Demonstration |
| GEDT-FR-00041 | The web app shall offer every edit and view function of the native app. | 00020 | Value adding | Test, shared UI suite |
| GEDT-AR-00032 | Hand-written program code of `gui_editor` shall be written in Rust. | Secondary | Critical | Inspection of repository |
| GEDT-AR-00033 | When `gui_editor` needs a hand-written script, the script shall be written in Bash. | Secondary | Critical | Inspection of repository |
| GEDT-AR-00034 | The build shall compile `gui_editor` in the Docker Official Image `docker.io/library/rust`. | Secondary | Critical | Inspection of build |

Rationale for GEDT-AR-00032 and GEDT-AR-00033: the product owner limits the languages to reduce toolchains, reviewer skills, and supply-chain surface.
Data and configuration formats are not program code or scripts, for example TOML, YAML, XML, Markdown, and Mermaid.
Code generated by the Rust toolchain, for example `wasm-bindgen` JavaScript glue, is not hand-written.

The web app architecture is decided with the product owner when the web release starts.

### Security

| ID | Requirement | Traces to | Priority | Verification |
| --- | --- | --- | --- | --- |
| GEDT-AR-00010 | The XML reader shall resolve no external entity and load no external DTD subset. | 00021 | Critical | Test, XXE fixtures |
| GEDT-AR-00011 | The XML reader shall reject a file that declares an entity, with an error. | 00021 | Critical | Test, billion laughs fixture |
| GEDT-AR-00012 | The core shall report every malformed-input condition as an error value, without panic or process exit. | 00001, 00021 | Critical | Test, fuzzing |
| GEDT-AR-00013 | The core shall reject a file with an address outside `a.b.c.d` with parts 0 to 255, a non-integer SortOrder, or a FileId outside 1 to 255. | 00001, 00021 | Critical | Test |
| GEDT-AR-00014 | The core shall load an Include file only when the resolved path, after symlink resolution, is inside the directory of the including project file. | 00021 | Critical | Test, traversal fixtures |
| GEDT-AR-00015 | The editor shall open no outbound network connection, and shall never fetch an Include url attribute. | 00021, 00025 | Critical | Test, network namespace without route |
| GEDT-AR-00016 | The core shall pass every value from files or user input to SQLite as a bound parameter. | 00008, 00021 | Critical | Inspection, static check |
| GEDT-AR-00017 | The web server shall listen only on 127.0.0.1 and ::1. | 00023 | Critical, web release | Test |
| GEDT-AR-00018 | The web server shall reject a request whose Host header is not `localhost`, `127.0.0.1`, or `[::1]` with the server port, with status 421. | 00023 | Critical, web release | Test, DNS rebinding case |
| GEDT-AR-00019 | The web server shall reject a state-changing request whose Origin header differs from the server origin, with status 403. | 00023 | Critical, web release | Test |
| GEDT-AR-00020 | The web server shall require, on every API request, a per-launch token of at least 128 bits from the OS random source, kept out of logs and request URLs. | 00023 | Critical, web release | Test, inspection |
| GEDT-AR-00021 | The web app shall render project text only as text, and every server response shall carry a Content-Security-Policy that allows scripts from the server origin only. | 00024 | Critical, web release | Test, `<script>` fixture |
| GEDT-AR-00023 | Diagnostic logs shall contain file paths, addresses, and error kinds only, without field text or tokens, and shall stay on the local machine. | 00025 | Critical | Inspection, test |
| GEDT-AR-00024 | The editor shall require and store no credentials, and the source shall contain no credentials. | 00025 | Critical | Inspection, secret scan |
| GEDT-AR-00025 | The build shall use the committed `Cargo.lock` with `--locked`, and shall fail on any unignored `cargo audit` or `cargo deny` finding. | Secondary | Critical | Inspection of build |
| GEDT-AR-00031 | The XML reader shall reject a file larger than 64 MiB, or with element nesting deeper than 32 levels, with an error. | 00021 | Critical | Test |

### Performance

| ID | Requirement | Traces to | Priority | Verification |
| --- | --- | --- | --- | --- |
| GEDT-AR-00027 | The editor shall load a project with 2,000 components, 500 viewpackets, and 5,000 relations within 2 seconds on the reference workstation. | 00001 | Critical | Test, benchmark |
| GEDT-AR-00028 | The component search shall update the result list within 100 ms of a keystroke for 2,000 components. | 00008 | Value adding | Test, benchmark |
| GEDT-AR-00029 | The editor shall run load and save outside the UI thread, keeping each UI frame under 100 ms. | 00003, 00021 | Critical | Test |

Reference workstation: 4 CPU cores, 8 GiB RAM, SSD, current Fedora release.
The project size targets are assumptions; see [open questions](#open-questions).

### Quality attributes

| Attribute | Measurable target | Verification |
| --- | --- | --- |
| Availability | Malformed input never terminates the process, GEDT-AR-00012. | Fuzzing of the XML reader, 1 hour without crash. |
| Compatibility | 100% of saved files pass `xmllint --noout --nonet --valid` and `sad2md`. | CI test on every fixture. |
| Maintainability | One shared core for `gui_editor`, `sad2md`, and `tui_editor`, GEDT-AR-00030. | CI runs all three test suites. |
| Observability | Every error names the file, and the line and column or the address. | Test of each error path. |
| Performance | Load within 2 seconds, search within 100 ms, UI frame under 100 ms. | Benchmarks in CI. |
| Portability | Native on Fedora Wayland and X11; web in Firefox and Chromium. | Release demonstration. |
| Reliability | Zero corrupted files after 1,000 kills during save. | Kill-loop test. |
| Security | Zero outbound connections; all security tests pass. | Network namespace test, security fixtures. |

## Interfaces

| Interface | Protocol | Format | Direction | Error handling | Versioning |
| --- | --- | --- | --- | --- | --- |
| Include reference | Local file read, atomic write after GEDT-AR-00035 | Relative path in `<Include>`; reads Component and Team elements, GEDT-FR-00042; writes Component elements only, GEDT-FR-00043 and GEDT-FR-00044 | Core reads; core writes in a later save task | Path outside project directory or missing file: error, project loads without the include; write check fails: error, included file kept. | SAD DTD version |
| SAD XML project file | Local file read, atomic write | XML 1.0, UTF-8, SAD DTD | Core reads and writes | Parse or rule failure: error with path, line, column; write failure: previous file kept, error shown, project stays modified, GEDT-FR-00046 to GEDT-FR-00048. | Root `Version` attribute, 0.1.0 |
| Selection events | In-process | Rust types, window and address | Core to all windows | A window that closed is dropped from the receivers. | Crate version |

Web app interfaces are defined with the web app architecture.

## Design decisions

### In-memory SQLite through sad_xml_sql

- Decision: Confirm the business-analyst assumption.
  The core holds loaded projects in in-memory SQLite, through the `sad_xml_sql` library.
- Rationale: `sad2md` and `tui_editor` already use the same schema; SQLite serves search and filters.
- Rejected: Plain Rust structs; duplicates the model and search of `sad_xml_sql`.
- Rejected: On-disk SQLite; creates a second source of truth and a copy of sensitive data.

### Hardened shared library

- Decision: Extend `sad_xml_sql` in place for error values, lossless round trip, and DTD order.
- Rationale: One parser to secure and test, used by every tool in the repository.
- Rejected: Fork the code into `gui_editor`; the copies drift, and the `sad2md` parser stays unsafe.

### Cross-project references by address

- Decision: A reference is the component address, with the owning project FileId as first part.
  FileIds stay unique across loaded projects.
- Rationale: The DTD and `sad_xml_sql` already use this address form; no DTD change is needed.
- Rejected: New project attribute on references; changes the DTD and breaks `sad2md`.
- Rejected: Name-based references; renames break the links.

### Atomic save

- Decision: Write a temporary file in the project directory, flush, rename over the project file,
  then flush the directory.
- Rationale: Rename is atomic on Linux file systems; the temporary file stays on the same device.
- Rejected: In-place write; a crash leaves a truncated file.
- Rejected: Backup copy then overwrite; leaves a corruption window and an extra copy of the data.

### DTD rule check in Rust

- Decision: The core checks DTD rules in Rust; CI also runs `xmllint --noout --nonet --valid`.
- Rationale: No C XML library at runtime; `xmllint` gives an independent check in tests.
- Rejected: libxml2 at runtime; adds a C dependency with a history of entity and memory bugs.

### Accepted save losses

- Decision: Accept the losses in [Known save limitations](#known-save-limitations) for the first release.
- Source: product owner answer to open question 8.
- Rationale: The fix needs reader, writer, and schema changes in `sad_xml_sql` for each dropped item.
- Rejected: Fix the DTD-defined items before the first release; delays the first release.
- Consequence: The GEDT-STORY-00002 save criterion is knowingly not fully met.

### Editor-only reader option for included files

- Decision: Add an option to the shared `sad_xml_sql` reader that loads only Component and Team elements from an included file, GEDT-FR-00042.
  The option skips the viewpackets, relations, behaviors, and nested Include elements of an included file.
  Only the editor sets the option.
- Source: product owner answer to open question 5.
- Rationale: The default read stays unchanged, so `sad2md` output and golden tests stay unchanged.
- Rejected: Restrict the default shared reader to Component and Team elements of included files; removes the external viewpacket links in `sad2md` output.
- Cost: Two read paths to test.

### Component-only write-back for included files

- Decision: A save writes only Component elements to an included file;
  Team elements stay unchanged.
- Source: product owner answer to open question 9.
- Rationale: The `sad_xml_sql` model drops Team `Member`,
  so a Team write-back loses data the user did not edit.
- Rejected: Write Team elements back without `Member`; extends the accepted loss to included files.
- Rejected: Read and write Team `Member` first; needs reader, writer, and schema changes.
- Cost: Team edits of included files are never saved; GEDT-FR-00045 warns the user.

### Encryption

- Decision: The editor adds no encryption; data at rest relies on OS disk encryption.
- Rationale: Data stays on the local machine, GEDT-AR-00009 and GEDT-AR-00015.
- Rejected: File encryption; breaks `sad2md`, git diffs, and the XML constraint.

## Risks

| Risk | Likelihood | Impact | Mitigation |
| --- | --- | --- | --- |
| Address parts that are not numbers silently become 0 in `convert_from_address_to_id`. | High | High | GEDT-AR-00013. |
| Autosave overwrites a file changed by `aws2xml` or a hand edit. | Medium | High | GEDT-FR-00009. |
| Changes to `sad_xml_sql` break `sad2md` or `tui_editor`. | Medium | Medium | GEDT-AR-00030; CI runs all suites. |
| Component reorder on first save produces large git diffs in existing files. | Medium | Low | One-time reorder commit before first use. |
| Dump in `sad_xml_sql` drops DTD-defined items on save. | Certain | High | T-40 restores Include elements, ViewPacket `TeamId`, and Behavior `DiagramKey`; T-03 lists the remaining gaps; GEDT-FR-00005 golden-file round-trip tests; the rest is in [Known save limitations](#known-save-limitations). |
| Dump in `sad_xml_sql` regresses to the root attribute `version="0.1.1"`, fixed by T-40; the DTD requires `Version`. | Low | High | GEDT-FR-00006; round-trip tests. |
| Dump in `sad_xml_sql` regresses to writing elements out of DTD order, fixed by T-40. | Low | High | GEDT-FR-00006; round-trip tests. |
| Edits to components of an included file are lost, because no save writes included files yet. | High | Medium | GEDT-FR-00045 warns on each explicit Save or exit; a later save task writes included files. |
| Edits to teams of an included file are lost, because no save writes Team elements to an included file. | High | Medium | GEDT-FR-00045 warns on each explicit Save or exit. |
| Include path traversal, marked TODO in `db_populate_from_xml.rs`. | High | High | GEDT-AR-00014 before first release. |
| Included file written with only Component elements fails the SAD DTD rule `ViewPacket+` and loses content. | Medium | High | GEDT-FR-00044. |
| Loader in `sad_xml_sql` panics or exits on malformed input, losing unsaved work. | Certain | High | GEDT-AR-00012. |
| Multiple egui windows misbehave on Wayland. | Medium | Medium | Prototype GEDT-FR-00032 early. |
| Restricting the shared reader to Component and Team elements removes the external viewpacket links that `sad2md` writes for included files. | Low | Medium | GEDT-FR-00042 with an editor-only reader option; GEDT-AR-00030. |
| Save in `tui_editor/src/work.rs` line 314 discards the `Result` of `dump_db_to_xml`, so a failed save is silent and the user loses edits. | Certain | High | GEDT-FR-00046 to GEDT-FR-00048; new task under GEDT-STORY-00002. |
| Save in `tui_editor/src/work.rs` line 314 writes to the fixed relative path `tui_out.xml` in the working directory, not to the loaded project file. | Certain | Medium | GEDT-FR-00004 for the editor; for `tui_editor`, the product owner keeps `tui_out.xml`, open question 10, so `PLAN.md` task T-48 is deferred and unscheduled. |
| Symlink swap between load and save redirects a write of an included file outside the project directory. | Low | High | GEDT-AR-00035. |
| Template `template_sad.xml` uses IDs such as `1101`, which the loader rejects. | High | Medium | Open question 1; update the template. |
| Web API reached by DNS rebinding or a cross-site page. | Medium | High | GEDT-AR-00018, GEDT-AR-00019, GEDT-AR-00020. |

## Backwards compatibility

- Files saved by the editor stay readable by `sad2md`, `sad_xml_sql`, and `tui_editor`.
- Existing files gain alphabetical Component order on first save, GEDT-FR-00012.
- The writer fix changes the root attribute from `version` to `Version`, as the DTD requires.
- Files that `sad_xml_sql` accepts today, but that break the DTD rules, fail to open in the editor.
  The error names the failing rule, so the maintainer can fix the file.
- Viewpackets, relations, behaviors, and requirements of an included file do not show in the editor,
  GEDT-FR-00042.
  `sad2md` keeps the viewpackets of included files,
  because the default read of the shared reader is unchanged.
- A save drops the items in [Known save limitations](#known-save-limitations);
  the product owner accepts the loss.

## Feature interaction

- `aws2xml` and hand edits can change a file during a session, see GEDT-FR-00009.
- `sad2md` shares the hardened parser, so `sad2md` gains the same input protection.
- Git provides history and audit, so the editor keeps no audit log.

## Requirements to be tested

- Fixture set: valid files, DTD-rule failures, XXE, entity expansion, deep nesting,
  path traversal, `<script>` text, special characters in names.
- Round trip: open and save each valid fixture, then compare canonical XML and run `xmllint`.
  The compare excludes only the items in [Known save limitations](#known-save-limitations).
- Save failure: save into a read-only directory and onto a full disk,
  then assert the error, the kept file, and the modified flag.
- Kill loop: kill the editor at random points during save, then validate the file.
- Network: run the editor in a network namespace without a route, and assert zero connection attempts.
- Web security: Host, Origin, token, and CSP tests against the local server.

## Legacy test case updates

- `sad_xml_sql` tests that expect a panic, for example on a malformed address,
  change to expect an error value.
- `db_dump_to_xml` tests add checks for the `Version` attribute and DTD element order.

## Open questions

None blocks the first release; the proposed defaults are in the requirements above.

- 1: Address scheme.
  `sad_xml_sql` assumes the first address part is the FileId; `template_sad.xml` uses IDs such as `1101`.
  Options: `a.b.c.d` with FileId first, or a new scheme with a DTD change.
  Proposed default: `a.b.c.d` with FileId first, and update `template_sad.xml`.
- 2: Client-server filter criteria, GEDT-STORY-00010.
  The business-analyst default is component type, but the SAD DTD has no component type.
  Options: Name substring, a new `Type` element in the DTD, or the component Title.
  Proposed default: Name substring; a DTD change also touches `sad2md`.
- 3: Diagram styles in GEDT-STORY-00007.
  The story names context and installation; the DTD values are ContextModelKey and Install.
  Proposed default: installation means Install; context means the ContextModelKey diagram.
- 4: Project size and reference workstation for GEDT-AR-00027 and GEDT-AR-00028.
  Proposed default: 2,000 components, 500 viewpackets, 5,000 relations, 4 cores, 8 GiB RAM.
- 5: Answered: editor-only reader option, see GEDT-FR-00042.
- 6: Answered: keep the other elements of an included file unchanged, see GEDT-FR-00044.
- 7: Answered: allow edits and warn on save, see GEDT-FR-00045.
- 8: Answered: accept the losses, see [Known save limitations](#known-save-limitations).
- 9: Answered: a save writes only Component elements to an included file, see GEDT-FR-00043.
- 10: Answered: `tui_editor` keeps `tui_out.xml`; task T-48 is deferred and unscheduled, see [Save failures](#storage).
- 11: Answered: warn when the user selects Save, directly or through exit, see GEDT-FR-00045.
- 12: Answered: a non-modal warning on direct Save; on exit, a modal warning that lets the user cancel the exit, see GEDT-FR-00045.
- 13: External change of an included file before a save writes that file, GEDT-FR-00009 and GEDT-FR-00044.
  Options: the GEDT-FR-00009 prompt covers each included file; the save skips a changed included file without a prompt; the save fails.
  Architect default: the GEDT-FR-00009 prompt covers each included file, and a confirmed save keeps the other elements as on disk at write time.
  The default needs product owner review before the save task that writes included files.
