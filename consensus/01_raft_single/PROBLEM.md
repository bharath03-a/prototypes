# 01_raft_single — Single-node Raft state machine

## Goal

Implement the Raft state machine for a single node.
No cluster, no voting, no networking — just one node going through
all the state transitions correctly.

This is the foundation. Once this is solid, `02_raft_cluster` adds multiple nodes.

---

## Background

A Raft node is fundamentally a state machine with three states.
Understanding how one node transitions between them — and why —
is the prerequisite to understanding the whole algorithm.

---

## The state machine

A node is always in one of three roles:

```
Follower → (timeout)          → Candidate → (wins election)  → Leader
                                           → (loses election) → Follower
Leader   → (sees higher term) → Follower
```

### Follower

- Default starting state
- Accepts log entries from a leader via AppendEntries
- Resets its election timer when it hears from the leader
- If the timer expires with no heartbeat → becomes Candidate

### Candidate

- Increments its own term
- Votes for itself
- In a single node cluster — it is the only voter, so it immediately wins
- Becomes Leader

### Leader

- Accepts writes (client commands)
- Appends entries to its own log
- Commits entries immediately (no majority needed in single node)
- Sends periodic heartbeats (simulated via function call)
- Steps down to Follower if it ever sees a higher term

---

## The interface to implement

You need to define the following types and methods yourself — no starter code.

### Types

- `Role` enum with three variants: `Follower`, `Candidate`, `Leader`
- `LogEntry` struct with a `term: u64` and a `command: String`
- `Node` struct with the fields listed below

### Node fields

| Field          | Type            | Description                                              |
| -------------- | --------------- | -------------------------------------------------------- |
| `id`           | `u64`           | unique node identifier                                   |
| `current_term` | `u64`           | monotonically increasing election cycle number           |
| `voted_for`    | `Option<u64>`   | which node id this node voted for in current term        |
| `log`          | `Vec<LogEntry>` | ordered list of all log entries                          |
| `commit_index` | `usize`         | index of highest committed entry (0 = nothing committed) |
| `role`         | `Role`          | current state of this node                               |

### Methods

`new(id: u64) -> Node`
Create a new node. Always starts as Follower in term 0 with an empty log.

`trigger_election(&mut self)`
Election timer expired. Transition Follower → Candidate → Leader.
In single node, the candidate is the only voter so it wins immediately.

`client_write(&mut self, command: &str) -> Result<usize, &str>`
Leader only: append a command to the log, commit it, return the log index.
Return Err if this node is not the leader.

`receive_heartbeat(&mut self, leader_term: u64, leader_id: u64) -> bool`
Simulate receiving a heartbeat from an external leader.
If their term >= our term: step down to Follower, update term, return true.
If their term < our term: reject, return false.

`committed_log(&self) -> &[LogEntry]`
Return only the committed entries — everything up to and including commit_index.

---

## Scenarios to verify

Work through these in order. Each one should pass before moving to the next.

### Scenario 1 — Fresh node

```
node = Node::new(1)
assert role       == Follower
assert term       == 0
assert log        == empty
assert commit_index == 0
```

### Scenario 2 — Election timeout

```
node.trigger_election()
assert role       == Leader
assert term       == 1        // incremented during election
assert voted_for  == Some(1)  // voted for itself
```

### Scenario 3 — Write as leader

```
node.client_write("set x 10")  // => Ok(1)
node.client_write("set y 20")  // => Ok(2)
assert log.len()               == 2
assert committed_log().len()   == 2
assert committed_log()[0].command == "set x 10"
assert committed_log()[1].command == "set y 20"
```

### Scenario 4 — Write without being leader

```
node = Node::new(1)             // fresh node, still Follower
node.client_write("set x 10")  // => Err("not the leader")
```

### Scenario 5 — Receive valid heartbeat

```
// node is Leader in term 1
node.receive_heartbeat(2, 99)  // higher term arrives
assert role         == Follower  // stepped down
assert current_term == 2         // updated
assert voted_for    == None      // reset on term change
```

### Scenario 6 — Receive stale heartbeat

```
// node is Leader in term 3
node.receive_heartbeat(1, 99)  // stale term
assert role         == Leader   // did not step down
assert current_term == 3        // unchanged
// returns false
```

### Scenario 7 — Full lifecycle

```
node = Node::new(1)
node.trigger_election()           // term 1, becomes leader
node.client_write("set a 1")      // Ok(1)
node.client_write("set b 2")      // Ok(2)
node.receive_heartbeat(5, 99)     // higher term → step down, term 5
node.client_write("set c 3")      // Err — no longer leader
node.trigger_election()           // term 6, becomes leader again
node.client_write("set c 3")      // Ok(3)
assert committed_log().len() == 3
```

---

## Constraints

- No external crates — standard library only
- `client_write` must return `Err` if role is not Leader — no panics
- Term must always increase — it must never go backwards
- `voted_for` must reset to `None` whenever the term increases
- `committed_log()` must return only entries up to `commit_index`

---

## Key invariants to get right

- **Term never goes backwards.** Every decision in Raft flows from the term number.
- **`voted_for` resets on every term change.** A node can only vote once per term.
- **Stepping down is immediate.** The moment you see a higher term, you are a Follower.
- **Single node commits immediately.** No waiting for majority — you are the majority.

---

## Files

- `src/main.rs` — your implementation
- `Cargo.toml` — no external dependencies needed

## Reference

The original Raft paper is surprisingly readable and worth skimming before you start:
"In Search of an Understandable Consensus Algorithm" — Ongaro & Ousterhout, 2014
https://raft.github.io/raft.pdf

---

## Next

`02_raft_cluster` — multi-node Raft where elections require real majority votes
and log replication involves message passing between nodes.
