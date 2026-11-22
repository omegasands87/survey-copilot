# Survey Copilot — Blueprint

**Status:** High-level summary  
**Authority:** [Product Definition](01-product/PRODUCT_DEFINITION.md)

## Product purpose

Survey Copilot is a simple Windows 11 desktop AI copilot that reads online survey questions from screenshots or pasted text, understands them, uses the selected real respondent profile, and recommends truthful, consistent answers. It also helps compare visible survey eligibility criteria against the user's profiles.

The user operates survey websites normally. The app recommends; the user manually fills and submits all responses.

## Core capabilities

1. **Multiple Digital Personas:** isolated real respondent profiles, facts, sources, and histories.
2. **Screen Intelligence:** user-initiated region/window capture, OCR, visual layout reconstruction, and correction.
3. **Question Intelligence:** classify intent, answer format, timeframe, units, and constraints.
4. **Answer Copilot:** retrieve relevant confirmed facts, check consistency, recommend a supported answer, or ask/abstain.
5. **Survey Fit:** compare visible criteria against each persona and explain match, mismatch, unknown, stale, or conflict.
6. **AI Provider Hub:** Gemini-first API pool with multiple keys, automatic failover, bounded retry, and custom compatible provider/endpoint/model support.
7. **History:** preserve AI recommendations and user edits/confirmations separately.

## Owner-confirmed scope

- Windows 11 x64.
- Indonesian-first interface.
- Free personal use.
- One local operator with multiple real-persona profiles.
- Screenshot and manual text input.
- Gemini API pool and custom provider configuration.
- User manually performs all survey actions.
- No earnings, reward, points, balance, payout, or income tracking in v1.0.

## Integrity boundaries

- No fabricated facts or fictional respondent profiles.
- No cross-persona fact/history mixing.
- No deceptive screener optimization.
- No silent/background screen capture.
- No browser automation, autofill, clicking, navigation, auto-submit, CAPTCHA bypass, or platform circumvention.
- No unsupported recommendation presented as certain.

## Technical direction

Tauri 2, Rust, React, TypeScript, SQLite, Windows Credential Manager, local OCR, Gemini adapter, and extensible provider pooling. See the architecture specifications for implementation detail.

## Project status

The repository contains specifications and planning documents. Application source code has not yet been implemented.
