---
id: t-9fdbbef
title: Migrate from plain nix to devbox following ADR-20260131001
status: open
deps: []
links: []
created: 2026-02-02T19:25:32.557123780Z
type: task
priority: 2
description: Replace nix develop commands with devbox, add justfile, and implement standard developer UX flow
parent: t-1811dd60---

# Migrate from plain nix to devbox following ADR-20260131001

Replace nix develop commands with devbox, add justfile, and implement standard developer UX flow

## Implementation Requirements

### 1. Update devbox.json
- Add `just` package (already has rustc, cargo, clippy, rustfmt)
- Configure shell init_hook for bootstrap
- Add scripts mapping to muggle targets (not *-internal targets)

### 2. Create justfile (following ADR flows and architecture)

**Normal Targets** (Developer interface, ensure devbox environment):
```just
# Normal targets - Developer interface (REQUIRED)
clean:
    devbox shell clean

build:
    devbox shell build

test:
    devbox shell test

lint:
    devbox shell lint

# Bootstrap recipes (REQUIRED)
bootstrap:
    # Ensure devbox is available and environment is ready
    devbox shell bootstrap

bootstrap-internal:
    cargo build --release
    echo "Rust project bootstrap complete!"

# Health and diagnostics (REQUIRED)
doctor:
    devbox shell doctor

doctor-internal:
    rustc --version
    cargo --version
    echo "Environment healthy!"

# Rust-specific commands
dev:
    devbox shell dev

dev-internal:
    cargo run

run:
    devbox shell run

run-internal:
    cargo run -- $(ARGS)

check:
    devbox shell check

check-internal:
    cargo check

clippy:
    devbox shell clippy

clippy-internal:
    cargo clippy -- -D warnings

fmt:
    devbox shell fmt

fmt-internal:
    cargo fmt

# Clean up
clean-internal:
    cargo clean
    echo "Build artifacts removed"
```

### 3. Update devbox.json scripts
- Add scripts that call just *-internal targets directly
- This allows devbox shell to call internal targets without infinite recursion

### 4. Update .envrc
- Add direnv devbox integration
- Ensure watch_file devbox.json

### 5. Update Makefile (transition)
- Replace nix develop calls with devbox shell calls
- Add migration notes for eventual justfile replacement

### 6. Documentation Updates
- Update README.md with new workflow
- Add devbox setup instructions
- Document just command usage and flows

## Migration Steps
1. Create/Update devbox.json with Rust packages and just
2. Create justfile with normal and *-internal targets
3. Update .envrc for devbox integration
4. Update Makefile to use devbox shell
5. Update documentation
6. Test all workflows work correctly

## Success Criteria
- [ ] `just bootstrap` sets up complete environment
- [ ] `just build` works via devbox shell build-internal
- [ ] `just test` runs tests correctly
- [ ] `just lint` performs linting
- [ ] direnv auto-activates devbox environment
- [ ] All existing Makefile targets still work
- [ ] Documentation updated with new workflow
- [ ] Automated flow works: direnv → devbox → just (*-internal) → cargo
- [ ] Novice flow works: just (normal) → devbox shell → just (*-internal) → cargo
- [ ] Power user flow works: (already in devbox shell) just (directly call *-internal) → cargo
