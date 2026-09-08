---
type: Practice
title: Mandatory Testing Workflow
description: TDD-first development with mandatory regression tests for bug fixes, comprehensive coverage requirements, and quality gates enforced via pre-commit hooks and CI/CD.
tags: [testing, tdd, quality-gates, regression, pre-commit, ci-cd]
date:
  created: "2026-07-18"
  knowledge-basis: "2026-07-17"
  last-used: "2026-07-17"
sources:
  - id: levonk-base-boilerplate-testing-requirements-section
    resource: internal-docs/adr/adr-20260131001-standard-developer-ux-flow.md
    title: levonk-base-boilerplate (Testing Requirements section)
---

# Mandatory Testing Workflow

## Failure Mode

Changes land without tests, regressions slip through, and quality gates are
inconsistent across projects. Without mandatory testing requirements, "it works
on my machine" replaces verified correctness.

## Practice

Tests are **mandatory** for ALL changes. The standard developer UX flow
enforces testing at every level.

### Core Principles

1. **Test-First Development (TDD)**: Write failing tests before implementing
   features
2. **Regression Tests**: Every bug fix must include tests to prevent recurrence
3. **Comprehensive Coverage**: Test both happy paths and edge cases
4. **Quality Gates**: All tests must pass before considering work complete

### Required Test Types

**For New Features**:
- Unit tests for core functionality
- Integration tests for component interactions
- End-to-end tests for complete workflows
- Error handling and edge case tests

**For Bug Fixes**:
- Regression test that reproduces the original bug
- Verification test that confirms the fix works
- Additional tests to cover related edge cases

**For Refactoring**:
- Existing tests must continue to pass
- New tests for any added functionality
- Performance tests if behavior affects performance

### Testing Workflow

```bash
# Standard testing workflow across all flows
just test           # Run all tests (auto-detects devbox)
just test feature   # Run specific test
just test -- --nocapture  # Run with output

# Quality gates before completion
just test && just lint
```

### Enforcement

- **Pre-commit hooks**: Prevent commits without passing tests
- **CI/CD pipelines**: Fail builds without comprehensive test coverage
- **Code review**: Reviewers must verify adequate test coverage
- **Documentation**: Test requirements documented in project AGENTS.md

### Quality Check Target

```just
quality:
    just lint
    just test
    just typecheck
```

## Related Concepts

- [Standard Developer UX Flow](standard-developer-ux-flow.md) — The workflow that
  enforces testing
- [Internal vs Normal Targets](internal-vs-normal-targets.md) — Test targets
  follow the naming convention
