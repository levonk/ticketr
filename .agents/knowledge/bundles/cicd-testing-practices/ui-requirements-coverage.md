---
type: Practice
title: UI Requirements Coverage Testing
description: Verify every documented UI requirement is represented in the running application using deterministic accessibility-tree inspection. agent-browser for web, agent-device for mobile. CLI-first, no AI tokens required.
tags: [ui-testing, requirements-coverage, agent-browser, agent-device, accessibility-tree, deterministic, cli, web, mobile]
date:
  created: "2026-08-08"
  knowledge-basis: "2026-08-08"
  last-used: "2026-08-08"
sources:
  - id: gh-vercel-labs-agent-browser
    resource: "https://github.com/vercel-labs/agent-browser"
    title: "agent-browser — Browser automation CLI for AI agents"
  - id: gh-callstack-agent-device
    resource: "https://github.com/callstack/agent-device"
    title: "agent-device — Device automation CLI for AI agents"
---


# UI Requirements Coverage Testing

## Failure Mode

Requirements documents list UI elements, flows, and states that the
application should expose. Without deterministic verification, elements go
missing, flows break silently, and states are unreachable — none caught until
manual QA or user reports. Manual cross-checking is expensive, inconsistent,
and skipped under deadline pressure.

## Practice

Use deterministic accessibility-tree inspection to verify that every
documented UI requirement is present and reachable in the running
application. This is the "are all my requirements represented?" check — it
does not test whether a user can figure out how to use them (that is
[UX Usability Testing](ux-usability-testing.md)).

### Web: agent-browser

[agent-browser](https://github.com/vercel-labs/agent-browser) is a native
Rust CLI for browser automation. It uses the inspect-act-verify pattern:
take an accessibility snapshot, identify elements by ref, interact, verify.

```bash
agent-browser open http://localhost:3000
agent-browser snapshot                    # Accessibility tree with element refs
agent-browser find role button text --name "Submit"   # Verify button exists
agent-browser find label "Email" text                  # Verify email field exists
agent-browser find testid "checkout-confirm" click     # Verify checkout flow reachable
agent-browser screenshot --annotate                     # Evidence capture
agent-browser close
```

Key properties:
- **CLI-first** — no SDK, no test framework lock-in. Commands compose in
  shell scripts or AI agent loops.
- **Accessibility tree** — elements identified by role, name, label, or
  `data-testid`, not brittle CSS selectors.
- **No AI tokens** — deterministic inspection. Zero per-run cost beyond the
  browser process.
- **MIT licensed** — free, self-hosted, no vendor lock-in.

### Mobile: agent-device

[agent-device](https://github.com/callstack/agent-device) extends the same
inspect-act-verify pattern to iOS, Android, tvOS, and web. Uses XCTest on
iOS and ADB on Android.

```bash
agent-device open MyApp --platform ios
agent-device snapshot -i                   # Accessibility tree with element refs
agent-device press @e2 --settle            # Interact by ref
agent-device fill @e7 "test@example.com" --settle
agent-device screenshot ./evidence.png     # Evidence capture
agent-device close
```

Key properties:
- **CLI-first** — same command structure as agent-browser. Agents switch
  between web and mobile with minimal context change.
- **MIT licensed** — free, self-hosted.
- **Replay scripts** — record a session as `.ad` script for CI replay.
- **React Native support** — component tree inspection beyond accessibility
  snapshots.

### The Coverage Contract

For each documented UI requirement, produce a verification step:

1. **Parse the requirements** — read PRD, user stories, or acceptance
   criteria. Extract UI elements, flows, and states.
2. **Map to accessibility queries** — each requirement becomes a
   `find role`/`find label`/`find testid` command (web) or a snapshot ref
   lookup (mobile).
3. **Execute** — run the commands against the running application.
4. **Report gaps** — any requirement that produces no match is a coverage
   gap. Output a structured report: requirement ID, query, result (found /
   not found / ambiguous).

### CI Integration

Record coverage sessions as replay scripts and run them in CI:

- Web: `agent-browser` session scripts replayed in GitHub Actions.
- Mobile: `agent-device` `.ad` scripts replayed in EAS workflows or GitHub
  Actions.

Coverage gaps fail the CI build — a missing requirement is a regression.

### Graceful Missing-Key Handling

Both tools work without API keys. agent-browser and agent-device use
deterministic accessibility inspection — no LLM calls, no tokens, no API
keys required. If the application is running and the browser/device is
available, coverage testing works.

## Related Concepts

- [UX Usability Testing](ux-usability-testing.md) — the complementary
  "can the user figure it out?" check using AI-driven tools
- [Hybrid Playwright/Stagehand Testing](hybrid-playwright-stagehand.md) —
  the 80/20 split this practice extends with requirements-coverage and
  usability dimensions
- [Accessibility Testing](accessibility-testing.md) — a11y compliance is a
  subset of requirements coverage
- [Pre-Commit CI Parity](pre-commit-ci-parity.md) — coverage tests run in
  both local and CI contexts
