# 02_raft_cluster — Multi-node Raft (in-process simulation)

## Prerequisite

Finish `**01_raft_single**` first. You should be comfortable with **term**, **roles** (Follower / Candidate / Leader), `**voted_for`**, the **log** with per-entry **term**, and **commit_index**.

This exercise keeps **no real network**: all RPCs are **Rust values** delivered by a small **simulator** (your code) that calls into each node.

---

## Goal

Run **several Raft nodes in one process** with:

1. **Leader election** that requires a **majority** of votes (not instant win).
2. **Log replication** from leader to followers via **AppendEntries**-style messages.
3. **Commit advance** on the leader only when a **majority** of nodes have replicated an entry (match index / leader commit semantics).

You are **not** required to implement persistence to disk, snapshots, or membership changes.

---

## Why in-process?

Real Raft adds TCP/gRPC, serialization, and async timing. Here you isolate **the algorithm**: message types, state transitions, and quorums—without I/O noise.

---

## Suggested shape (you may adapt names)

### Messages (RPCs as data)

Model at least:

`**RequestVote`**

- `term`, `candidate_id`
- `last_log_index`, `last_log_term` (for the “up-to-date log” check)

`**RequestVoteResponse**`

- `term`, `vote_granted`

`**AppendEntries**` (heartbeats may be **empty** `entries`)

- `term`, `leader_id`
- `prev_log_index`, `prev_log_term`
- `entries: Vec<LogEntry>` (can be empty for heartbeat)
- `leader_commit: usize` (highest index the leader has committed)

`**AppendEntriesResponse`**

- `term`, `success`

Use the same `**LogEntry { term, command }**` idea as `01_raft_single`, or align field types with your single-node code.

### Node

Each node holds the usual Raft volatile / log fields (at minimum):

- `id`, `current_term`, `voted_for`, `log`, `commit_index`, `role`
- **Leader-only** (optional in v1, but needed for replication): `next_index`, `match_index` per peer—or an equivalent you document.

### Cluster / simulator

Something that:

- Owns `**Vec<Node>`** (or `HashMap<u64, Node>`).
- **Routes** a message from `from` → `to` by calling the target’s handler.
- Lets you **inject** “election timeout fired on node `i`” or run a `**step()`** loop that processes pending RPCs in a deterministic order.

You do **not** need threads; a single-threaded event loop is enough.

---

## Rules to implement (checklist)

**Elections**

- On timeout, increment **term**, become **Candidate**, **vote for self**, send `**RequestVote`** to all others (simulator delivers them).
- Grant vote only if: `**voted_for` is `None` or same candidate**, **candidate’s log is at least as up-to-date** as yours (compare **last term**, then **length**), and `**RequestVote.term >= current_term`** (and update term if you see a newer one).
- If votes **≥ majority** → **Leader**; on becoming leader, initialize replication state (`next_index` / `match_index`) sensibly (e.g. `next_index = len+1` if you use 1-based log indices—**pick one indexing scheme and stick to it**).
- If `**AppendEntries` or `RequestVote` shows a higher `term`** → step down to **Follower**, update term, clear `voted_for` as in `01`.

**Replication**

- Leader appends **client** commands to its log (you can expose `client_write` only on leader or only through the cluster API).
- For each follower, send `**AppendEntries`** that preserves **log consistency**: if `**prev_log_*`** does not match, follower rejects; leader decrements `next_index` and retries (standard Raft).
- On successful append, update `**match_index**`, then compute **new `commit_index`** as the **highest index** such that a **majority** of `match_index >= N` and the entry at `N` has `**term == current_term`** (leader never commits a prior term’s entry by count alone).

**Commit on followers**

- When `**AppendEntries.leader_commit`** is larger, follower sets `**commit_index = min(leader_commit, last_log_index)**` (and applies entries if you model an application state; optional for this exercise if you only check log + commit index).

---

## Scenarios to verify

Work in order; automate with `**#[cfg(test)]**` when you can.

### 1 — Cold start, pick a leader

- Cluster size **3** or **5**, all start as followers (term 0).
- Trigger election on **one** node (or run until one wins).
- **Exactly one** **Leader** in the latest **term**; others **Follower**; **terms** consistent.

### 2 — Client write replicates

- With a stable leader, `**client_write("a")`** then assert **majority** of nodes have the entry in their **log** at the **same index** with the **same term**.

### 3 — Commit needs majority

- `**commit_index`** on the leader does **not** advance past an index until **enough** followers have acknowledged it (your test can inspect `**match_index`** or follower logs).

### 4 — Higher term steps down

- Leader in term **T**. Deliver `**AppendEntries`** or `**RequestVote**` with term **T+1** from another node (simulated). Old leader becomes **Follower**, adopts **T+1**.

### 5 — Log conflict

- Manually diverge two logs (same index, **different term**). New leader’s `**AppendEntries`** should **overwrite** follower suffix and converge logs.

*(Optional hard mode: split votes, unreliable delivery, randomized message order—only after the above pass.)*

---

## Constraints

- **Standard library only** (same as `01_raft_single`).
- **No panics** on normal “reject” paths; use `**Result`** or boolean RPC results.
- **Terms never decrease** on a node.
- Document your **log indexing** (0-based vs 1-based); tests and RPC fields must match.

---

## Files

- `src/main.rs` — implementation and/or a tiny demo runner
- `PROBLEM.md` — this spec

## Reference

- Raft paper: [https://raft.github.io/raft.pdf](https://raft.github.io/raft.pdf)  
- Visual guide: [https://raft.github.io/](https://raft.github.io/)

---

## Next (after this works)

- Persistent log + **WAL** (`storage/` prototypes)
- Real **transport** (TCP) and fuzz/property tests
- **Membership changes** (joint consensus)—advanced

