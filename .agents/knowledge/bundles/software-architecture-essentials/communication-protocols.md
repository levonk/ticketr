---
type: Practice
title: Communication Protocols
description: Choose between HTTP, TCP/UDP, RPC (gRPC), REST, GraphQL, WebSockets, SSE, WebTransport, WebRTC, and HTTP/3 (QUIC) for inter-service and client-server communication — including the application-vs-transport layering, browser support matrix, NAT traversal, and a P2P branch for decentralized apps.
tags: [architecture, communication, http, tcp, udp, rpc, grpc, rest, graphql, websockets, sse, webtransport, webrtc, http3, quic, p2p]
date:
  created: "2026-07-24"
  knowledge-basis: "2026-07-25"
  last-used: "2026-07-25"
---

# Communication Protocols

## Layering Note

These protocols are **not all at the same layer**. Comparing HTTP to QUIC is
like comparing a letter to a postal truck — they sit at different levels of the
stack. The choice of transport (TCP vs QUIC) drives many of the
application-layer differences.

| Layer | Protocols |
|-------|-----------|
| **Application** (browser APIs, request/response semantics) | HTTP/1.1, HTTP/2, HTTP/3, WebSocket, WebTransport, WebRTC, SSE |
| **Transport** (how bytes actually move) | TCP, UDP, QUIC |

HTTP/3 *is* HTTP semantics over QUIC. WebSocket *is* HTTP upgraded to a
persistent TCP stream. WebTransport *is* the browser API for QUIC
streams/datagrams. The layering matters when reading the matrix below.

## TCP vs UDP

| | TCP | UDP |
|---|-----|-----|
| Reliability | Ordered, guaranteed, checksum | Best-effort, may reorder/lose |
| Overhead | Higher (handshake, flow/congestion control) | Lower |
| Use case | Web, DB, file transfer, SSH | Streaming, VoIP, games, DNS, QUIC |

## HTTP / REST

Use REST for public APIs, browser clients, and cacheable resources.

| Verb | Idempotent | Safe | Typical use |
|------|------------|------|-------------|
| GET | Yes | Yes | Read |
| POST | No | No | Create / trigger |
| PUT | Yes | No | Replace |
| PATCH | No | No | Partial update |
| DELETE | Yes | No | Delete |

REST tradeoffs: simple and cacheable, but nested resources need multiple round
trips and payloads bloat over time.

## RPC / gRPC

Use gRPC (HTTP/2 + Protobuf) for internal, performance-sensitive,
strongly-typed service-to-service calls.

- Benefits: streaming, code generation, low latency, multiplexing.
- Tradeoffs: tighter coupling, harder caching, requires stubs.

## GraphQL

Use when clients need flexible, nested views and the data lives across multiple
services.

- Benefits: client-specified fields, fewer round trips.
- Tradeoffs: server complexity, N+1 query risk, caching is harder.

## WebSockets

Use for bidirectional real-time push: chat, collaboration, live dashboards.
Mature (2011), universal browser support, true full-duplex over TCP. The
default choice when both ends need to send messages anytime.

## Server-Sent Events (SSE)

Use for **one-way** server-to-client streaming: notifications, stock tickers,
live logs, dashboards. Simplest possible API (`EventSource`), automatic
reconnection built in, works over standard HTTP so it passes every firewall
and proxy. Use this *before* reaching for WebSocket if you only need one-way
push.

## HTTP/3 (QUIC)

Use for public internet where connection migration and reduced head-of-line
blocking matter. Requires infrastructure support (LBs, proxies must understand
QUIC). 1-RTT or 0-RTT setup, per-stream head-of-line blocking elimination,
connection migration across network changes (WiFi → cellular).

## WebTransport

Use for low-latency real-time on unstable/mobile networks: competitive gaming,
live trading, remote desktop. Built on QUIC, so it inherits 0-RTT reconnect,
no head-of-line blocking, datagrams for unreliable low-latency sends, and
connection migration. **Caveat**: browser support is still partial (Chrome yes,
Firefox partial, Safari no as of 2025) — fall back to WebSocket.

## WebRTC

Use for **browser-to-browser P2P**: video calls, file transfer, decentralized
apps. The *only* browser-native way to send data directly between two browsers
without a server in the middle. Built-in NAT traversal (STUN/TURN), mandatory
encryption (DTLS), data channels for arbitrary bytes. The natural transport for
[Decentralized P2P Architecture](decentralized-p2p-architecture.md) on the
browser hop. **Caveat**: setup is heavy (ICE + DTLS, seconds), and signaling
still needs an out-of-band channel (often a WebSocket to a signaling server).

## Browser Support Matrix

| Protocol | Native browser API | Notes |
|----------|-------------------|-------|
| HTTP/1.1, HTTP/2, HTTP/3 | `fetch` / XHR (transparent) | HTTP/3 picked automatically when both ends support it |
| WebSocket | `new WebSocket(...)` | Universal since 2011-2012 |
| SSE | `EventSource` | Universal, one-way only |
| WebTransport | `WebTransport` | Chrome yes, Firefox partial, Safari no (2025) |
| WebRTC | `RTCPeerConnection` | Universal, but heavy setup |
| QUIC | (only via HTTP/3 / WebTransport) | No direct browser API |
| TCP / UDP | ❌ no browser API | Browsers cannot open raw sockets |

## NAT Traversal

A first-class concern for any browser-to-X communication that is not plain HTTP
over 80/443:

- **HTTP / WebSocket / SSE**: pass every firewall and proxy (look like HTTP).
- **WebTransport / QUIC**: UDP 443 — some networks block UDP, fall back to
  WebSocket.
- **WebRTC**: needs STUN (discovery) and TURN (relay fallback) servers; the
  only browser-native option for true P2P across NATs.

## Decision Checklist

1. **Public API or internal service?** Public → REST or GraphQL. Internal →
   gRPC.
2. **Real-time?** Bidirectional → WebSocket or WebTransport. One-way push → SSE.
   Browser-to-browser → WebRTC.
3. **P2P / decentralized?** Browser-to-browser data → WebRTC. Browser-to-local-
   kernel → WebSocket (see
   [Decentralized P2P Architecture](decentralized-p2p-architecture.md)).
4. **Reliability required?** Yes → TCP. Lowest latency, late > lost → UDP/QUIC.
5. **Cacheable?** REST is cacheable by default; gRPC and GraphQL are not.
6. **Nested/complex queries?** GraphQL or a BFF pattern over REST.
7. **Mobile / unstable networks?** WebTransport (QUIC) for connection migration;
   fall back to WebSocket where unsupported.
8. **Firewall / NAT traversal critical?** Prefer HTTP-shaped traffic (SSE,
   WebSocket) over UDP-based (WebTransport, WebRTC) when traversing restrictive
   networks; use STUN/TURN for WebRTC.

## See Also

- [Load Balancing and Reverse Proxy](load-balancing-and-proxy.md) — L7 LBs
  route by HTTP; QUIC needs connection-ID routing.
- [System Security Basics](system-security-basics.md) — TLS for REST/HTTP,
  mTLS for gRPC, DTLS for WebRTC.
- [Asynchronism and Queues](asynchronism-and-queues.md) — message queues are
  another communication pattern.
- [Decentralized P2P Architecture](decentralized-p2p-architecture.md) — WebRTC
  is the only browser-native P2P transport; WebSocket is the standard
  browser-to-local-kernel hop.

## Sources

- [The System Design Primer](https://github.com/donnemartin/system-design-primer)
  — Communication (HTTP, TCP, UDP, RPC, REST).
- [RFC 6455 (WebSocket)](https://datatracker.ietf.org/doc/html/rfc6455),
  [RFC 9000 (QUIC)](https://datatracker.ietf.org/doc/html/rfc9000),
  [WebTransport](https://www.w3.org/groups/wg/webtransport/),
  [WebRTC](https://www.w3.org/TR/webrtc/),
  [SSE (WHATWG)](https://html.spec.whatwg.org/multipage/server-sent-events.html)
  — protocol specifications and browser API references, ingested 2026-07-25.
