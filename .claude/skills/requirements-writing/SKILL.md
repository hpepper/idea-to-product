---
name: requirements-writing
description: Write, classify, identify, and trace technical requirements. Use when deriving architectural or functional requirements from business requirements, assigning FEAT-TYPE-nnnnn identifiers and priorities, checking requirement quality, or verifying traceability.
---

# Requirements writing

## Vocabulary

- Capability: Feature or function the customer needs, stated solution independent.
- Condition: Measurable attribute of a capability that limits designer options.
- Constraint: Immovable boundary imposed on the solution by circumstance, force, or compulsion.
- Derived: Child requirement that decomposes its parent, usually because each child is allocated to a different entity.
- Emergent property: Requirement no single component can satisfy, for example call center throughput.
- Fit criterion: Acceptance criterion for a requirement.
- Functional requirement: Function the software must execute.
- Non-functional requirement: Constraint or quality goal, for example performance, maintainability, safety, reliability.
- Orphaned: Requirement whose parent was deleted.
  Resolve by deleting, reparenting, or reclassifying as Secondary.
- Primary: External requirement with contractual value or weight.
- Process parameter: Constraint on how the software is developed, for example a mandated language.
- Rationale: Reason a requirement matters and how it contributes to the product purpose.
- Secondary: Requirement with no parent that is not an external requirement.
  Originates from design constraints, considerations unknown to the layer above, or a missing Primary requirement.
  Carries no contractual value, so check that a business reason still exists.

## Capability, condition, and constraint

Write each requirement as a capability, qualified by measurable conditions and bounded by constraints.
Keep conditions as attributes of a capability, not as separate capabilities.

Example: "Move people between Los Angeles and New York at a cruising speed of 2500 km/hr, never exceeding 5300 km/hr."

- Capability: Move people between Los Angeles and New York.
- Condition: Cruising speed of 2500 km/hr.
- Constraint: Maximum speed of 5300 km/hr.

## Identifier format

`FEAT-TYPE-nnnnn`, for example `PLTF-AR-13020`.

- FEAT: Four uppercase letters naming the feature or product area.
  - PLTF: Platform, for requirements that span the whole product.
  - Reuse a FEAT code already used in the repository docs before creating a new one.
- TYPE:
  - AD: Architectural Design, a design decision at the AR level.
  - AR: Architectural Requirement, for example quality attributes, interfaces, and constraints.
  - FD: Functional Design, a design decision at the FR level.
  - FR: Functional Requirement.
- nnnnn: Five digit number, unique within FEAT and TYPE.

Change the ID when a requirement changes from one project version to the next.

## Priority

- Critical: The product fails or is not acceptable without the requirement.
- Value adding: Important, but a workaround exists or delivery can follow later.
- Nice to have: First candidate to drop when time or budget runs short.

A supporting requirement can be low priority to the user but essential to a Critical one.
Record that relationship so the supporting requirement is reconsidered if the Critical one is removed.

## Statement format

- `<WHO> shall <WHAT> <WHERE> <WHEN> <CONSTRAINTS>`
- `<CONSTRAINT>, <WHO> has the ability to <WHAT>`

| Field | Meaning |
| --- | --- |
| CONSTRAINT | Process point, validation. |
| WHAT | Predicate, action, result. |
| WHEN | Process point, speed. |
| WHERE | Process point, item location. |
| WHO | Everyone, group, subject. |

Verb meanings:

- Shall: Required.
  Use shall consistently; do not mix shall and must in one document.
- Should, may: Optional goal, not a requirement.
  Avoid in requirement statements.
- Will: Statement of fact or future intent.

## Quality rules

- Single: One requirement per ID.
  Split a statement that covers two needs, for example two languages or two platforms, into two IDs.
- Abstract: States what, not how.
- Clear: One interpretation, one function, one sentence, one verb.
  "And", "or", "also", "unless", and "except" signal combined requirements; never use "and/or".
- Complete: No TBD.
  Add a rationale when the reason is not obvious.
- Consistent: No contradiction with other requirements or the parent; same term for the same item.
- Feasible: When reachability is unknown, propose a prototype and record a Goal.
- Granular: A small number of related test cases verifies the requirement.
  Many diverse tests signal a bundle of requirements.
- Necessary: Driven by voice of customer or voice of business; omitting it makes a difference.
- Traceable: Unique ID and a trace link to the higher-level source.
- Verifiable: Provable by demonstration, test, analysis, or inspection.
  No conceivable test means the requirement is ambiguous or incomplete.

Style:

- Use the active voice and name the actor, for example "The Buyer shall".
- Specify the trigger condition, the action, and the observable result.
- Use a table for a set of related requirements; write N/A for combinations with no requirement.
- Give enough detail that several acceptable designs remain, but the customer intent is clear.

## Ownership

- The architect is the only writer of requirement statements and requirement IDs.
- Other agents send a requirement change request to the architect instead of editing requirements.
  - A change request names the trigger, the affected IDs, the proposed change, and the source.
  - A product owner answer used as source is quoted verbatim.

## Untagged constraints

A constraint without an ID escapes traceability, verification, and review.

- Every statement that limits the solution is a requirement with an ID, or an open question.
- Rationale, design decisions, risks, and open question answers explain requirements and never add a constraint.
  - Words such as "stay", "only", "never", "always", and "must" in those sections signal a hidden constraint.
- A product owner answer that adds a constraint produces a new or amended ID before any other document uses the constraint.
- Record a product owner answer exactly as chosen, with no added condition.
- A question option contains only the choice asked about.
  - Put any extra condition in a separate option or a separate question.
- A definition of done, a test, or a review finding cites requirement IDs and never introduces a constraint.
- A constraint found without an ID is a finding, not a gap to fill with an invented value.

## Ambiguous terms

| Ambiguous term | Replacement |
| --- | --- |
| acceptable, adequate | Define acceptability and how the system judges it. |
| as much as practicable | Mark TBD with a date to resolve. |
| at least, not more than, not to exceed | State minimum and maximum values. |
| between | State whether endpoints are included. |
| depends on | Describe the dependency: input, prerequisite, or service. |
| efficient | State resource usage, operation speed, or ease of use. |
| fast, rapid | State the minimum acceptable speed. |
| flexible | Describe how the system changes for which conditions. |
| improved, better, faster, superior | Quantify the improvement. |
| including, etc., such as | List all possibilities. |
| maximize, minimize, optimize | State acceptable maximum and minimum values. |
| normally, ideally | Also describe abnormal behavior. |
| optionally | State whose choice: system, user, or developer. |
| reasonable, when necessary, where appropriate | Explain how to judge. |
| robust | Define exception handling and response to unexpected conditions. |
| seamless, transparent, graceful | State observable product characteristics. |
| several | State how many, or the range bounds. |
| shouldn't | State as a positive requirement. |
| state-of-the-art | Define for this product. |
| sufficient | State how much. |
| support, enable | State the exact functions performed. |
| user-friendly, simple, easy | State the usability characteristics. |

## Non-functional requirement types

Check each type for applicability when deriving AR requirements:

- Accessibility, availability, compatibility with existing systems, conversion of older data.
- Documentation, environmental conditions, ergonomics, external interfaces.
- Growth capacity, input, installation, maintainability, output.
- Performance, policy and regulatory, quality provisions, reliability.
- Safety, security, standards and technical policies, technology.
- Testing, training, transportability.

## Attributes

- Criticality: Priority, see Priority.
- Difficulty: High, Medium, or Low technical complexity.
  Internal only.
- Feasibility: Joint customer and analyst assessment of technology, readiness for change, risk, and cost.
- Risk: Financial loss, environmental impact, safety and health, or standards and laws.
- Source: Who originated the requirement, so you know whom to consult.
- Stability: High, Medium, or Low probability of change.
- Trace: Link to the higher-level source.

## Traceability

Link each requirement to:

- Higher-level requirements.
- Project and version.
- Story cards.
- Test cases.
- Use cases.
- Voice of customer or voice of business.

Traceability supports implementation coverage, decomposition logic, verification coverage, and impact assessment.
Trace every function to its origin to prevent gold plating.

## Pitfalls

- Ambiguity and vague terms.
- Combined requirements.
- Implementation written instead of the need.
- Incorrect assumptions.
- Let-out clauses.
- Missing requirements, including overlooked user classes.
- Rambling and speculation.
- Specifying the impossible.
- Suggestions written as requirements.

## Checklist

- Organization and completeness:
  - Cross-references between requirements are correct.
  - Level of detail is consistent and adequate for design.
  - Each requirement has a priority.
  - All external hardware, software, and communication interfaces are defined.
  - Expected behavior is documented for every anticipated error condition.
  - Missing information is marked TBD.
- Correctness:
  - No requirement conflicts with or duplicates another.
  - Each requirement is verifiable by test, demonstration, review, or analysis.
  - Each requirement is in scope and implementable within known constraints.
  - Error messages are unique and meaningful.
- Quality attributes:
  - Performance, security, and safety objectives are specified and quantified.
  - Trade-offs between quality attributes are stated.
- Special issues:
  - Every requirement is a requirement, not a design or implementation solution.
  - Time-critical functions have timing criteria.
  - Internationalization is addressed.
- Traceability:
  - Each requirement has a unique, correct FEAT-TYPE-nnnnn ID.
  - Each requirement traces to a higher-level requirement or use case.
  - Secondary and Orphaned requirements are flagged.
  - No constraint appears outside a requirement with an ID.

## References

- Iee98: IEEE Std 1233-1998, Guide for Developing System Requirements Specifications.
- Rob06: Robertson, S. and Robertson, J., Mastering the Requirements Process, 2nd ed., 2006.
- Wie03: Wiegers, K., Software Requirements, 2nd ed., Microsoft Press, 2003.
