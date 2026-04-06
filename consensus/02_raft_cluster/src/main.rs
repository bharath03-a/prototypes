//! `02_raft_cluster` — multi-node Raft **in-process** (`PROBLEM.md`).
//!
//! **Indexing:** `log` is **0-based**. `commit_index` is an **exclusive end** of the
//! committed prefix: committed entries are `log[..commit_index]` (same idea as
//! `01_raft_single`’s `&log[..commit_index]`).

use std::collections::{HashMap, VecDeque};

// ---------------------------------------------------------------------------
// Roles
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NodeRole {
    Follower,
    Candidate,
    Leader,
}

// ---------------------------------------------------------------------------
// Log
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
struct LogEntry {
    term: u64,
    command: String,
}

// ---------------------------------------------------------------------------
// Node
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct Node {
    id: u64,
    role: NodeRole,
    current_term: u64,
    voted_for: Option<u64>,
    log: Vec<LogEntry>,
    /// Exclusive end of committed prefix: `log[..commit_index]` is committed.
    commit_index: usize,
    next_index: HashMap<u64, usize>,
    /// Exclusive end index replicated on each follower (`0` = empty log on follower).
    match_index: HashMap<u64, usize>,
    /// Abstract ticks until this follower/candidate may call `trigger_election`.
    election_ticks: u64,
    peer_ids: Vec<u64>,
    votes: u32,
    cluster_size: usize,
}

// ---------------------------------------------------------------------------
// Messages
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct MessageEnvelope {
    from: u64,
    to: u64,
    message: Message,
}

#[derive(Debug, Clone)]
enum Message {
    RequestVote {
        term: u64,
        candidate_id: u64,
        last_log_index: usize,
        last_log_term: u64,
    },
    RequestVoteResponse {
        term: u64,
        vote_granted: bool,
    },
    AppendEntries {
        term: u64,
        leader_id: u64,
        prev_log_index: usize,
        prev_log_term: u64,
        entries: Vec<LogEntry>,
        /// Same convention as `commit_index` on the leader (exclusive end).
        leader_commit: usize,
    },
    AppendEntriesResponse {
        term: u64,
        success: bool,
    },
}

// ---------------------------------------------------------------------------
// Node impl
// ---------------------------------------------------------------------------

impl Node {
    fn new(id: u64, peer_ids: Vec<u64>) -> Self {
        let cluster_size = peer_ids.len() + 1;
        Self {
            id,
            role: NodeRole::Follower,
            current_term: 0,
            voted_for: None,
            log: Vec::new(),
            commit_index: 0,
            next_index: HashMap::new(),
            match_index: HashMap::new(),
            election_ticks: 150 + (id.wrapping_mul(37) % 150),
            peer_ids,
            votes: 0,
            cluster_size,
        }
    }

    fn majority(&self) -> usize {
        self.cluster_size / 2 + 1
    }

    /// `(last_index, last_term)` with `(0, 0)` when the log is empty (RPC convention).
    fn last_log_meta(&self) -> (usize, u64) {
        match self.log.last() {
            None => (0, 0),
            Some(e) => (self.log.len() - 1, e.term),
        }
    }

    fn log_is_up_to_date(&self, cand_last_index: usize, cand_last_term: u64) -> bool {
        let (my_idx, my_term) = self.last_log_meta();
        cand_last_term > my_term || (cand_last_term == my_term && cand_last_index >= my_idx)
    }

    /// Followers/candidates count down; leaders rely on the cluster for heartbeats.
    /// Use via [`Cluster::tick_node`] when you simulate timeouts.
    #[allow(dead_code)]
    fn tick(&mut self) -> Vec<MessageEnvelope> {
        if self.role == NodeRole::Leader {
            return Vec::new();
        }
        self.election_ticks = self.election_ticks.saturating_sub(1);
        if self.election_ticks == 0 {
            return self.trigger_election();
        }
        Vec::new()
    }

    fn trigger_election(&mut self) -> Vec<MessageEnvelope> {
        self.current_term += 1;
        self.role = NodeRole::Candidate;
        self.voted_for = Some(self.id);
        self.votes = 1;
        self.election_ticks = 150 + (self.id.wrapping_mul(37) % 150);

        if self.peer_ids.is_empty() {
            self.become_leader();
            return Vec::new();
        }

        let (last_log_index, last_log_term) = self.last_log_meta();
        self.peer_ids
            .iter()
            .map(|&peer_id| MessageEnvelope {
                from: self.id,
                to: peer_id,
                message: Message::RequestVote {
                    term: self.current_term,
                    candidate_id: self.id,
                    last_log_index,
                    last_log_term,
                },
            })
            .collect()
    }

    fn become_leader(&mut self) {
        self.role = NodeRole::Leader;
        let next = self.log.len();
        for &pid in &self.peer_ids {
            self.next_index.insert(pid, next);
            self.match_index.insert(pid, 0);
        }
    }

    fn handle_request_vote(&mut self, envelope: &MessageEnvelope) -> MessageEnvelope {
        let Message::RequestVote {
            term,
            candidate_id,
            last_log_index,
            last_log_term,
        } = &envelope.message
        else {
            panic!("expected RequestVote");
        };

        if *term < self.current_term {
            return MessageEnvelope {
                from: self.id,
                to: envelope.from,
                message: Message::RequestVoteResponse {
                    term: self.current_term,
                    vote_granted: false,
                },
            };
        }

        if *term > self.current_term {
            self.current_term = *term;
            self.voted_for = None;
            self.role = NodeRole::Follower;
        }

        let grant = (self.voted_for.is_none() || self.voted_for == Some(*candidate_id))
            && self.log_is_up_to_date(*last_log_index, *last_log_term);

        if grant {
            self.voted_for = Some(*candidate_id);
        }

        MessageEnvelope {
            from: self.id,
            to: envelope.from,
            message: Message::RequestVoteResponse {
                term: self.current_term,
                vote_granted: grant,
            },
        }
    }

    fn handle_request_vote_response(&mut self, envelope: &MessageEnvelope) {
        let Message::RequestVoteResponse { term, vote_granted } = &envelope.message else {
            panic!("expected RequestVoteResponse");
        };

        if *term > self.current_term {
            self.current_term = *term;
            self.role = NodeRole::Follower;
            self.voted_for = None;
            self.votes = 0;
            return;
        }

        if *term < self.current_term || self.role != NodeRole::Candidate {
            return;
        }

        if *vote_granted {
            self.votes += 1;
        }

        if self.votes as usize >= self.majority() {
            self.become_leader();
        }
    }

    /// Follower (or candidate stepping down) applies leader append/heartbeat.
    fn handle_append_entries(&mut self, envelope: &MessageEnvelope) -> MessageEnvelope {
        let Message::AppendEntries {
            term,
            leader_id: _leader_id,
            prev_log_index,
            prev_log_term,
            entries,
            leader_commit,
        } = &envelope.message
        else {
            panic!("expected AppendEntries");
        };

        if *term < self.current_term {
            return MessageEnvelope {
                from: self.id,
                to: envelope.from,
                message: Message::AppendEntriesResponse {
                    term: self.current_term,
                    success: false,
                },
            };
        }

        // Newer or equal term → recognize this leader, become follower.
        if *term >= self.current_term {
            if *term > self.current_term {
                self.current_term = *term;
                self.voted_for = None;
            }
            self.role = NodeRole::Follower;
        }

        let prefix_ok = if *prev_log_index == 0 && *prev_log_term == 0 {
            // “No predecessor”: only valid when our log is empty before append.
            self.log.is_empty()
        } else if *prev_log_index >= self.log.len() {
            false
        } else {
            self.log[*prev_log_index].term == *prev_log_term
        };

        if !prefix_ok {
            return MessageEnvelope {
                from: self.id,
                to: envelope.from,
                message: Message::AppendEntriesResponse {
                    term: self.current_term,
                    success: false,
                },
            };
        }

        // Truncate past the agreed prefix, then append fresh suffix.
        let keep = if *prev_log_index == 0 && *prev_log_term == 0 {
            0
        } else {
            prev_log_index + 1
        };
        self.log.truncate(keep);
        self.log.extend(entries.iter().cloned());

        // Follower commit watermark: cannot commit beyond what we have.
        let last_new = self.log.len();
        self.commit_index = (*leader_commit).min(last_new);

        MessageEnvelope {
            from: self.id,
            to: envelope.from,
            message: Message::AppendEntriesResponse {
                term: self.current_term,
                success: true,
            },
        }
    }

    /// Leader adjusts `next_index` / `match_index` and may advance `commit_index`.
    ///
    /// On success the follower’s log matches the leader’s through `self.log.len()`
    /// (exclusive match index = full leader length).
    fn handle_append_entries_response(&mut self, envelope: &MessageEnvelope) {
        let Message::AppendEntriesResponse { term, success } = &envelope.message else {
            panic!("expected AppendEntriesResponse");
        };
        let peer = envelope.from;

        if *term > self.current_term {
            self.current_term = *term;
            self.role = NodeRole::Follower;
            self.voted_for = None;
            return;
        }

        if self.role != NodeRole::Leader || *term < self.current_term {
            return;
        }

        if *success {
            self.match_index.insert(peer, self.log.len());
            self.next_index.insert(peer, self.log.len());
        } else {
            let ni = self.next_index.entry(peer).or_insert(0);
            *ni = (*ni).saturating_sub(1);
        }

        self.advance_leader_commit();
    }

    /// Pick the largest exclusive `commit_index` where `log[new_commit - 1].term == current_term`
    /// and a quorum has `match_index >= new_commit`.
    fn advance_leader_commit(&mut self) {
        if self.role != NodeRole::Leader || self.log.is_empty() {
            return;
        }

        for new_commit in (self.commit_index + 1..=self.log.len()).rev() {
            let last_idx = new_commit - 1;
            if self.log[last_idx].term != self.current_term {
                continue;
            }
            let mut count = 1usize; // leader has the full prefix
            for &pid in &self.peer_ids {
                if *self.match_index.get(&pid).unwrap_or(&0) >= new_commit {
                    count += 1;
                }
            }
            if count >= self.majority() {
                self.commit_index = new_commit;
                break;
            }
        }
    }

    fn append_entries_rpc_to(&self, peer: u64) -> MessageEnvelope {
        let ni = *self.next_index.get(&peer).unwrap_or(&0);
        let (prev_log_index, prev_log_term) = if ni == 0 {
            (0usize, 0u64)
        } else {
            (ni - 1, self.log[ni - 1].term)
        };
        let entries = self.log.get(ni..).unwrap_or(&[]).to_vec();
        MessageEnvelope {
            from: self.id,
            to: peer,
            message: Message::AppendEntries {
                term: self.current_term,
                leader_id: self.id,
                prev_log_index,
                prev_log_term,
                entries,
                leader_commit: self.commit_index,
            },
        }
    }

    fn leader_append(&mut self, command: String) -> usize {
        self.log.push(LogEntry {
            term: self.current_term,
            command,
        });
        self.log.len()
    }
}

// ---------------------------------------------------------------------------
// Cluster
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct Cluster {
    nodes: HashMap<u64, Node>,
    inbox: VecDeque<MessageEnvelope>,
}

impl Cluster {
    fn new(ids: &[u64]) -> Self {
        let mut nodes = HashMap::new();
        for &id in ids {
            let peers: Vec<u64> = ids.iter().copied().filter(|&x| x != id).collect();
            nodes.insert(id, Node::new(id, peers));
        }
        Self {
            nodes,
            inbox: VecDeque::new(),
        }
    }

    fn node(&self, id: u64) -> &Node {
        self.nodes.get(&id).expect("unknown id")
    }

    fn node_mut(&mut self, id: u64) -> &mut Node {
        self.nodes.get_mut(&id).expect("unknown id")
    }

    fn enqueue_all(&mut self, msgs: Vec<MessageEnvelope>) {
        for m in msgs {
            self.inbox.push_back(m);
        }
    }

    fn deliver_one(&mut self, env: MessageEnvelope) {
        let reply = match &env.message {
            Message::RequestVote { .. } => Some(self.node_mut(env.to).handle_request_vote(&env)),
            Message::RequestVoteResponse { .. } => {
                self.node_mut(env.to).handle_request_vote_response(&env);
                None
            }
            Message::AppendEntries { .. } => Some(self.node_mut(env.to).handle_append_entries(&env)),
            Message::AppendEntriesResponse { .. } => {
                self.node_mut(env.to).handle_append_entries_response(&env);
                None
            }
        };
        if let Some(r) = reply {
            self.deliver_one(r);
        }
    }

    fn drain_inbox(&mut self) {
        while let Some(m) = self.inbox.pop_front() {
            self.deliver_one(m);
        }
    }

    #[allow(dead_code)]
    fn tick_node(&mut self, id: u64) {
        let msgs = self.node_mut(id).tick();
        self.enqueue_all(msgs);
        self.drain_inbox();
    }

    fn start_election(&mut self, id: u64) {
        let msgs = self.node_mut(id).trigger_election();
        self.enqueue_all(msgs);
        self.drain_inbox();
    }

    fn leader_broadcast_append(&mut self, leader_id: u64) {
        let peers = self.node(leader_id).peer_ids.clone();
        let mut out = Vec::new();
        for p in peers {
            out.push(self.node(leader_id).append_entries_rpc_to(p));
        }
        self.enqueue_all(out);
        self.drain_inbox();
    }

    fn client_write(&mut self, leader_id: u64, command: &str) -> Result<usize, &'static str> {
        {
            let leader = self.node_mut(leader_id);
            if leader.role != NodeRole::Leader {
                return Err("not the leader");
            }
            leader.leader_append(command.to_string());
        }
        // Replicate new entries; responses may advance the leader’s `commit_index` **after**
        // followers applied this RPC, so `leader_commit` in-flight can be stale by one step.
        self.leader_broadcast_append(leader_id);
        // Empty append (heartbeat) carries the updated `leader_commit` so followers catch up.
        self.leader_broadcast_append(leader_id);
        Ok(self.node(leader_id).log.len())
    }
}

fn main() {
    let ids = [1_u64, 2, 3];
    let mut c = Cluster::new(&ids);
    c.start_election(1);
    println!(
        "leader id 1 role {:?} term {}",
        c.node(1).role,
        c.node(1).current_term
    );
    let _ = c.client_write(1, "hello");
    println!("commit_index on leader = {}", c.node(1).commit_index);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn elect_and_replicate_three_nodes() {
        let ids = [1_u64, 2, 3];
        let mut c = Cluster::new(&ids);
        c.start_election(1);
        assert_eq!(c.node(1).role, NodeRole::Leader);

        c.client_write(1, "a").unwrap();
        c.client_write(1, "b").unwrap();

        assert_eq!(c.node(1).log.len(), 2);
        assert_eq!(c.node(1).commit_index, 2);
        assert_eq!(c.node(2).log, c.node(1).log);
        assert_eq!(c.node(3).log, c.node(1).log);
        assert_eq!(c.node(2).commit_index, 2);
        assert_eq!(c.node(3).commit_index, 2);
    }
}
