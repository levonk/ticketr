---
type: Practice
title: Per-Platform Devbox Package Entries
description: When a devbox package builds on most platforms but fails on one (e.g., x86_64-darwin / Intel Macs), use the per-platform object form in devbox.json — @latest with excluded_platforms for the working platforms, and a pinned github:nixos/nixpkgs/<channel>#pkg with platforms for the broken one. This keeps devbox functional on ALL platforms without bypassing it or pinning the entire nixpkgs to an older revision.
tags: [devbox, nix, platform, x86_64-darwin, intel-mac, per-platform, nixpkgs, pinning, reproducibility]
date:
  created: "2026-09-02"
  knowledge-basis: "2026-09-02"
  last-used: "2026-09-02"
sources:
  - id: skills-src-devbox-json
    resource: devbox.json
    title: skills-src devbox.json per-platform package entries
  - id: devbox-broken-override
    resource: devbox-broken-override.md
    title: 'Companion concept: devbox broken override (bypass devbox entirely — the fallback when per-platform pinning is not viable)'
---

# Per-Platform Devbox Package Entries

## Failure Mode

A package in `devbox.json` builds cleanly on most platforms but fails on one
— most commonly `x86_64-darwin` (Intel Macs), where a nixpkgs revision may
have dropped or broken a package that still works on `aarch64-darwin` (Apple
Silicon) and Linux. The failure is **package-specific and platform-specific**:
the same `devbox.json` works on other machines, so the configuration is not
wrong, but on the affected platform every `devbox run --` command fails because
devbox cannot build the environment.

### Naive approaches and why they fail

| Approach | Problem |
|----------|---------|
| Pin the entire `nixpkgs.commit` to an older revision | Holds back ALL packages to fix one — every other tool gets stale versions, misses security patches, and accumulates its own breakage |
| Bypass devbox on the broken platform ([Devbox Broken Override](devbox-broken-override.md)) | Loses reproducibility and environment isolation; subagents must use direct package-manager commands, which drift from the pinned versions |
| Remove the broken package from devbox.json | Loses the tool on ALL platforms, not just the broken one |
| Wait for upstream nixpkgs to fix the package | Indeterminate timeline; blocks development on the affected platform indefinitely |

### Root cause

Devbox resolves packages from nixpkgs. When a nixpkgs channel (or the pinned
`nixpkgs.commit`) does not include a buildable version of a package for a
specific architecture, devbox cannot create the environment on that
architecture. The `@latest` specifier pulls from the default nixpkgs channel,
which may not have the package for every platform.

## Practice

Use devbox's **per-platform package entry format**. The `packages` field
accepts an object (not just an array) where each key is a package specifier
and each value is an object with `platforms` or `excluded_platforms` arrays.
For each package that is broken on one platform, add **two entries**:

1. **`pkg@latest`** with `"excluded_platforms": ["x86_64-darwin"]` — gets the
   latest version on all platforms where the package builds cleanly.
2. **`github:nixos/nixpkgs/<channel>#pkg`** with `"platforms": ["x86_64-darwin"]`
   — pins to a specific nixpkgs channel/revision known to build the package
   on the broken platform.

### Configuration

```json
{
  "packages": {
    "difftastic@latest":                              { "excluded_platforms": ["x86_64-darwin"] },
    "github:nixos/nixpkgs/25.05#difftastic":          { "platforms": ["x86_64-darwin"] },
    "just@latest":                                    { "excluded_platforms": ["x86_64-darwin"] },
    "github:nixos/nixpkgs/25.05#just":                { "platforms": ["x86_64-darwin"] },
    "git@latest":                                     { "excluded_platforms": ["x86_64-darwin"] },
    "github:nixos/nixpkgs/25.05#git":                 { "platforms": ["x86_64-darwin"] }
  }
}
```

Packages that build on ALL platforms (no platform-specific breakage) stay
simple — either in the array form or as object entries with empty metadata:

```json
{
  "packages": {
    "github:levonk/nono/feat-nix-package-manager-install#nono": {},
    "github:kunchenguid/treehouse":                          {}
  }
}
```

### How it works

Devbox evaluates each package entry's `platforms` / `excluded_platforms`
against the current machine's architecture (reported by `uname -m` mapped to
Nix platform triples: `aarch64-darwin`, `x86_64-darwin`, `aarch64-linux`,
`x86_64-linux`). Only matching entries are included in the environment build.
This means:

- On `aarch64-darwin` (Apple Silicon): the `@latest` entries are used; the
  pinned `github:nixos/nixpkgs/25.05#...` entries are skipped (not in
  `platforms`).
- On `x86_64-darwin` (Intel Mac): the pinned entries are used; the `@latest`
  entries are skipped (in `excluded_platforms`).
- On Linux: the `@latest` entries are used; the pinned entries are skipped.

The result is that devbox builds successfully on every platform, using the
latest package versions where available and pinned versions where necessary.

### When to use per-platform pinning vs bypass

| Situation | Approach |
|-----------|----------|
| One package broken on one platform, rest of nixpkgs works | **Per-platform pinning** (this concept) — devbox stays functional everywhere |
| Multiple packages broken on one platform, nixpkgs pin is broadly broken | [Devbox Broken Override](devbox-broken-override.md) — bypass devbox on that platform; per-platform pinning becomes unmanageable with too many entries |
| Package broken on ALL platforms | Fix the `devbox.json` / `devbox.lock` — this is a project-level fix, not a platform issue |
| Package broken in CI only | Fix CI environment — pin a known-good nixpkgs revision in the CI-specific config |

### Choosing the pinned channel

The `github:nixos/nixpkgs/<channel>#pkg` specifier pins to a specific nixpkgs
release channel. Choose the channel that:

1. **Has the package for the target platform** — verify by checking the
   nixpkgs repository or `nix-env -qaP -f '<nixpkgs>' --system x86_64-darwin`.
2. **Is recent enough to have security patches** — prefer the latest stable
   release (e.g., `25.05`) over ancient pins.
3. **Is stable** — prefer release channels (`25.05`, `25.11`) over unstable
   branches (`nixpkgs-unstable`) for the pinned entry, since the whole point
   is reliability on the broken platform.

## Concrete Instances

### skills-src devbox.json

The `skills-src` repository uses this pattern for packages that fail to build
on Intel Macs (`x86_64-darwin`). Each broken package gets a pair of entries:
`@latest` excluded from `x86_64-darwin`, and `github:nixos/nixpkgs/25.05#pkg`
pinned for `x86_64-darwin` only. Packages that build on all platforms (e.g.,
`github:kunchenguid/treehouse`) use empty metadata objects.

### Migration from array form to per-platform object form

When migrating an existing `devbox.json` from the array form to the
per-platform object form:

1. **Identify broken packages** — run `devbox run -- echo ok` on the affected
   platform; the error names the missing/broken package(s).
2. **Find a working nixpkgs channel** for each broken package on the affected
   platform (test with `github:nixos/nixpkgs/<channel>#pkg`).
3. **Convert the `packages` field** from array to object form.
4. **Add per-platform entries** for each broken package (the `@latest` +
   pinned pair).
5. **Add empty-metadata entries** for packages that work on all platforms.
6. **Test on all platforms** — `devbox run -- echo ok` on each architecture.

```json
// Before (array form — fails on x86_64-darwin)
{
  "packages": [
    "github:nixos/nixpkgs/25.05#difftastic",
    "github:nixos/nixpkgs/25.05#just"
  ]
}

// After (per-platform object form — works everywhere)
{
  "packages": {
    "difftastic@latest":                              { "excluded_platforms": ["x86_64-darwin"] },
    "github:nixos/nixpkgs/25.05#difftastic":          { "platforms": ["x86_64-darwin"] },
    "just@latest":                                    { "excluded_platforms": ["x86_64-darwin"] },
    "github:nixos/nixpkgs/25.05#just":                { "platforms": ["x86_64-darwin"] }
  }
}
```

## Prevention

1. **Test devbox on all target architectures before merging a `devbox.json`
   change** — CI should run on every architecture the team uses. A package
   that builds on `aarch64-darwin` may fail on `x86_64-darwin`.
2. **Prefer `@latest` over pinned channels** for packages that build on all
   platforms — pinning holds back versions and accumulates stale packages.
   Only pin per-platform when a package is actually broken on a specific
   architecture.
3. **Document which packages are per-platform pinned and why** — a comment in
   `devbox.json` (or a companion doc) explaining "difftastic is pinned to
   25.05 on x86_64-darwin because the default channel dropped it" prevents
   future maintainers from "cleaning up" the duplicate entries and
   reintroducing the breakage.
4. **Re-check pinned channels periodically** — upstream nixpkgs may fix the
   package in a newer channel, at which point the per-platform pin can be
   removed and the package can go back to `@latest` on all platforms.
5. **Keep the bypass override documented as a fallback** —
   [Devbox Broken Override](devbox-broken-override.md) is the escalation path
   when per-platform pinning becomes unmanageable (too many broken packages on
   one platform).

## Related Concepts

- [Devbox Broken Override](devbox-broken-override.md) — the fallback when
  per-platform pinning is not viable (too many packages broken on one
  platform). This concept is the preferred solution; the bypass is the
  escalation.
- [Devbox Over Raw Nix](devbox-over-raw-nix.md) — the devbox.json package
  convention this concept extends with per-platform entries.
- [Multi-Language Devbox](multi-language-devbox.md) — managing multiple
  tool packages in one devbox; per-platform pinning composes with
  multi-language setups.
- [Devbox Rust Versions](devbox-rust-versions.md) — Rust toolchain pinning
  in devbox; per-platform entries may be needed when a Rust version fails
  to build on one architecture.
