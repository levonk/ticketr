---
type: Practice
title: Decentralized P2P Architecture
description: Peer-to-peer network topology where every node is both client and server — subscriber-based replication, CRDT/commutative-monoid state, WASM-sandboxed contracts, censorship-resistance, browser-as-node limitations, and the homelab always-on-node deployment pattern.
tags: [architecture, p2p, decentralized, crdt, commutative-monoid, webassembly, censorship-resistance, replication, freenet]
date:
  created: "2026-07-25"
  knowledge-basis: "2026-07-25"
  last-used: "2026-07-25"
---

# Decentralized P2P Architecture

A **peer-to-peer (P2P) architecture** has no central server. Every participant
runs a **node** that is both a client (consumes state) and a server (holds and
serves state). This is the structural opposite of the client-server /
microservices model covered in
[Application Layer and Microservices](application-layer-microservices.md).

## When P2P is the right fit

- **Censorship-resistance** — there is no single machine to seize, block, or
  subpoena. Taking the system down requires attacking every subscriber.
- **No server budget for the creator** — the cost of hosting moves from the
  creator's wallet to the users' bandwidth/disk/CPU.
- **User-hosted shared state** — chat, forums, shared documents, identity,
  small shared state where the *users* are the natural hosts.
- **Offline-first / local-first** applications that must work without a server
  in the middle.

## When P2P is the wrong fit

- **Large analytics databases** — no SQL, no query engine, no indexes; arbitrary
  row mutations do not compose as a CRDT without hand-building log-structured
  data, which is a research project, not a tutorial.
- **Big query workloads** — keep the giant DB somewhere queryable (a real DB, a
  warehouse, even a static file on IPFS with a separate query layer) and use P2P
  only for the *social* layer (identity, permissions, shared small state).
- **Per-row CRUD at scale** — every subscriber stores the entire contract state,
  so a 500 GB table becomes 500 GB on every user's laptop.

## Subscriber-Based Replication

The defining design choice of modern P2P app platforms (Freenet, Nostr relays,
Scuttlebutt): a piece of shared state **only lives on nodes that subscribed to
it** — not on every node on Earth. This is what "scales without practical limit"
actually means: the claim is about *how many contracts the network can hold*,
not about *how big one contract's state can get*.

This is structurally different from blockchain smart contracts, which are
replicated everywhere. A blockchain contract is a function everyone runs on
every transaction; a P2P contract is a *data structure* with merge rules,
replicated only where it is wanted. The former scales by adding gas; the latter
scales by adding contracts.

## State Model: Commutative Monoid / CRDT

P2P contract state must form a **commutative monoid** — updates can be applied
in any order and still produce the same final state. This is the same property
that makes [CRDTs](https://crdt.tech/) work for multi-region active-active
deployments (see [CAP, Consistency, and Availability](cap-consistency-availability.md)).

Peers exchange:

- **Summary** — a compact representation of what a peer has (hashes, version
  numbers).
- **Delta** — the minimal update needed to bring another peer up to date.

This makes *updates* cheap. It does **not** shrink *storage* cost — every
subscriber still holds the full state. The summaries-and-deltas trick makes
*sync* cheap, not *storage*.

## WASM-Sandboxed Contract Execution

Contracts run as **WebAssembly** in a sandbox on each subscribing peer. The
sandbox gives:

- **Portability** — one bytecode runs on any OS / CPU.
- **Isolation** — untrusted contract code cannot touch the host node's
  filesystem, network, or keys except through the kernel's API.
- **Verifiability** — the same WASM module produces the same state transitions
  everywhere, which is what makes commutative-monoid merging sound.

The node's **kernel** manages keys (via a per-user **delegate**), runs contract
WASM, talks to other peers, and exposes a local API (typically WebSocket on
localhost) that the UI — a normal web page — connects to.

## Browser-as-Node: Why It Almost Works But Doesn't

| Kernel job | Browser can do it? | Notes |
|------------|--------------------|-------|
| Run contract WASM | Yes, natively | Browsers run WASM; trivially portable |
| Hold signing keys / do crypto | Yes, via WebCrypto | Doable |
| Persistent storage | Partial | IndexedDB / OPFS / localStorage work but are smaller and slower than a real filesystem |
| Talk to other peers | **Hard** | Browsers cannot open raw TCP/UDP sockets. WebSockets and WebRTC work but require relay/signaling servers (centralized, defeats the point) or direct browser-to-browser WebRTC (which needs signaling anyway) |

There is also the **always-on problem**: a closed browser tab = your node
offline. A P2P network needs peers to be available; a browser-only peer
disappears every time the user switches tabs or closes the lid. See
[Communication Protocols](communication-protocols.md) for the transport
constraints that drive this.

## Homelab / Always-On-Node Deployment

The sane deployment for any serious P2P user: the kernel runs on an always-on
homelab server (real disk, real network, no tab-closed problem), and the
desktop/phone browser is a thin UI client pointing at the server's local API.
Same shape as running Jellyfin or a home Plex server.

**Benefits you get for free:**

- **Always-on syncing** — the server keeps pulling updates even when the desktop
  is asleep.
- **Better peer presence** — one stable node helps the network more than a
  laptop that comes and goes.
- **Storage on real disks** — no IndexedDB size limits.
- **Multiple devices share one node** — phone, laptop, tablet all hit the same
  kernel.

**Caveats:**

- The delegate's private keys live on the server, not the desktop. Fine for a
  homelab you control; not fine if the server is shared or untrusted.
- Auth/TLS on the exposed API is mandatory — the localhost assumption no longer
  holds. Put a reverse proxy with auth (Caddy/Traefik) in front.
- Multi-user on one kernel is often unclear — one kernel per identity may be
  required.

## Decision Checklist

1. **Is censorship-resistance a hard requirement?** If yes, P2P. If no, a
   client-server deployment is simpler and has better tooling.
2. **Is the shared state small and merge-friendly?** If yes, P2P contracts work.
   If it's a big queryable DB, keep the DB centralized and use P2P only for the
   social/metadata layer.
3. **Can users tolerate installing a node?** If "just give them a URL" is
   mandatory, you need a gateway (a centralized compromise) or a browser-packaged
   kernel (still immature).
4. **Do you need always-on peers?** If yes, plan a homelab/server deployment
   from day one — browser-only peers will not stay online.
5. **Can your state form a commutative monoid?** If not, you have a research
   project, not a deployable app.

## See Also

- [CAP, Consistency, and Availability](cap-consistency-availability.md) — CRDTs
  and commutative monoids are the same consistency tool used for multi-region
  active-active.
- [Communication Protocols](communication-protocols.md) — WebRTC is the only
  browser-native P2P transport; WebSocket is the standard browser-to-local-kernel
  hop.
- [Application Layer and Microservices](application-layer-microservices.md) —
  the client-server alternative this page is the structural opposite of.
- [System Security Basics](system-security-basics.md) — sandboxing untrusted
  contract code and the "no central point to seize" threat model.
- [Business Models Around Open Protocols](business-models-around-open-protocols.md)
  — how to build a profitable service on top of a free, censorship-resistant
  network.

## Sources

- [Freenet tutorial](https://freenet.org/build/manual/tutorial/) — Contract /
  Delegate / UI architecture, subscriber-based replication, commutative monoid
  state model, WASM sandbox, homelab deployment inferences.
- [Freenet reference app: River](https://github.com/freenet/org/tree/main/apps/river)
  — decentralized chat as the canonical sweet-spot workload.
- [CRDT papers and resources](https://crdt.tech/) — the commutative-monoid
  property formalized.
