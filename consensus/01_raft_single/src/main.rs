#[derive(Debug, Clone, Copy, PartialEq)]
enum NodeRole {
    Follower,
    Candidate,
    Leader,
}

#[derive(Debug, PartialEq, Eq)]
struct LogEntry {
    term: u64,
    command: String,
}

#[derive(Debug)]
struct Node {
    id: u64,
    current_term: u64,
    voted_for: Option<u64>,
    log: Vec<LogEntry>,
    commit_index: usize,
    role: NodeRole
}

impl Node {
    fn new(id: u64) -> Self {
        Self {
            id: id,
            current_term: 0,
            voted_for: None,
            log: Vec::new(),
            commit_index: 0,
            role: NodeRole::Follower
        }
    }

    fn trigger_election(&mut self) {
        // single node, the candidate is the only voter so it wins immediately
        self.current_term += 1;
        self.voted_for = Some(self.id);
        self.role = NodeRole::Leader;
    }

    fn client_write(&mut self, command: &str) -> Result<usize, &str> {
        if self.role != NodeRole::Leader {
            return Err("not the leader");
        }   
        self.log.push(LogEntry {
            term: self.current_term,
            command: command.to_string()
        });
        self.commit_index = self.log.len();
        Ok(self.commit_index)
    }

    fn receive_heartbeat(&mut self, leader_term: u64, _leader_id: u64) -> bool {
        if leader_term > self.current_term {
            self.role = NodeRole::Follower;
            self.current_term = leader_term;
            self.voted_for = None;
            true
        } else {
            if leader_term < self.current_term {
                false
            } else {
                true
            }   
        }
    }

    fn committed_log(&self) -> &[LogEntry] {
        &self.log[..self.commit_index]
    }
}

fn main() {
    let mut node = Node::new(1);
    node.trigger_election();
    println!("node = {:?}", node);
    node.client_write("set x 10").unwrap();
    println!("node = {:?}", node);
    node.receive_heartbeat(2, 99);
    println!("node = {:?}", node);
    println!("committed_log = {:?}", node.committed_log());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scenario_1_fresh_node() {
        let node = Node::new(1);
        assert_eq!(node.role, NodeRole::Follower);
        assert_eq!(node.current_term, 0);
        assert!(node.log.is_empty());
        assert_eq!(node.commit_index, 0);
    }

    #[test]
    fn scenario_2_election_timeout() {
        let mut node = Node::new(1);
        node.trigger_election();
        assert_eq!(node.role, NodeRole::Leader);
        assert_eq!(node.current_term, 1);
        assert_eq!(node.voted_for, Some(1));
    }

    #[test]
    fn scenario_3_write_as_leader() {
        let mut node = Node::new(1);
        node.trigger_election();
        assert_eq!(node.client_write("set x 10"), Ok(1));
        assert_eq!(node.client_write("set y 20"), Ok(2));
        assert_eq!(node.log.len(), 2);
        assert_eq!(node.committed_log().len(), 2);
        assert_eq!(node.committed_log()[0].command, "set x 10");
        assert_eq!(node.committed_log()[1].command, "set y 20");
    }

    #[test]
    fn scenario_4_write_without_being_leader() {
        let mut node = Node::new(1);
        assert_eq!(node.client_write("set x 10"), Err("not the leader"));
    }

    #[test]
    fn scenario_5_receive_valid_heartbeat() {
        let mut node = Node::new(1);
        node.trigger_election();
        assert_eq!(node.role, NodeRole::Leader);
        assert!(node.receive_heartbeat(2, 99));
        assert_eq!(node.role, NodeRole::Follower);
        assert_eq!(node.current_term, 2);
        assert_eq!(node.voted_for, None);
    }

    #[test]
    fn scenario_6_receive_stale_heartbeat() {
        let mut node = Node::new(1);
        node.trigger_election();
        node.trigger_election();
        node.trigger_election();
        assert_eq!(node.role, NodeRole::Leader);
        assert_eq!(node.current_term, 3);
        assert!(!node.receive_heartbeat(1, 99));
        assert_eq!(node.role, NodeRole::Leader);
        assert_eq!(node.current_term, 3);
    }

    #[test]
    fn scenario_7_full_lifecycle() {
        let mut node = Node::new(1);
        node.trigger_election();
        assert_eq!(node.client_write("set a 1"), Ok(1));
        assert_eq!(node.client_write("set b 2"), Ok(2));
        node.receive_heartbeat(5, 99);
        assert_eq!(node.client_write("set c 3"), Err("not the leader"));
        node.trigger_election();
        assert_eq!(node.client_write("set c 3"), Ok(3));
        assert_eq!(node.committed_log().len(), 3);
        assert_eq!(node.committed_log()[0].command, "set a 1");
        assert_eq!(node.committed_log()[1].command, "set b 2");
        assert_eq!(node.committed_log()[2].command, "set c 3");
    }
}