use intmap::IntMap;


// pub enum FailAction {
//     Terminate(u32),
//     Jump(usize)
// }

#[derive(Debug, Copy, Clone)]
pub enum Action {
    Next(usize),
    Jump(usize),
    Terminate(u32),
    TerminateJump(u32, usize),
}


struct Node {
    next: IntMap<u32, Action>,
    fail_action: Action
}

impl Node {
    // impl<'a> Node<'a> {

    fn new(fail: Action) -> Self {
        Node{fail_action: fail, next: IntMap::new()}
    }

    fn next(&self, key: u32) -> Action {
        match self.next.get(key) {
            Some(n) => *n,
            None => self.fail_action
        }
    }

    fn add_transition(&mut self, key: u32, next: Action) {
        self.next.insert(key, next);
    }

}

pub struct FiniteAutomata {
    nodes: Vec<Node>,
    start: usize,
    cur: usize,
}

impl FiniteAutomata {

    pub fn new() -> Self {
        FiniteAutomata{nodes: Vec::<Node>::new(), start: 0, cur: 0}
    }

    fn test_action(&self, action: Action) -> bool {
        match action {
            Action::Jump(to) | Action::Next(to) => {
                if to > self.nodes.len() {
                    false
                } else {
                    true
                }
            }
            _ => true
        }
    }

    fn get_node_termination(&self, n: usize) -> Option<u32> {
        // TODO: fix for maintaining current
        match self.nodes[n].fail_action {
            Action::Jump(to) => self.get_node_termination(to),
            Action::Next(_) => None,
            Action::Terminate(id) => Some(id),
            Action::TerminateJump(id, to) => {
                Some(id)
            }
        }
    }

    pub fn parse(&mut self, input: &str) -> (usize, u32)  {
        
        let mut jump;
        for (i, c) in input.chars().enumerate() {
            jump = true;
            while jump {
                jump = false;
                match self.nodes[self.cur].next(c as u32) {
                    Action::Next(n) => self.cur = n,
                    Action::Terminate(a) => return (i, a),
                    Action::Jump(to) => {
                        jump = true;
                        self.cur = to;
                    }
                    Action::TerminateJump(a, to) => {
                        self.cur = to;
                        return (i, a)
                    }
                }
            }
        }
        if let Some(id) = self.get_node_termination(self.cur) {
            (input.len(), id)
        } else {
            (0, 0)
        }
    }

    pub fn add_transition(&mut self, key: u32, from: usize, effect: Action) {
        if !self.test_action(effect) {
            panic!("Invalid Destination of Action");
        }
        self.nodes[from].add_transition(key, effect);
    }

    pub fn add_transition_between(&mut self, key: u32, from: usize, to: usize) {
        if from >= self.nodes.len() || to >= self.nodes.len() {panic!("Reference to non-existant node");}
        self.nodes[from].add_transition(key, Action::Next(to));
    }

    pub fn create_node(&mut self, fail_action: Action) -> usize {
        self.nodes.push(Node::new(fail_action));
        if !self.test_action(fail_action) {
            panic!("Invalid Destination of Fail Action");
        }
        self.nodes.len()-1
    }

    pub fn create_default_node(&mut self) -> usize {
        self.nodes.push(Node::new(Action::Terminate(0)));
        self.nodes.len()-1
    }

    pub fn set_fail_action(&mut self, node: usize, action: Action) {
        if !self.test_action(action) {
            panic!("Invalid Destination of Action");
        }
        self.nodes[node].fail_action = action;
    }

}
