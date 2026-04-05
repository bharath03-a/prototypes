#[derive(Debug, Clone, Copy, PartialEq)]
enum NodeRole {
    Follower,
    Candidate,
    Leader,
}

#[derive(Debug)]
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