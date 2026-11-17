# UI/UX Specification

## 1. Experience goals
- Clear, calm, and evidence-oriented.
- Make uncertainty and provenance easy to inspect.
- Keep the user in control of every external action.
- Avoid visual cues that imply guaranteed survey success.

## 2. Navigation
Primary areas:
1. Dashboard
2. Digital Persona
3. Survey Workspace
4. Answer History
5. Settings

The recommendation review is a focused state within Survey Workspace rather than a separate autonomous workflow.

## 3. Core recommendation panel
Show in this order:
1. Evidence status.
2. Draft answer or abstention.
3. Supporting facts with source/date.
4. Rationale and limitations.
5. Missing information or conflict.
6. User actions: Edit, Confirm, Copy, Reject.

Do not hide evidence behind a tooltip or require navigation away from the draft to inspect it.

## 4. Interaction rules
- Copy requires explicit user action and visible confirmation.
- Confirmation must not imply submission to a survey platform.
- Destructive actions require clear scope and confirmation.
- Preserve unsaved text on recoverable provider errors.
- Keyboard focus must remain predictable after dialogs and state changes.
- Do not use color alone to convey support status.

## 5. Empty and error states
Explain what the user can do next. Distinguish no profile facts, no relevant facts, provider unavailable, invalid output, database error, and unresolved conflict.

## 6. Accessibility
Target WCAG 2.2 AA principles where applicable: keyboard access, visible focus, semantic labels, contrast, scalable text, reduced motion, and screen-reader announcements for status changes.

## 7. Research
Validate navigation labels, evidence comprehension, and whether users understand the difference between Confirm and Copy through usability testing.
