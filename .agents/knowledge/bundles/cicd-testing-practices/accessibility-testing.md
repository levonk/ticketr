---
type: Practice
title: Accessibility Testing in CI
description: axe-core accessibility scanning in CI pipeline. WCAG 2.1 AA compliance from day one. Semantic HTML, keyboard navigation, ARIA, color contrast 4.5:1. Zero critical violations required.
tags: [accessibility, a11y, axe-core, wcag, ci-cd, compliance, testing]
date:
  created: "2026-07-18"
  knowledge-basis: "2026-07-17"
  last-used: "2026-07-17"
sources:
  - id: feat-202607170936-bookkeeping-saas-mvp
    resource: "internal-docs/feature/2026/07/bookkeeping-saas-mvp/feat-202607170936-bookkeeping-saas-mvp.md"
    title: "bookkeep-saas PRD NFR21-NFR24"
---


# Accessibility Testing in CI

## Failure Mode

Accessibility violations ship to production. WCAG non-compliance creates legal
risk and excludes users. Manual accessibility audits are expensive and
inconsistent.

## Practice

**WCAG 2.1 AA compliance from day one.** axe-core scanning in CI.

### Requirements

- **Semantic HTML**: Use proper HTML elements for their intended purpose
- **Keyboard navigation**: All interactive elements accessible via keyboard
- **ARIA**: Proper ARIA attributes for dynamic content
- **Color contrast**: 4.5:1 minimum ratio
- **Forms**: Labels, `aria-live` error regions, keyboard accessible

### CI Integration

```bash
pnpm run a11y    # axe-core accessibility scan
```

- Zero critical violations required for CI to pass
- Scans run on all pages with interactive elements
- Integrated into the shared quality script pipeline

### A11y in Auth/Billing Pages

From bookkeep-saas PRD (NFR21-NFR24):
- Login, signup, forgot/reset password pages — 0 critical violations
- Billing page — 0 critical violations
- All UI strings externalized to locale files (i18n)

### Why From Day One

- Retrofitting accessibility is 10x more expensive than building it in
- Legal compliance (ADA, Section 508) requires WCAG conformance
- Accessibility improves UX for all users, not just those with disabilities

## Related Concepts

- [Pre-Commit CI Parity](pre-commit-ci-parity.md) — a11y checks run in both
  contexts
- [Shared Quality Scripts](shared-quality-scripts.md) — axe-core invoked through
  quality script
