---
type: Practice
title: Test Isolation — Mock Pollution
description: Module-level mocks permanently replace modules in the process-wide cache — mock.restore() does not undo them. Tests that mock the same module with different implementations pollute each other when run in the same process. Split conflicting mocks into separate test invocations.
tags: [testing, mock-pollution, test-isolation, process-global, bun, jest, pytest]
date:
  created: "2026-08-10"
  knowledge-basis: "2026-08-10"
  last-used: "2026-08-10"
---

# Test Isolation — Mock Pollution

## Failure Mode

Test file A mocks `module-X` to return `{ status: "ok" }`. Test file B
mocks `module-X` to return `{ status: "error" }`. Both pass when run
individually. When run together in the same process, one of them fails —
because the mock from the first file loaded persists into the second
file's execution. `mock.restore()` does not undo it. The test suite
reports ~135 spurious failures that disappear when each file runs in
isolation.

This is **mock pollution**: module-level mocks that permanently replace
modules in the process-wide module cache. The replacement is
process-global and irreversible within that process. No teardown hook,
no `afterEach`, no `mock.restore()` can undo it — the module cache has
been mutated for the lifetime of the process.

## Practice

Split test files with conflicting module-level mocks into **separate
test invocations** (separate processes). Within a single invocation,
ensure no two test files mock the same module with different
implementations.

### Core Principles

1. **Mock pollution is process-global and irreversible**: Once a
   module-level mock is installed, it persists for the lifetime of the
   process. `mock.restore()`, `mock.clearAllMocks()`, and `afterEach`
   hooks do not undo it. This is a platform limitation, not a bug in
   your test code.
2. **One module, one mock per process**: If two test files need to mock
   the same module differently, they cannot run in the same process.
   Split them into separate invocations.
3. **Per-package isolation**: In a monorepo, run each package's tests as
   a separate process invocation. This prevents cross-package mock
   pollution and gives per-package test isolation by construction.
4. **Never run all tests in one process**: A root-level `test` command
   that discovers all test files and runs them in a single process is a
   mock-pollution trap. Use a per-package or per-batch invocation
   instead.

### Diagnosis

Mock pollution is hard to diagnose because:

- Tests pass individually but fail when run together.
- The failure message points to the wrong test file (the one that
  consumed the polluted mock, not the one that installed it).
- Adding or removing an unrelated test file can change which tests fail
  (it changes the file load order).
- The number of failures is large (~100+) and the failures seem
  unrelated.

The signature: a large number of test failures that disappear when each
file is run in isolation, and the failures involve mocked modules.

## Concrete Instances

### Bun (mock.module)

Bun's `mock.module()` permanently replaces modules in the process-wide
cache. `mock.restore()` does **not** undo it
([oven-sh/bun#7823](https://github.com/oven-sh/bun/issues/7823)).

```json
{
  "scripts": {
    "test": "bun --filter '*' --parallel test && bun test ./scripts/"
  }
}
```

The `bun --filter '*' --parallel test` invocation runs each workspace
package's tests as a separate process. Packages with conflicting
`mock.module()` calls are further split into batches within their
`package.json`:

```json
{
  "scripts": {
    "test": "bun test src/handlers/ && bun test src/adapters/ && bun test src/utils/"
  }
}
```

Each `bun test <dir>` invocation is a separate process. Test files
within the same directory must not mock the same module with different
implementations.

**Do NOT run `bun test` from the repo root** — it discovers all test
files across all packages and runs them in one process, causing
mock-pollution failures. Always use `bun run test` (which uses
per-package isolation via `bun --filter`).

### Jest (jest.mock)

Jest's `jest.mock()` has similar process-global behavior, though Jest's
default test runner creates a separate module registry per test file
( isolating mocks between files by default). However, custom runners or
`--testPathPattern` configurations that share a registry can trigger
the same pollution:

```javascript
// file-a.test.js
jest.mock('./config', () => ({ debug: true }));

// file-b.test.js
jest.mock('./config', () => ({ debug: false }));
```

With Jest's default isolation, these run in separate module registries
and do not conflict. But if `--no-isolateModules` is set (for
performance), they pollute each other. The fix: keep module isolation
enabled, or split into separate `jest` invocations.

### pytest (monkeypatch / unittest.mock.patch)

```python
# test_a.py
@patch("myapp.config.get_settings")
def test_a(mock_get_settings):
    mock_get_settings.return_value = Settings(debug=True)

# test_b.py
@patch("myapp.config.get_settings")
def test_b(mock_get_settings):
    mock_get_settings.return_value = Settings(debug=False)
```

pytest's `unittest.mock.patch` decorator is scoped to the test function
and restored after the test — it does **not** pollute between files by
default. However, `monkeypatch.setattr` at module level (outside a test
function) or `conftest.py` fixtures with `autouse=True` that patch
modules can cause cross-file pollution if the patch is not properly
scoped. The fix: ensure patches are function-scoped, or run conflicting
files with `pytest -p no:cacheprovider --forked` (separate process per
test).

## Prevention

1. **Use per-package test invocations** — in a monorepo, each package's
   tests run as a separate process. This is the primary defense against
   cross-package mock pollution.
2. **Split conflicting mocks into batches** — within a package, if two
   groups of test files mock the same module differently, split them
   into separate `test` script entries (separate process invocations).
3. **Never run a root-level "all tests in one process" command** — if
   the test runner discovers all files and runs them in one process,
   mock pollution is inevitable for any non-trivial codebase.
4. **Document the split** — in the project's AGENTS.md or testing docs,
   list the test batches and explain why they are split. A new
   contributor who merges the batches will rediscover the pollution.
5. **Prefer dependency injection over mocking** — if a module's
   dependencies are injected (passed as parameters), tests can pass
   fakes without module-level mocking. This eliminates the pollution
   risk entirely. See
   [dependency injection](https://github.com/levonk/skills-releases/blob/main/knowledge/software-architecture-essentials/dependency-injection-pattern.md)
   for the general pattern.

## Related Concepts

- [Test Determinism](test-determinism.md) — Mock pollution is one source
  of non-determinism; deterministic tests also avoid timing and network
  dependencies
- [Pre-Commit CI Parity](pre-commit-ci-parity.md) — The test split must
  be identical between local and CI invocations
- [Vitest Unified Runner](vitest-unified-runner.md) — Vitest's module
  isolation model and how it differs from Bun's
