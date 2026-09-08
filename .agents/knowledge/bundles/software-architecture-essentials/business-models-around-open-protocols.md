---
type: Practice
title: Business Models Around Open Protocols
description: How to build a profitable service on top of a free, open, censorship-resistant protocol — managed nodes, premium clients, curation, identity services, B2B infrastructure, consulting, and seedboxing — and why per-transaction tolls and middleman arbitrage fail on open networks.
tags: [architecture, business-model, open-protocol, freenet, managed-service, curation, identity, consulting, cost]
date:
  created: "2026-07-25"
  knowledge-basis: "2026-07-25"
  last-used: "2026-07-25"
---

# Business Models Around Open Protocols

When the underlying protocol is **free, open, and censorship-resistant**
(Freenet, Nostr, IPFS, ActivityPub/Mastodon, email, Matrix), you cannot charge
for "access to the network" — there is no native payment rail, no gas fees, no
way to enforce a toll. The businesses that work sell **convenience, expertise,
or trust on top of the open protocol**, the same pattern as managed Postgres,
email hosting, or managed Nostr relays.

This is the business-model counterpart to
[Cost-Aware System Design](cost-aware-system-design.md): cost-awareness covers
how you *spend*; this page covers how you *earn* when the substrate is free.

## Models That Plausibly Work

| Model | What you sell | Revenue shape | Analogy |
|-------|---------------|---------------|---------|
| **Managed node service (SaaS)** | "We run an always-online, backed-up node for you" | Recurring subscription | Pinata for IPFS, managed Nostr relays, managed Postgres |
| **Premium client / UI** | A polished client with extra features (search, notifications, mobile sync, themes) | Free tier + paid tier | Mastodon clients, paid email clients |
| **Curation and discovery** | "We find the good stuff for you" — directories, search indexes, recommendations over public contract data | Subscription or ads | Podcast directories, Substack discovery |
| **Identity / verification services** | Verified identity badges, anti-spam reputation, business verification bridging to the identity layer | Per-verification or subscription | "Blue checks" for P2P identities |
| **B2B censorship-resistant infrastructure** | Whistleblower portals, journalist drop sites, legal evidence archives, activist publishing — sold to organizations that need untakedownable publishing | Integration + SLAs + custom contract dev | Managed Mattermost for sensitive orgs |
| **Custom contract development (consulting)** | Companies want a P2P app but lack the Rust/WASM/CRDT expertise | Project-based or retainer | Any specialized dev consultancy |
| **Storage/bandwidth subsidies ("seedboxing")** | Subscribe to users' contracts and keep them alive, fast, replicated 24/7 | Per-contract or per-GB subscription | Pinata pinning, BitTorrent seedboxes |

## Models That Probably Do Not Work

- **Per-transaction fees on the network itself** — no native payment rail, no
  way to enforce a toll. The protocol is open; anyone can replicate your
  service.
- **Pure middleman arbitrage** — the network is open; anyone can replicate your
  service for free.
- **Ads inside contracts** — the contract runs on *other people's* machines; you
  cannot force them to render your ads.

## Why Open Protocols Invert the Rent-Seeking Logic

Censorship-resistance is a **feature for users** but a **headwind for
rent-seeking**. You build a business *around* the protocol, not *on top of* it
as a tollbooth. The protocol's openness is precisely what makes toll-based
models fail — anyone can bypass the toll by running the open code themselves.
The defensibility comes from convenience, trust, or expertise that the open
protocol does not provide by default.

This is the same dynamic that shaped the email ecosystem: SMTP is free and open,
but Gmail, Fastmail, and ProtonMail are profitable because they sell a better
*experience* and *trust* (spam filtering, reliability, support), not access to
SMTP.

## Relationship to Cost-Aware Design

A managed-node service is essentially a **flat-rate or per-seat contract** (see
[Cost-Aware System Design](cost-aware-system-design.md)) wrapped around a
protocol whose underlying marginal cost is borne by the network's peers, not by
you. Your cost is the always-on infrastructure; your revenue is the per-seat
subscription. The break-even math is the same as any per-seat SaaS — your
per-user infrastructure cost must stay below your per-seat price.

## Decision Checklist

1. **Is the underlying protocol open and free to replicate?** If yes, toll-based
   models will fail — pick a convenience/trust/expertise model instead.
2. **Can you offer something the open protocol does not provide by default?**
   (Always-on, backups, polished UI, curation, identity, support.) If yes, that
   is your product.
3. **Is your customer an end-user (premium client) or an organization (B2B
   infra/consulting)?** The sales motion and pricing are very different.
4. **Can your per-user infrastructure cost stay below your per-seat price?** If
   not, the managed-node model loses money at scale.
5. **Are you depending on a feature the protocol could remove?** Open protocols
   evolve; build on stable surfaces and document your assumptions.

## See Also

- [Cost-Aware System Design](cost-aware-system-design.md) — the cost side of the
  same business equation; per-seat vs per-call break-even math applies directly.
- [Decentralized P2P Architecture](decentralized-p2p-architecture.md) — the
  architecture these business models wrap around.
- [Tech Decision Risk Assessment](tech-decision-risk-assessment.md) — choosing a
  business model is a technology decision with end-user impact (highest risk
  tier).

## Sources

- [Freenet tutorial](https://freenet.org/build/manual/tutorial/) — the open,
  censorship-resistant protocol that motivated this analysis.
- Email / SMTP ecosystem — the canonical historical example of profitable
  services on a free open protocol.
- Pinata (IPFS pinning), managed Nostr relays, Mastodon client ecosystem —
  contemporary analogues.
