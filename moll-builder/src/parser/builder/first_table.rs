use std::collections::HashMap;
use super::{closure_table::ClosureTable, grammar};
use std::hash::Hash;


pub struct FirstTable<T,N> {
    firsts: HashMap<N, Vec<T>>
}

impl<T: PartialEq+Copy,N: Hash+Eq+Copy> FirstTable<T,N> {

    pub fn new() -> Self {
        FirstTable::<T,N> { firsts: HashMap::new()}
    }

    pub fn new_from(closure_table: &ClosureTable<N>, grammar: &grammar::Grammar<T,N>) -> Self {
        let mut ft = FirstTable::<T,N> {firsts: HashMap::new()};
        ft.fill(closure_table, grammar);
        ft
    }

    pub fn fill(&mut self, closure_table: &ClosureTable<N>, grammar: &grammar::Grammar<T,N>) {
        for (non_term, items) in closure_table.items() {
            let firsts = self.firsts.entry(*non_term).or_insert(Vec::new());
            for item in items {
                if let Some(grammar::Token::Terminal(first)) = item.first(grammar) {
                    if !firsts.contains(first) {
                        firsts.push(*first);
                    }
                }
            }
        } 
    }

    pub fn get(&self, result: N) -> Option<&Vec<T>> {
        self.firsts.get(&result)
    }
}