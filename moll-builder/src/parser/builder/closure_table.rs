use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::hash::Hash;
use crate::parser::grammar;


pub struct ClosureTable<N> {
    items: HashMap<N, HashSet<grammar::Item>>,
}


impl<N: Hash+Eq+Copy> ClosureTable<N> {

    pub fn new_from<T>(grammar: &grammar::Grammar<T,N>) -> Self {
        let mut table: ClosureTable<N> = ClosureTable{ items: HashMap::new()};
        let mut nonterm_starts: HashMap<N, Vec<N>> = HashMap::new();
        
        // Get set of first terminals (self.nexts) and non terminals (nonterm_stats) 
        for (idx, r) in grammar.rules().iter().enumerate() {
            match r.rule.first().unwrap() {
                grammar::Token::NonTerminal(t) => {
                    let starts = nonterm_starts.entry(r.result).or_insert(Vec::new());
                    if !starts.contains(t) {
                        starts.push(*t);
                    }
                    let set = table.items.entry(r.result).or_insert(HashSet::new());
                    set.insert(grammar::Item::new(grammar::RuleIdx::new(idx), 0 ));
                }
                grammar::Token::Terminal(_) => {
                    let set = table.items.entry(r.result).or_insert(HashSet::new());
                    set.insert(grammar::Item::new(grammar::RuleIdx::new(idx), 0 ));
                }
                _ => {}
            }
        }
        table
    }

    // updates closre : START(result) := START(result) + START(nonterm) 
    #[inline]
    fn closure_step(&mut self, result: N, nonterm: N) -> bool {
        if let Some(first_vec) = self.items.get(&nonterm) {
            let interum = first_vec.clone();
            let start_set = self.items.entry(result).or_insert(HashSet::new());
            let mut modified: bool = false;
            for new_start in interum {
                modified |= start_set.insert(new_start);
            }
            return modified;
        }
        false
    }

    fn closure(&mut self, nonterm_starts: HashMap<N, Vec<N>>) {
        // process cloasure
        let mut modified: bool = true;
        while modified {
            modified = false;
            for (result, nonterm_vec) in nonterm_starts.iter() {
                for nonterm in nonterm_vec {
                    modified |= self.closure_step(*result, *nonterm)
                }
            } 
        } 
    }

    pub fn items(&self) -> &HashMap<N, HashSet<grammar::Item>> {&self.items}

    pub fn get(&self, result: N) -> Option<&HashSet<grammar::Item>> {
        self.items.get(&result)
    }

}


impl<N: Display> ClosureTable<N> {
    pub fn display_table<T: Display>(&self, grammar: &grammar::Grammar<T,N>) {
        for (result, starts) in self.items.iter() {
            print!("{:} :", *result);
            for s in starts {
                print!(" {{{:}}}", s.display(grammar));
            }
            println!("");
        }
    }
}

