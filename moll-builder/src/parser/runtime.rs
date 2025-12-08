
use std::{collections::HashMap, fmt::Display, hash::Hash};
use std::fmt::Debug;

use super::{grammar, ParserBuild};

#[derive(Copy, Clone)]
struct NodeIdx {
    pub idx: usize
}

impl NodeIdx {
    pub fn new(idx: usize) -> Self {NodeIdx{idx}}
}

#[derive(Copy, Clone)]
enum Action {
    Shift(NodeIdx),
    Reduce(grammar::RuleIdx),
}

impl Action {
    pub fn display<'a, T: Display, N: Display> (&'a self, grammar: &'a grammar::Grammar<T,N>) -> ActionDisplay<'a, T, N> {
        ActionDisplay { action: self, grammar }
    }
}

struct ActionDisplay<'a, T, N> {
    action: &'a Action,
    grammar: &'a grammar::Grammar<T,N>,
}

impl<'a, T: Display, N: Display> Display for ActionDisplay<'a, T, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.action {
            Action::Shift(node_idx) => {_ = write!(f, "Shift( {} )", node_idx.idx);}
            Action::Reduce(rule_idx) => {_ = write!(f, "Reduce( {} )", grammar::Item::new(*rule_idx, 0).display(self.grammar));}
        }
        write!(f, "")
    }
}

struct Node<T, N> {
    next: HashMap<grammar::Token<T,N>, Action>,
}

impl<T: Eq+Hash, N: Eq+Hash> Node<T,N> {

    fn new() -> Self {
        Node{next: HashMap::new()}
    }

    fn next(&self, key: grammar::Token<T,N>) -> Option<&Action> {
        self.next.get(&key)
    }

    fn add_transition(&mut self, key: grammar::Token<T,N>, next: Action) {
        self.next.insert(key, next);
    }

    pub fn display<'a> (&'a self, grammar: &'a grammar::Grammar<T,N>) -> NodeDisplay<'a, T, N> {
        NodeDisplay { node: self, grammar }
    }
}

struct NodeDisplay<'a, T, N> {
    node: &'a Node<T,N>,
    grammar: &'a grammar::Grammar<T,N>,
}

impl<'a, T: Display, N: Display> Display for NodeDisplay<'a, T, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        _ = write!(f, "{{ ");
        for (by, action) in self.node.next.iter() {
            _ = write!(f, "{{ {by} -> {} ", action.display(self.grammar));
        }

        write!(f, "}}")
    }
}


pub struct LALRParser<T, N> {
    nodes: Vec<Node<T,N>>,
    start: NodeIdx,
    cur: NodeIdx,
}

impl<T: Eq+Hash+Copy+Debug, N: Eq+Hash+Copy+Debug> LALRParser<T,N> {

    fn reparse(&mut self, stack: &mut Vec<grammar::Token<T,N>>, grammar: &grammar::Grammar<T,N>) {
        println!("Backtracking");
        self.cur = self.start;
        let mut new_stack: Vec<grammar::Token<T,N>> = Vec::new();

        for (_, c) in stack.iter().enumerate() {
            self.parse_step(*c, &mut new_stack, grammar)
        }
        *stack = new_stack;
    }

    fn parse_step(&mut self, c: grammar::Token<T,N>, stack: &mut Vec<grammar::Token<T,N>>, grammar: &grammar::Grammar<T,N>) {
        if let Some(a) = self.nodes[self.cur.idx].next(c) {
            match a {
                Action::Shift(n) =>  {
                    stack.push(c);
                    self.cur = *n;
                }
                Action::Reduce(r) => {
                    let rule = grammar.get(*r).unwrap();
                    stack.truncate(stack.len() - rule.rule.len()+1);
                    *stack.last_mut().unwrap() = grammar::Token::NonTerminal(rule.result);
                    self.reparse(stack, grammar);
                    self.parse_step(c, stack, grammar);
                },
            }
        } else {
            println!("Fail, no know action for input {:?} at state {}", c, self.cur.idx);
        }
    }

    pub fn parse<A: Assembler>(&mut self, input: Vec<grammar::Token<T,N>>, grammar: &grammar::Grammar<T,N>) {
        let mut stack: Vec<grammar::Token<T,N>> = Vec::new();
        
        println!("Parsing input");
        for (_, c) in input.iter().enumerate() {
            self.parse_step(*c, &mut stack, grammar);
        }
        self.parse_step(grammar::Token::Final, &mut stack, grammar);

        print!("Final stack: ");
        for s in stack.iter() {
            print!("{:?} ", s);
        }
        println!("");
    }

    pub fn display<'a> (&'a self, grammar: &'a grammar::Grammar<T,N>) -> LALRParserDisplay<'a, T, N> {
        LALRParserDisplay { parser: self, grammar }
    }

}

pub struct LALRParserDisplay<'a, T, N> {
    parser: &'a LALRParser<T,N>,
    grammar: &'a grammar::Grammar<T,N>,
}

impl<T: Eq+Hash+Display, N: Eq+Hash+Display> Display for LALRParserDisplay<'_, T, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        _ = write!(f, "Network {{\n");
        for (i, n) in self.parser.nodes.iter().enumerate() {
            _ = write!(f, "\t{i}: {}\n", n.display(self.grammar));
        }
        write!(f, "}}")
    }
}

impl<T: Eq+Hash, N: Eq+Hash> ParserBuild<T,N> for LALRParser<T,N> {
    
    fn new() -> Self {
        LALRParser {nodes: Vec::new(), start: NodeIdx::new(0), cur: NodeIdx::new(0)}
    }

    // fn new_state(&mut self) -> usize {
    //     self.nodes.push(Node::new());
    //     self.nodes.len()-1
    // }
    fn new_state(&mut self) -> usize {
        self.nodes.push(Node::new());
        println!("creating new state");
        self.nodes.len()-1
    }

    fn add_connection(&mut self, condition: grammar::Token<T,N>, from: usize, to: usize) {
        println!("Creating connection: from {from}, to {to}");
        assert!(to < self.nodes.len());
        self.nodes[from].add_transition(condition, Action::Shift(NodeIdx::new(to)));
    }

    fn new_reduction(&mut self, condition: grammar::Token<T,N>, from: usize, rule: grammar::RuleIdx) {
        println!("Creating reduction from {from}, with rule {}", rule.idx);
        self.nodes[from].add_transition(condition, Action::Reduce(rule));
    }

}


pub trait Assembler {

}

pub struct AssemTmp {

}

impl Assembler for AssemTmp {

}