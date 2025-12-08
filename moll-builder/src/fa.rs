// use std::{collections::HashMap, hash::Hash};

// use intmap::IntMap;


// pub trait FAProducer<Symbol, State, Result> {

//     fn new() -> Self;

//     fn new_state(&mut self, result: Result) -> State;

//     fn set_state_result(&mut self, state: State, result: Result) -> State;

//     fn new_connection(&mut self, condition: Symbol, from: State, to: State) -> bool;

// }

// trait FA<Symbol, State, Result>: FAProducer<Symbol, State, Result> {

//     fn parse<T: IntoIterator<Item = Symbol>>(input: T) -> Result;

// }

// pub trait Action: Copy {
//     fn validate(&self) -> bool;
//     fn default() -> Self;
// }


// struct Node<S, T>
//     where 
//         S: Hash + Eq
// {
//     next: HashMap<S, usize>,
//     fail_action: T
// }

// impl<S, T> Node<S, T> 
// where 
//         S: Hash + Eq,
//         T: Copy
// {
//     fn new(fail: T) -> Self {
//         Node{fail_action: fail, next: HashMap::new()}
//     }

//     fn next(&self, key: S) -> Result<usize, T> {
//         match self.next.get(&key) {
//             Some(n) => Ok(*n),
//             None => Err(self.fail_action)
//         }
//     }

//     fn add_transition(&mut self, key: S, next: usize) {
//         self.next.insert(key, next);
//     }

// }


// pub struct FiniteAutomata<S, T>
// where 
//         S: Hash + Eq,
// {
//     nodes: Vec<Node<S, T>>,
//     start: usize,
//     cur: usize,
// }

// impl<S, T> FAProducer<S, usize, T> for FiniteAutomata<S, T>
// where 
//         S: Hash + Eq,
//         T: Copy
// {
    
//     fn new() -> Self {
//         FiniteAutomata{nodes: Vec::<Node<S, T>>::new(), start: 0, cur: 0}
//     }

//     fn new_state(&mut self, result: T) -> usize {
//         self.nodes.push(Node::new(result));
//         self.nodes.len()-1
//     }

//     fn set_state_result(&mut self, state: usize, result: T) -> usize {
//         self.nodes[state].fail_action = result;
//         state
//     }

//     fn new_connection(&mut self, condition: S, from: usize, to: usize) -> bool {
        
//     }
// }