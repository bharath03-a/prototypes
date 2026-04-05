# Exercise — Minimal Raft (single-process simulation)

## Goal

Build a **toy Raft** core in Rust: enough structure to simulate leader election and log append in one process (no real network). Add networking in a follow-up if you want.

## Suggested phases

1. **Types** — `Term`, `NodeId`, `LogEntry`, follower/candidate/leader roles, volatile state (commit index, last applied).
2. **Elections** — timeouts, `RequestVote` / `RequestVoteResponse`, majority quorum.
3. **Replication** — `AppendEntries` heartbeat + log match; advance `commit_index` when a majority has replicated.
4. **Tests** — 3–5 in-memory nodes, deterministic RNG for timeouts, assert single leader per term and monotonic commits.

Skip for v1: snapshots, dynamic membership, persistent disk log.
