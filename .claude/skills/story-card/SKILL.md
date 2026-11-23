---
name: story-card
description: Capture a raw idea as a story card before writing requirements. Use when a request is a fuzzy idea, feature wish, technical debt, or architecture change that needs its root reason, value, and consequence of not doing it recorded before a PRD, SRS, or FIP.
---

# Story card

Capture an idea in a story card before moving to requirements and code.

## Story types

- User story
  - Format: "As a <role>, I want <goal> so that <purpose>."
  - Written by the product owner.
- Technical debt story
  - Example: "As a developer I need to break up the TicketRouting class to increase maintainability of the routing functionality."
  - Written by developers.
- Architecture story
  - Example: "As an architect I need to break up the Payment Service to support better extensibility when adding new payment methods."
  - Written by architects.

## Writing story cards

- Write the story card in the matching story type format.
- Ask the five whys to find the goal or wanted outcome.
  Record the root reason for doing the work in the card.
- Record what happens if the card is not done.
- Order the cards by priority.
- Move the first batch to the analysis phase and execute in iterations.

Planning game, Statement of Work (SOW), and estimation follow requirements analysis.
Leave those fields empty when only capturing the idea, and mark them as open for the architect or developers.

## Story card fields

Adapted from Bec00, 88:

- Id: Unique reference that distinguishes the card from others.
- Tag: Short unique name, when the Id alone is not enough.
- Project: Project the card belongs to, for grouping.
- Headline: Summary of what the card achieves.
- Description: What the card achieves.
  Can link to a use case.
- Priority: Critical, Value adding, or Nice to have.
- Customer: Who asked for or benefits from the card.
- Tech: Technology or component affected.
- Complexity: Expected difficulty.
- What if not done: Consequence of not implementing the card.
- Estimation: Technical estimate.
  Track the estimate alongside the SOW it was made against, since both can change.
- SOW: Technical description of the expected work, with best case, worst case, and expected time.
- Risk: Risks associated with the work.
- Dependency: Other work the card depends on.
- Type of work: New, Fix, or Enhancement.
- Date: Date the card was created.

## Output format

```markdown
### <Id> <Headline>

- Tag:
- Project:
- Type of work:
- Story: As a <role>, I want <goal> so that <purpose>.
- Root reason (five whys):
- Description:
- Customer:
- Priority:
- What if not done:
- Tech:
- Complexity:
- Risk:
- Dependency:
- Estimation: open
- SOW: open
- Date:
```

## Audiences

- Customer: Sees the fuzzy requirement, its importance, and the consequence of not doing it.
- Developer (implementer): Understands what to implement and why.
- Developer (maintainer): Understands how a later change came about.
- Manager (dev): Understands how and where the card fits the schedule.
- QA: Uses cards for continuous improvement and statistics.

## References

- Bec00: Beck, K. and Fowler, M., Planning Extreme Programming, Addison-Wesley, 2000.
- [Lesson 106 - Architecture Stories](https://www.youtube.com/watch?v=Z0HMnZyoHP0)
