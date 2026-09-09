---
type: Practice
title: CI Matrix Strategy
description: Multi-OS CI matrix with fail-fast disabled so every leg reports independently, proactive disk-space reclamation for large builds, and a smoke test that verifies the built artifact actually starts and serves — not just that it compiles.
tags: [ci-cd, matrix-strategy, fail-fast, multi-os, space-reclamation, smoke-test, github-actions, gitlab-ci]
date:
  created: "2026-08-10"
  knowledge-basis: "2026-08-10"
  last-used: "2026-08-10"
---

# CI Matrix Strategy

## Failure Mode

A CI pipeline runs only on Linux. A developer pushes a change that
works on Linux but breaks on Windows (path separators, shell
differences, platform-specific APIs). The breakage is invisible until a
user reports it — weeks later, with no CI evidence to diagnose from.

Or: CI runs on multiple OSes with `fail-fast: true` (the default). The
Linux leg fails first, and the Windows leg is cancelled. A
Windows-only break stays hidden until the next round — and a cancelled
leg blocks the merge because it never reported a status.

Or: a Docker image build fails with "no space left on device" because
the runner's disk filled up with cached toolchains and SDKs that the
build does not need. The failure is intermittent (cached builds squeak
by; cold builds fail) and hard to reproduce.

Or: the CI builds a Docker image and reports success — but the image
does not start. The entrypoint has a runtime error that no compile-time
check can catch. The broken image ships to production.

## Practice

A CI matrix strategy that ensures **every platform reports
independently**, **disk space is proactively reclaimed**, and **the
built artifact is smoke-tested** before the pipeline passes.

### Core Principles

1. **Multi-OS matrix**: Run the full test suite on every OS the project
   supports (typically `ubuntu-latest` + `windows-latest`). Do not
   assume Linux-only — even server-side projects may have developers on
   Windows or platform-specific build steps.
2. **Fail-fast disabled**: Set `fail-fast: false` so every matrix leg
   runs to completion regardless of other legs' status. A cancelled leg
   reports no status, which blocks merges (required checks never
   report). Both legs must always report pass or fail — never
   "cancelled."
3. **Platform-conditional steps**: Some tests are platform-specific
   (e.g., a POSIX-shell installer test that cannot run on Windows
   MINGW). Use `if: runner.os != 'Windows'` to skip them on the wrong
   platform rather than letting them fail.
4. **Proactive space reclamation**: CI runners have limited disk space
   (~14GB free on a stock GitHub Actions runner). Large builds
   (multi-platform Docker images, heavy SDK vendor binaries) can
   exhaust it. Reclaim space before the build by removing pre-installed
   toolchains the build does not need.
5. **Smoke test the artifact**: After building a container image (or
   other deployable artifact), run a smoke test: start the container,
   hit a health endpoint, verify it responds. A successful build does
   not mean the artifact runs.

### Matrix Configuration

```yaml
strategy:
  # Both legs must always report. With the default fail-fast, an ubuntu
  # failure cancels windows, so a windows-only break stays invisible.
  fail-fast: false
  matrix:
    os: [ubuntu-latest, windows-latest]
runs-on: ${{ matrix.os }}
```

The `fail-fast: false` comment is important — it explains **why** the
default is overridden. Without the comment, a future contributor may
"fix" it back to `true` and reintroduce the visibility problem.

### Space Reclamation

```yaml
- name: Free runner disk space
  run: |
    sudo rm -rf /usr/share/dotnet /usr/local/lib/android /opt/ghc /opt/hostedtoolcache/CodeQL
    sudo docker image prune --all --force
    df -h /
```

This removes pre-installed .NET, Android, Haskell, and CodeQL toolchains
that most projects do not use, reclaiming ~14GB. The `df -h /` at the
end logs the free space for diagnosis. Run this step **before** the
build step, not after.

### Smoke Test

```yaml
- name: Build Docker image
  uses: docker/build-push-action@v6
  with:
    push: false
    load: true
    tags: myapp-ci:test
    cache-from: type=gha
    cache-to: type=gha,mode=max

- name: Smoke test — container starts and serves /api/health
  run: |
    docker run -d --name myapp-smoke -e PORT=3000 -p 3000:3000 myapp-ci:test
    sleep 5
    curl --fail --retry 10 --retry-delay 3 --retry-all-errors http://localhost:3000/api/health

- name: Dump container logs on failure
  if: failure()
  run: docker logs myapp-smoke 2>&1 || true

- name: Cleanup smoke test container
  if: always()
  run: docker rm -f myapp-smoke || true
```

The smoke test starts the container, waits for it to be ready (with
retries), and hits a health endpoint. If the container fails to start
or the health check fails, the step fails and the pipeline blocks. The
`if: failure()` step dumps logs for diagnosis, and `if: always()`
cleans up the container even on failure.

## Concrete Instances

### GitHub Actions

```yaml
jobs:
  test:
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - name: Setup Bun
        uses: oven-sh/setup-bun@v2
        with:
          bun-version: 1.3.11
      - run: bun install --frozen-lockfile
      - run: bun run type-check
      - run: bun run lint --max-warnings 0
      - run: bun run format:check
      - name: Run installer tests
        if: runner.os != 'Windows'  # POSIX-only installer
        run: bun run test:install
      - run: bun run test

  docker-build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Free runner disk space
        run: |
          sudo rm -rf /usr/share/dotnet /usr/local/lib/android /opt/ghc /opt/hostedtoolcache/CodeQL
          sudo docker image prune --all --force
          df -h /
      - name: Build Docker image
        uses: docker/build-push-action@v6
        with:
          push: false
          load: true
          tags: myapp-ci:test
          cache-from: type=gha
          cache-to: type=gha,mode=max
      - name: Smoke test
        run: |
          docker run -d --name myapp-smoke -e PORT=3000 -p 3000:3000 myapp-ci:test
          sleep 5
          curl --fail --retry 10 --retry-delay 3 --retry-all-errors http://localhost:3000/api/health
      - name: Dump logs on failure
        if: failure()
        run: docker logs myapp-smoke 2>&1 || true
      - name: Cleanup
        if: always()
        run: docker rm -f myapp-smoke || true
```

The test job runs on both Ubuntu and Windows with `fail-fast: false`.
The installer test is skipped on Windows (`if: runner.os != 'Windows'`)
because the POSIX-shell installer refuses MINGW/MSYS by design. The
docker-build job reclaims space, builds the image with layer caching,
and smoke-tests it with a health-endpoint check.

### GitLab CI

```yaml
test:
  parallel:
    matrix:
      - OS: [ubuntu-latest, windows-latest]
  tags:
    - ${OS}
  script:
    - bun install --frozen-lockfile
    - bun run type-check
    - bun run lint --max-warnings 0
    - bun run test
  allow_failure: false  # equivalent to fail-fast: false per-job

docker-build:
  tags:
    - ubuntu-latest
  before_script:
    - df -h /
    - sudo rm -rf /usr/share/dotnet /usr/local/lib/android /opt/ghc || true
  script:
    - docker build -t myapp-ci:test .
    - docker run -d --name myapp-smoke -e PORT=3000 -p 3000:3000 myapp-ci:test
    - sleep 5
    - curl --fail --retry 10 --retry-delay 3 http://localhost:3000/api/health
  after_script:
    - docker rm -f myapp-smoke || true
```

GitLab CI's `parallel: matrix` provides the same multi-OS fan-out.
GitLab does not have a global `fail-fast` setting — each job reports
independently by default. The space reclamation and smoke test patterns
are identical.

### Concurrency Control

Both GitHub Actions and GitLab CI support concurrency cancellation —
canceling in-progress runs when a new commit is pushed to the same
branch. This is orthogonal to fail-fast: concurrency cancellation
avoids **wasting runner minutes** on superseded commits, while
`fail-fast: false` ensures **all legs of a single commit's matrix**
report independently.

```yaml
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true
```

Use both: `cancel-in-progress: true` to save runner minutes, and
`fail-fast: false` to ensure every leg of the current commit's matrix
completes.

## Prevention

1. **Comment the `fail-fast: false`** — explain why the default is
   overridden. Without the comment, someone will "simplify" it back.
2. **Log disk space before and after reclamation** — `df -h /` before
   and after the cleanup step makes it easy to diagnose space issues.
3. **Use `--retry` on smoke-test health checks** — the container may
   need a few seconds to start. `curl --fail --retry 10 --retry-delay 3
   --retry-all-errors` gives it up to 30 seconds while failing
   deterministically (not flakily) if it never comes up.
4. **Dump logs on failure** — `if: failure()` steps that print
   container logs transform "smoke test failed" from a mystery into a
   diagnosable error.
5. **Clean up with `if: always()`** — containers and temp resources
   must be cleaned up even when the step fails, otherwise they
   accumulate on the runner and cause space issues on subsequent runs.
6. **Cache aggressively** — `cache-from: type=gha` and
   `cache-to: type=gha,mode=max` for Docker builds, and Actions cache
   for dependency directories. Cold builds are slow and may hit space
   limits that cached builds avoid.

## Related Concepts

- [Pre-Commit CI Parity](pre-commit-ci-parity.md) — The checks run in
  the matrix must match the checks run locally via the validation gate
- [Pre-PR Validation Gate](https://github.com/levonk/skills-releases/blob/main/knowledge/dev-environment-practices/pre-pr-validation-gate.md)
  — The single-command gate that ensures CI parity before pushing
- [Test Determinism](test-determinism.md) — A flaky test makes the
  matrix unreliable; deterministic tests are a prerequisite for
  trustworthy multi-OS results
- [Rust CI Tooling](rust-ci-tooling.md) — Rust-specific CI tooling
  (cargo-deny, cargo-audit, cargo-nextest) that runs inside the matrix
