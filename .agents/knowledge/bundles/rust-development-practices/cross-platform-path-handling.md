---
type: Practice
title: Cross-Platform Path Handling
description: Use camino Utf8PathBuf for String-friendly paths, prefer lexical normalization over canonicalize where possible, handle symlinks and Windows junctions, strip the \\?\ UNC prefix, and display paths lossy in errors.
tags: [rust, paths, camino, cross-platform, symlinks, unc, windows]
date:
  created: "2026-08-05"
  knowledge-basis: "2026-08-05"
  last-used: "2026-08-05"

sources:
  - id: project-lint-audit
    resource: "project-lint/Cargo.toml"
    title: "project-lint"
---

# Cross-Platform Path Handling

## Failure Mode

`PathBuf` forces `to_str()` unwraps at every boundary, panicking on non-UTF-8
names. `canonicalize` resolves symlinks and fails on missing paths, breaking
watchers. Windows junctions and `\\?\` UNC prefixes leak into error messages
that users cannot paste back. `to_string_lossy()` in logs hides the real bytes
when debugging encoding bugs.

## Practice

### std::path vs camino

Use `camino::Utf8PathBuf` for application paths. It guarantees UTF-8 at the
type level, so you pass `&str` into APIs without unwrap cascades. Keep
`std::path::Path` at FFI and OS boundaries where bytes may not be UTF-8.

```rust
use camino::Utf8PathBuf;

fn load_config(root: &Utf8PathBuf) -> Config {
    let path = root.join(".project-lint").join("config.toml");
    Config::load(&path)
}
```

> Note: camino is not yet in project-lint's `Cargo.toml`. Add it as a workspace
> dependency before adopting.

### Path Normalization

Prefer lexical normalization (`..` and `.` removal) over `canonicalize`.
Lexical normalization does not touch the filesystem, so it works on paths that
do not yet exist and does not resolve symlinks unexpectedly.

```rust
fn normalize(p: &Utf8PathBuf) -> Utf8PathBuf {
    let mut out = Vec::new();
    for comp in p.components() {
        match comp {
            camino::Utf8Component::CurDir => {}
            camino::Utf8Component::ParentDir => { out.pop(); }
            c => out.push(c.as_str()),
        }
    }
    Utf8PathBuf::from_iter(out)
}
```

Reach for `canonicalize` only when you need the real on-disk identity — for
deduplication after symlink resolution, for example.

### Symlinks and Windows Junctions

On Unix, symlinks are transparent to `read_link`. On Windows, junctions are not
symlinks; `std::fs::symlink_metadata` distinguishes them. When walking, decide
explicitly whether to follow: lint tools usually do not cross project boundaries,
so treat junctions as out-of-scope unless configured.

### Windows UNC Paths

Long paths on Windows arrive with the `\\?\` prefix. Strip it before displaying
to users so error messages match what they typed.

```rust
fn display_path(p: &Utf8PathBuf) -> String {
    let s = p.as_str();
    s.strip_prefix(r"\\?\").unwrap_or(s).to_string()
}
```

### Path Display in Errors

Use `display()` for human-facing messages. Reserve `to_string_lossy()` for
non-UTF-8 fallbacks and never as the default — it replaces invalid bytes with
`U+FFFD`, hiding the real name. When logging for debugging, emit the raw bytes
as a separate field.

```rust
return Err(anyhow!("config not found at {}", path.display()));
```

## Related Concepts

- [Error Handling](error-handling.md) — paths in error context
- [File Watcher Patterns](file-watcher-patterns.md) — normalizing event paths
- [Project Structure](project-structure.md) — path module placement
