# Survey Copilot

Desktop AI copilot for understanding online survey questions and recommending truthful, profile-grounded answers.

- **Platform:** Windows 11 x64
- **Language:** Indonesian
- **Users:** One local operator managing multiple real respondent profiles
- **Input:** User-selected screenshot capture or pasted text
- **AI:** Gemini API pool with custom compatible provider/endpoint/model support
- **Use:** AI recommends; user manually fills and submits surveys
- **Business model:** Free personal use

## What it does

1. Reads a survey question from the screen or pasted text.
2. Understands the question and its answer format.
3. Uses the selected respondent profile and checks consistency.
4. Recommends an answer with evidence—or asks when information is missing.
5. Compares visible survey criteria against available profiles.
6. Uses a pool of Gemini API keys and can fail over when a key is limited or unavailable.

## Product boundaries

- Multiple real-persona profiles are isolated.
- Recommendations must be truthful and evidence-grounded.
- Screen capture is user initiated and visible.
- The user reviews AI recommendations and performs all survey interactions.
- No browser automation, autofill, auto-click, auto-submit, CAPTCHA bypass, or screener deception.
- No earnings, reward, points, balance, or income tracking in v1.0.
- API secrets are stored in Windows Credential Manager.

## Documentation

Start at [Documentation Index](docs/00-INDEX.md). The [Product Definition](docs/01-product/PRODUCT_DEFINITION.md) is the authoritative scope.

## Technology direction

Tauri 2, Rust, React, TypeScript, SQLite, Windows Credential Manager, local OCR, and a provider adapter architecture with Gemini first.

## Status

An initial React + Tauri application shell is now in the repository. Core features (persona storage, screen capture/OCR, Gemini API pooling, answer analysis, and installer validation) are not implemented yet. Do not treat planned capabilities as shipped.
