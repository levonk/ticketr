---
type: Practice
title: Database Scaling
description: Scale relational databases with replication, federation, sharding, denormalization, and SQL tuning; choose between SQL, NoSQL, and distributed SQL; and pick the right specialized store for vector, time-series, and lakehouse workloads.
tags: [architecture, database, rdbms, nosql, replication, sharding, federation, denormalization, distributed-sql, vector-db, lakehouse]
date:
  created: "2026-07-24"
  knowledge-basis: "2026-07-24"
  last-used: "2026-07-24"
---

# Database Scaling

## SQL (RDBMS)

Good for structured data, complex queries, and ACID transactions.

### Scaling techniques

| Technique | How it works | When to use |
|-----------|--------------|-------------|
| **Primary-replica replication** | Writes to primary, replicas serve reads | Read-heavy, can tolerate replication lag |
| **Multi-primary replication** | Multiple writable nodes | Write-heavy, conflict resolution is acceptable |
| **Federation** | Split DBs by function (users, forums, products) | Clear functional boundaries, no huge cross-DB joins |
| **Sharding** | Distribute data across multiple DBs | Outgrown single node, shard key is well-distributed |
| **Denormalization** | Redundant copies to avoid joins | Read-heavy, write volume is manageable |
| **SQL tuning** | Indices, schema tightening, query optimization | Before adding hardware |

### Decision checklist for SQL scaling

1. Is the read load the bottleneck? Add read replicas.
2. Is the write load the bottleneck? Shard, federate, or move to distributed SQL.
3. Do cross-shard joins dominate? Federation or sharding may be wrong —
   consider denormalization or a different model.
4. Is the shard key skewed? Hot shards are worse than one big node. Use
   consistent hashing or re-shard.

## NoSQL

| Type | Model | Best for | Examples |
|------|-------|----------|----------|
| **Key-value** | Hash table | Cache, sessions, simple lookups | Redis, DynamoDB |
| **Document** | JSON/BSON documents | Flexible schemas, nested data | MongoDB, CouchDB |
| **Wide-column** | 2D map (row + column) | Time-series, high writes, sparse data | Cassandra, Bigtable |
| **Graph** | Nodes, edges, properties | Relationships, recommendations | Neo4j, Dgraph |

NoSQL favors BASE (Basically Available, Soft state, Eventual consistency) over
ACID. Joins are done in application code.

## SQL or NoSQL?

Use both. Most systems keep the transactional core in SQL and offload specific
workloads to specialized stores:

- Cache/session → key-value.
- Metrics/events → wide-column / time-series.
- Recommendations → graph.
- LLM embeddings → vector.
- Analytics → lakehouse.

## Modern Database Categories

- **Distributed SQL** (Spanner, CockroachDB, YugabyteDB) — ACID across regions
  with Raft/Paxos. Use when you need horizontal scale + strong consistency.
- **Vector databases** (pgvector, Pinecone, Weaviate, Milvus) — similarity
  search for embeddings. Use for RAG, semantic search, recommendations.
- **Lakehouse** (Iceberg, Delta Lake, Hudi) — ACID on object storage with
  schema evolution. Use for analytics replacing lake + warehouse.
- **Time-series databases** (TimescaleDB, InfluxDB, Prometheus) — optimized for
  timestamped, append-mostly, high-cardinality data.

## Decision Checklist

1. Do you need ACID transactions across records? Yes → SQL or distributed SQL.
2. Is the schema stable or rapidly evolving? Stable → SQL. Evolving → document.
3. Is the workload high-write, time-ordered, or sparse? Yes → wide-column / time-series.
4. Are relationships the primary query pattern? Yes → graph.
5. Is cost or scale the blocker? Distributed SQL and cloud-managed NoSQL can
   defer sharding complexity.

## See Also

- [CAP, Consistency, and Availability](cap-consistency-availability.md) — the
  consistency/availability tradeoff.
- [Data Access Layer](data-access-layer.md) — centralize access to multiple
  stores.
- [Caching Strategies](caching-strategies.md) — absorb read spikes.

## Sources

- [The System Design Primer](https://github.com/donnemartin/system-design-primer)
  — Database (RDBMS, NoSQL, replication, federation, sharding, denormalization,
  SQL tuning).
