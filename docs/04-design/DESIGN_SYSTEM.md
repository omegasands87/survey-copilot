# Design System

## 1. Principles
- Functional clarity over decoration.
- Neutral surfaces and restrained accent colors.
- Consistent spacing, typography, and interaction states.
- Status always includes text or an icon plus text; never color alone.

## 2. Tokens
Define centralized tokens for:
- Color roles: background, surface, text, muted text, border, primary, destructive, success, warning, focus.
- Typography: display, heading, body, label, caption, monospace.
- Spacing: a consistent 4px-based scale.
- Radius: small, medium, large.
- Elevation: minimal, reserved for overlays.
- Motion: short transitions with reduced-motion support.

Exact values should be selected during visual implementation and documented in the theme source.

## 3. Components
Button, IconButton, TextField, TextArea, Select, Dialog, Tabs, Sidebar, Breadcrumb, StatusLabel, FactCard, EvidenceList, QuestionCard, RecommendationPanel, EmptyState, Toast, ConfirmDialog.

## 4. States
Every interactive component defines default, hover, focus-visible, active, disabled, loading, and error states as appropriate.

## 5. Content style
Use concise, neutral language. Prefer “Not enough information” to “AI failed.” Prefer “Evidence supports this draft” to “High chance of success.”
