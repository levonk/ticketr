---
type: Practice
title: Distribution & Packaging
description: Single-binary distributions for CLIs, prebuilt binaries via NPM, and tracking of install/update/offline behavior.
tags: [architecture, distribution, packaging, cli, npm]
date:
  created: "2026-07-18"
  knowledge-basis: "2026-07-18"
  last-used: "2026-07-18"
---

# Distribution & Packaging

- Prefer single-binary or minimal-runtime distributions for CLIs.
- For NPM delivery of native apps, ship prebuilt binaries plus thin wrappers.
- Track compressed and unpacked sizes; optimize with platform splits and stripping.
- Document install paths, update strategy, and offline behavior.

## See Also

- [CDN and DNS](cdn-and-dns.md) — the distribution mechanism for web-delivered
  content at scale.
- [Load Balancing and Reverse Proxy](load-balancing-and-proxy.md) — traffic
  distribution across multiple origin servers.
- [Cost-Aware System Design](cost-aware-system-design.md) — distribution cost
  (CDN egress, data transfer) is a first-class constraint.

## Sources

- Migrated from src/current/rules/software-dev/general/architecture/distribution.md
