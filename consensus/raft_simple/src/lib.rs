//! In-memory Raft building blocks for a single-process simulation.
//!
//! Implement election and log replication per `problem.md`.

use std::cmp::Ordering;

/// Monotonic leader generation (Raft paper).
pub type Term = u64;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct NodeId(pub u64);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LogEntry {
    pub term: Term,
    pub command: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Role {
    Follower,
    Candidate,
    Leader,
}

#[derive(Debug)]
pub struct RaftState {
    pub id: NodeId,
    pub current_term: Term,
    pub voted_for: Option<NodeId>,
    pub log: Vec<LogEntry>,
    pub commit_index: usize,
    pub last_applied: usize,
    pub role: Role,
}

impl RaftState {
    pub fn new(id: NodeId) -> Self {
        Self {
            id,
            current_term: 0,
            voted_for: None,
            log: Vec::new(),
            commit_index: 0,
            last_applied: 0,
            role: Role::Follower,
        }
    }

    /// Last log term, or 0 if the log is empty (Raft RPC fields).
    pub fn last_log_term(&self) -> Term {
        self.log.last().map(|e| e.term).unwrap_or(0)
    }

    pub fn last_log_index(&self) -> usize {
        self.log.len()
    }
}

/// Compare (term, index) as in Raft's "up-to-date" check for RequestVote.
pub fn log_is_up_to_date(last_term: Term, last_index: usize, other_last_term: Term, other_last_index: usize) -> bool {
    match last_term.cmp(&other_last_term) {
        Ordering::Greater => true,
        Ordering::Less => false,
        Ordering::Equal => last_index >= other_last_index,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_node_starts_as_follower_with_empty_log() {
        let s = RaftState::new(NodeId(1));
        assert_eq!(s.role, Role::Follower);
        assert_eq!(s.last_log_term(), 0);
        assert_eq!(s.last_log_index(), 0);
    }

    #[test]
    fn log_up_to_date_orders_by_term_then_index() {
        assert!(log_is_up_to_date(2, 1, 1, 9));
        assert!(!log_is_up_to_date(1, 9, 2, 1));
        assert!(log_is_up_to_date(1, 5, 1, 3));
        assert!(log_is_up_to_date(1, 3, 1, 3));
    }
}
