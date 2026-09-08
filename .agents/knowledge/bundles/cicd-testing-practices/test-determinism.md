---
type: Practice
title: Test Determinism
description: Tests must be deterministic — no flaky timing dependencies, no network calls without guardrails, no reliance on system state. A test that passes on Tuesday and fails on Wednesday is worse than no test at all because it erodes trust in the entire suite.
tags: [testing, determinism, flaky-tests, network-mocking, timing, reproducibility, ci-cd]
date:
  created: "2026-08-10"
  knowledge-basis: "2026-08-10"
  last-used: "2026-08-10"
---

# Test Determinism

## Failure Mode

A test passes on the developer's machine and fails in CI. Or it passes
in CI on the first run and fails on the retry. Or it passes 9 out of 10
times. Each flaky test erodes trust in the test suite: developers start
ignoring CI failures ("it's just the flaky one"), re-running until
green, and eventually a real regression slips through because nobody
believes the test anymore.

The root causes are always the same: the test depends on something that
is not under its control — a clock, a network endpoint, a filesystem
state, a process schedule, a random seed.

## Practice

Tests must be **deterministic**: given the same code and the same test
inputs, the test produces the same result every time, on every machine,
in every CI run. No exceptions.

### Core Principles

1. **No timing dependencies**: Never assert on elapsed time, never use
   `sleep(N)` as a synchronization mechanism, never rely on "the
   operation should take less than X seconds." Use deterministic
   synchronization: await promises, poll for conditions with a timeout,
   or use test hooks that signal readiness.
2. **No network without guardrails**: Tests that hit real network
   endpoints are non-deterministic — the endpoint may be down, slow, or
   return different data. Either mock the network layer entirely, or
   wrap the call in a retry-with-timeout that fails clearly (not
   flakily) when the endpoint is unavailable.
3. **No system state dependence**: Tests must not depend on the system
   clock, the system timezone, the user's home directory, or the
   presence of specific environment variables. Set up and tear down all
   state the test needs.
4. **No random without a fixed seed**: If a test uses randomness (e.g.,
   property-based testing, random data generation), the random seed
   must be fixed or logged so failures are reproducible. Most test
   frameworks support seeding; use it.
5. **No filesystem leakage**: Tests that write to the filesystem must
   write to a temporary directory that is cleaned up after the test. Do
   not write to the project directory or the user's home directory.
6. **No test ordering dependence**: Tests must pass in any order. If
   test B depends on test A having run first, that is an implicit
   coupling — extract the setup into a fixture or `beforeEach` hook.

### Anti-Patterns

| Anti-pattern | Why it flakes | Fix |
|--------------|---------------|-----|
| `await sleep(100); expect(x).toBe(5)` | Operation may take 101ms | Poll with timeout: `await waitFor(() => expect(x).toBe(5))` |
| `fetch("https://api.example.com/...")` | Endpoint may be down or slow | Mock `fetch`, or use a local test server |
| `expect(Date.now()).toBeGreaterThan(...)` | Clock advances during test | Inject a fixed clock via dependency injection |
| `fs.writeFileSync("./output.txt", ...)` | File persists between runs | Use `fs.mkdtemp()` + cleanup in `afterEach` |
| `Math.random()` in test data | Different input each run | Use a seeded PRNG: `Math.seedrandom("test-123")` |

## Concrete Instances

### Vitest (TypeScript)

```typescript
import { describe, test, expect, vi, beforeEach, afterEach } from 'vitest';

// Mock time — deterministic
beforeEach(() => {
  vi.useFakeTimers();
  vi.setSystemTime(new Date('2026-01-01T00:00:00Z'));
});
afterEach(() => {
  vi.useRealTimers();
});

// Mock network — deterministic
vi.mock('../api/client', () => ({
  fetchUser: vi.fn().mockResolvedValue({ id: 1, name: 'Test User' }),
}));

test('displays user name after fetch', async () => {
  // No sleep, no real network, no real clock — fully deterministic
  const result = await displayUser(1);
  expect(result).toBe('Test User');
});
```

Vitest provides built-in fake timers, module mocking, and deterministic
test isolation. The key: mock at the module boundary, not at the
network boundary. `vi.mock` replaces the entire module, so the test
never touches the real network.

### pytest (Python)

```python
import pytest
from freezegun import freeze_time
from unittest.mock import patch

@freeze_time("2026-01-01")
@patch("myapp.service.requests.get")
def test_fetch_user(mock_get):
    mock_get.return_value.json.return_value = {"id": 1, "name": "Test User"}
    mock_get.return_value.status_code = 200

    result = fetch_user(1)
    assert result == "Test User"
    # No real network, frozen clock — deterministic
```

pytest with `freezegun` (frozen clock) and `unittest.mock.patch` (mocked
network) gives the same determinism guarantees. For property-based
testing, `hypothesis` supports `@seed(12345)` to make random test
generation reproducible.

### cargo test (Rust)

```rust
use mockall::mock;

mock! {
    Client {}
    impl ApiClient for Client {
        fn fetch_user(&self, id: u64) -> Result<User, Error>;
    }
}

#[test]
fn test_fetch_user() {
    let mut mock = MockClient::new();
    mock.expect_fetch_user()
        .with(eq(1))
        .returning(|_| Ok(User { id: 1, name: "Test User".into() }));

    let result = fetch_user(&mock, 1);
    assert_eq!(result.name, "Test User");
    // No real network, no timing — deterministic
}
```

Rust's `mockall` generates mock implementations at compile time. For
time-dependent code, inject a `Clock` trait rather than calling
`SystemTime::now()` directly. For property-based testing, `proptest`
supports deterministic shrinking with a fixed seed.

## Prevention

1. **Run tests in random order** — if the test runner supports it
   (Vitest: `--sequence.shuffle`, pytest: `pytest-randomly`), enable
   random test ordering in CI. Tests that pass only in one order have an
   implicit dependency — random ordering surfaces it early.
2. **Run the suite multiple times in CI** — for critical paths, run the
   test suite 3× in CI. If any run fails, the test is flaky and must be
   fixed or quarantined.
3. **Quarantine flaky tests immediately** — a flaky test that is left
   in the suite teaches developers to ignore failures. Move it to a
   quarantine suite that does not block CI, fix it, then move it back.
4. **Log the random seed** — if a test uses randomness, log the seed on
   failure so the exact run can be reproduced. Most frameworks support
   this; enable it.
5. **Assert on state, not timing** — instead of "the operation completed
   in under 100ms," assert "the operation produced result X." If
   performance matters, write a separate benchmark with explicit
   tolerance bands — not a test that flakes when CI is slow.

## Related Concepts

- [Test Isolation — Mock Pollution](test-isolation-mock-pollution.md) —
  Mock pollution is a specific source of non-determinism; isolated test
  processes prevent it
- [Pre-Commit CI Parity](pre-commit-ci-parity.md) — Deterministic tests
  are a prerequisite for local/CI parity; a flaky test cannot have parity
- [Property-Based Testing (Rust)](property-based-testing-rust.md) —
  Property-based tests use seeded randomness for reproducible failures
- [Vitest Unified Runner](vitest-unified-runner.md) — Vitest's built-in
  isolation and fake-timer support for deterministic tests
