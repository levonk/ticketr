---
type: Practice
title: UX Usability Testing
description: Test whether a user can accomplish tasks without documentation by giving AI agents natural-language goals and observing if they succeed. Stagehand for web, finalrun-agent for mobile. AI-driven, BYOK token costs apply.
tags: [ux-testing, usability, stagehand, finalrun-agent, ai-testing, natural-language, web, mobile, byok]
date:
  created: "2026-08-08"
  knowledge-basis: "2026-08-08"
  last-used: "2026-08-08"
sources:
  - id: gh-browserbase-stagehand
    resource: "https://github.com/browserbase/stagehand"
    title: "Stagehand — The AI Browser Automation Framework"
  - id: gh-final-run-finalrun-agent
    resource: "https://github.com/final-run/finalrun-agent"
    title: "finalrun-agent — AI-driven CLI that tests mobile apps using natural language"
  - id: gh-kaeawc-auto-mobile
    resource: "https://github.com/kaeawc/auto-mobile"
    title: "AutoMobile — MCP server for mobile UX exploration"
---


# UX Usability Testing

## Failure Mode

Requirements coverage testing verifies that UI elements exist, but not that
a user can figure out how to use them. An application can have every
documented element present and still be unusable — confusing navigation,
unclear labels, hidden flows, dead-end states. Traditional usability testing
requires human testers, is expensive, slow, and does not scale.

## Practice

Give an AI agent a natural-language goal (not step-by-step instructions) and
observe whether it can accomplish the task using only the application's UI.
This is the "if I ask the user to do X, can they figure it out without my
documentation?" check — it does not verify that requirements are represented
(that is [UI Requirements Coverage Testing](ui-requirements-coverage.md)).

### Web: Stagehand

[Stagehand](https://github.com/browserbase/stagehand) is an AI browser
automation framework built on Playwright. It combines natural-language
actions with deterministic code.

```typescript
const page = stagehand.context.pages()[0];
await page.goto("https://localhost:3000");

// Natural-language action — the AI figures out how to accomplish it
await stagehand.act("complete the checkout flow with a test credit card");

// Multi-step task — the agent plans and executes
const agent = stagehand.agent();
await agent.execute("create a new project and invite a team member");

// Structured extraction — verify the outcome
const { success, errorMessage } = await stagehand.extract(
  "did the checkout complete successfully?",
  z.object({
    success: z.boolean(),
    errorMessage: z.string().optional(),
  }),
);
```

Key properties:
- **AI-driven** — `act()` and `agent.execute()` take natural-language goals,
  not selectors. The AI sees the page and decides how to interact.
- **Built on Playwright** — deterministic code works alongside AI actions.
  Switch between modes in the same script.
- **Self-healing** — auto-caching remembers previous actions. Runs without
  LLM inference on repeat. Re-involves AI when the website changes.
- **MIT licensed** — free, self-hosted. BYOK for LLM provider.
- **Cost control** — use Gemini 2.0 Flash (free tier) or local LLMs via
  Ollama for zero token cost. Stagehand caching skips AI calls after first
  run.

### Mobile: finalrun-agent

[finalrun-agent](https://github.com/final-run/finalrun-agent) is an AI-driven
CLI that tests iOS and Android apps using natural language. You write a
plain-English test in YAML; FinalRun launches the app, uses an AI model to
see the screen and perform each step, and produces a pass/fail report.

```yaml
# .finalrun/tests/checkout.yaml
name: "Complete checkout as a new user"
steps:
  - "Open the app and navigate to the product catalog"
  - "Add the first product to the cart"
  - "Proceed to checkout"
  - "Enter test payment details"
  - "Complete the purchase"
expected_state:
  - "Order confirmation screen is visible"
  - "Order number is displayed"
```

```bash
finalrun test checkout.yaml --platform android --model google/gemini-3-flash-preview
```

Key properties:
- **CLI-first** — `finalrun test` / `finalrun suite` commands. No SDK
  required.
- **AI-driven** — the AI sees the screen via screenshots and decides how to
  interact. No selectors, no accessibility IDs needed.
- **BYOK** — bring your own Google/OpenAI/Anthropic API key. Standard API
  billing applies.
- **Apache-2.0 licensed** — free, self-hosted.
- **Evidence** — video recordings, device logs, screenshots on failure.
- **Vercel skills** — ships `/finalrun-generate-test` and
  `/finalrun-test-and-fix` skills for AI agent integration.

### Alternative: AutoMobile (MCP-first)

[AutoMobile](https://github.com/kaeawc/auto-mobile) is an MCP server for
mobile UX exploration. It explicitly targets "UX deep dives" and "identify
confusing interactions." Apache-2.0, free, BYOK. Use when the MCP
integration pattern fits your agent stack better than a CLI.

### The Usability Contract

For each user task (not requirement), produce a usability test:

1. **Define the task** — a natural-language goal a user would attempt
   ("complete checkout", "invite a team member", "reset password"). Do not
   include step-by-step instructions — the point is to test if the UI is
   self-explanatory.
2. **Run the AI agent** — give the goal to Stagehand (web) or finalrun-agent
   (mobile). The AI attempts to accomplish it using only the UI.
3. **Observe** — did the agent succeed? How many steps did it take? Did it
   get stuck? Did it take wrong turns?
4. **Classify** — pass (task completed), partial (task completed with
   difficulty — wrong turns, backtracking), fail (task not completed).
5. **Report** — for partial/fail, capture the agent's reasoning and
   screenshots. These are usability issues to fix.

### CI Integration

Usability tests are slower and cost tokens, so run them less frequently than
coverage tests:

- **On PR** — run smoke usability tests (1-2 critical paths).
- **Nightly** — run the full usability suite.
- **On release** — run the full suite plus edge-case tasks.

### Graceful Missing-Key Handling

Usability testing requires LLM API keys. If keys are missing:

1. **Skip usability tests** — do not fail the build. Coverage tests
   (deterministic, no keys) still run.
2. **Log a warning** — "UX usability tests skipped: no LLM API key found.
   Set `GOOGLE_API_KEY` / `OPENAI_API_KEY` / `ANTHROPIC_API_KEY` to enable."
3. **Exit 0** for the usability test step — missing keys are a configuration
   gap, not a test failure.
4. **Suggest free-tier options** — Gemini 2.0 Flash free tier, Ollama local
   LLMs (web only).

This ensures the quality gate degrades gracefully: coverage testing always
runs, usability testing runs when keys are available.

## Related Concepts

- [UI Requirements Coverage Testing](ui-requirements-coverage.md) — the
  complementary "are all my requirements represented?" check using
  deterministic tools
- [Hybrid Playwright/Stagehand Testing](hybrid-playwright-stagehand.md) —
  the 80/20 split this practice extends with the usability dimension
- [Accessibility Testing](accessibility-testing.md) — a11y compliance
  prevents usability barriers for users with disabilities
- [Shared Quality Scripts](shared-quality-scripts.md) — quality scripts
  orchestrate both coverage and usability tests
