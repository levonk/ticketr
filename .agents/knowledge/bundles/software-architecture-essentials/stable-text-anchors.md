---
type: Practice
title: Stable Text Anchors for Plan Insertion
description: When an automated tool or agent must insert content at a specific point in a file, use stable text anchors — a unique, descriptive snippet of existing content — never raw line numbers. Line numbers drift on every preceding edit; stable text anchors survive edits anywhere except at the anchor itself. The anchor is a semantic marker, not a positional coordinate.
tags: [architecture, text-anchors, plan-insertion, automated-editing, agent-runtime, file-editing, stability]
date:
  created: "2026-08-10"
  knowledge-basis: "2026-08-10"
  last-used: "2026-08-10"
---

# Stable Text Anchors for Plan Insertion

## The General Rule

When an automated tool, code generator, or AI agent needs to insert content
at a specific point in an existing file, it must identify the insertion
point by a **stable text anchor** — a unique, descriptive snippet of the
existing content — not by a raw line number. Line numbers are positional
coordinates that drift on every edit above the insertion point; text anchors
are semantic markers that survive any edit that does not modify the anchor
text itself.

- **Use stable text anchors (e.g., "after the `it('throws on ...')` test
  block"), never raw line numbers.** The anchor is a snippet of existing
  content that uniquely identifies the insertion point. The tool searches
  for the anchor and inserts relative to it.
- **Line numbers drift on every preceding edit.** If a plan says "insert at
  line 42" and a previous step added 10 lines above line 42, the insertion
  now goes to line 52 — the wrong place. A text anchor is unaffected by
  edits above it.
- **The anchor must be unique within the file.** If the anchor text appears
  multiple times, the tool cannot determine which occurrence is the
  insertion point. Choose a snippet that is distinctive enough to match
  exactly once.
- **The anchor must be stable.** Choose a snippet that is unlikely to change
  in the near term — a function signature, a test description, a comment.
  Avoid anchors that are part of a frequently edited region.

## Why It Matters

Automated tools that edit files — AI agents, code generators, refactoring
scripts — often work in multiple steps. Step 1 edits one part of the file;
step 2 edits another part. If step 2's insertion point was specified as a
line number, step 1's edit may have shifted the line number, and step 2
inserts in the wrong place.

This is not a rare edge case. It is the default outcome of multi-step file
editing. Every edit above the insertion point shifts the line number. The
more steps in the plan, the more likely the drift.

Text anchors solve this because they are content-relative, not
position-relative. An anchor at "after the `it('throws on invalid input')`
test block" is unaffected by edits anywhere else in the file — the anchor
text is still there, and the tool finds it.

## How To Choose A Good Anchor

A good anchor is:

1. **Unique.** The anchor text appears exactly once in the file. If it
   appears multiple times, add more context to the anchor until it is
   unique.
2. **Descriptive.** The anchor describes what is at that location, not just
   the raw text. "after the `it('throws on ...')` test block" is better than
   "after `});`" — the latter appears dozens of times.
3. **Stable.** The anchor is in a region that is unlikely to change between
   the plan's creation and its execution. A function signature or a test
   description is stable; a line inside a frequently edited function is not.
4. **Structural.** The anchor identifies a structural element (a function, a
   class, a test block, a config section) rather than an arbitrary line.
   Structural elements are more stable than arbitrary lines because they
   have semantic meaning — a developer is less likely to delete a test
   description than to reformat the lines around it.

## Anchor Formats

| Format | Example | When to use |
|--------|---------|-------------|
| **After a named element** | "after the `it('throws on invalid input')` test block" | Test files, named functions |
| **Before a named element** | "before the `export function main()` declaration" | Module entry points |
| **Inside a named block** | "inside the `configure()` function, after the `loadConfig()` call" | Function bodies |
| **After a comment marker** | "after the `// --- handlers ---` comment" | Section-delimited files |
| **At end of file** | "at the end of the file, after the last export" | Appending new content |

Avoid:

| Bad anchor | Why it fails |
|------------|--------------|
| "line 42" | Drifts on any edit above |
| "after `});`" | Not unique — appears many times |
| "after the third function" | Ordinal positions drift if functions are added/removed |
| "at the cursor position" | The cursor is not a file property |

## Anti-Patterns

- **Raw line numbers in a plan.** "Insert at line 87." The plan has 5 steps;
  step 2 added 15 lines above line 87; step 5 inserts at line 102 — the
  wrong place.
- **Non-unique anchors.** "After the `return` statement." There are 20
  return statements in the file. The tool picks the first one, which is
  wrong.
- **Anchors in volatile regions.** "After the line that says `const result =
  await fetch(url)`." The fetch call is in a function being actively
  refactored; the anchor text may change before the plan executes.
- **Ordinal positions.** "After the third test block." If a test is added or
  removed, the third test is now a different test.

## Concrete Instances

- **Archon (Bun + TypeScript).** The Development Guidelines state: "Plan
  insertion points: Use stable text anchors (e.g., 'after the `it('throws
  on ...')` test block'), never raw line numbers — line numbers drift on
  every preceding edit." This rule is binding for any automated tool or
  agent that edits files in the archon codebase.
- **GitHub Copilot Workspace.** When proposing edits, Copilot Workspace
  identifies insertion points by surrounding code context (a function
  signature, a comment), not by line number. The plan survives edits above
  the insertion point because the context anchor is content-relative.
- **ast-grep / tree-sitter edit APIs.** Code transformation tools that
  operate on ASTs use node identities (a named function, a named class) as
  insertion anchors — not line numbers. The AST node is the stable anchor;
  line numbers are a rendering detail. This is the structural extreme of the
  pattern: the anchor is not text but a semantic node in the tree.

## See Also

- [KISS Principle](kiss-principle.md) — a text anchor is simpler than a
  line-number-tracking system; the simplest stable reference wins.
- [Fail Fast + Explicit Errors](fail-fast-explicit-errors.md) — if the
  anchor is not found, fail with a clear error ("anchor 'after the
  `it('throws on ...')` test block' not found in file") rather than
  inserting at a guessed position.
- [Root-Cause First](root-cause-first.md) — line-number drift is a symptom;
  the root cause is positional referencing in a mutable file. Text anchors
  fix the root cause.
