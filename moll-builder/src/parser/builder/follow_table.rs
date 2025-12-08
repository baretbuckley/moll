use std::collections::HashMap;
use super::{first_table::FirstTable, grammar};
use std::hash::Hash;

// Follow table 
// Contains set of Terminals and Final that can follow a given non-terminal in the grammar
pub struct FollowTable<T,N> {
    nexts: HashMap<N, Vec<grammar::Token<T,N>>>
}

impl<T: Eq+Hash+Copy,N: Eq+Hash+Copy> FollowTable<T,N> {

    pub fn new() -> Self {
        FollowTable { nexts: HashMap::new()}
    }

    pub fn new_from(first_table: &FirstTable<T,N>, grammar: &grammar::Grammar<T,N> ) -> Self {
        let mut nt = FollowTable {nexts: HashMap::new()};
        nt.fill(first_table, grammar);
        nt
    }

    // Appends a vec of non terminals to the 'next set' coorisponding to the 'from' non-terminal.
    fn append_nexts_non_terms(&mut self, from: N, nexts: &Vec<T>) -> bool {
        let mut change = false;
        let v = self.nexts.entry(from).or_insert(Vec::new());
        for next in nexts {
            let tk: grammar::Token<T,N> = grammar::Token::Terminal(*next);
            if !v.contains(&tk) {
                v.push(tk);
                change = true;
            }
        }
        change
    }

    // Appends a vec of grammar tokens to the 'next set' coorisponding to the 'from' non-terminal.
    fn append_nexts(&mut self, from: N, nexts: &Vec<grammar::Token<T,N>>) -> bool {
        let mut change = false;
        let v = self.nexts.entry(from).or_insert(Vec::new());
        for next in nexts {
            if !v.contains(next) {
                v.push(*next);
                change = true;
            }
        }
        change
    }


    pub fn fill(&mut self, first_table: &FirstTable<T,N>, grammar: &grammar::Grammar<T,N>) {

        // Base case: The follow set of the start symbol contains the final token
        let v = self.nexts.entry(*grammar.start()).or_insert(Vec::new());
        if !v.contains(&grammar::Token::Final) {
            v.push(grammar::Token::Final);
        }

        // Dependancy sets specify how to update the follow sets in a closure step
        // first dependancy: A -> {B}: then closure step: Follow(A) += First(B)
        // Next dependancy:  A -> {B}: then closure step: Follow(A) += Follow(B)
        let mut first_dependancies: HashMap<N, Vec<N>> = HashMap::new();
        let mut next_dependancies: HashMap<N, Vec<N>> = HashMap::new();

        // For each non-term that appears on the right side of a rule,
        // record the termanal tokens that can follow it (incomplete for full follow set)
        for (_, rule) in grammar.rules().iter().enumerate() {
            for idx in 0..rule.rule.len()-1 {
                if let grammar::Token::NonTerminal(tkn) = rule.rule[idx] {
                    match rule.rule[idx+1] {

                        grammar::Token::Terminal(next) => {
                            let v = self.nexts.entry(tkn).or_insert(Vec::new());
                            if !v.contains(&grammar::Token::Terminal(next)) {
                                v.push(grammar::Token::Terminal(next));
                            }
                        }
                        grammar::Token::NonTerminal(next) => {
                            let v = first_dependancies.entry(tkn).or_insert(Vec::new());
                            if !v.contains(&next) {
                                v.push(next);
                            }
                        }
                        grammar::Token::Final => {
                            let v = self.nexts.entry(tkn).or_insert(Vec::new());
                            if !v.contains(&grammar::Token::Final) {
                                v.push(grammar::Token::Final);
                            }
                        },
                        grammar::Token::Root => {
                            panic!("Grammar rules cannot contain Root tokens");
                        }
                    }
                }
            }
            if let Some(grammar::Token::NonTerminal(last)) = rule.rule.last() {
                let v = next_dependancies.entry(*last).or_insert(Vec::new());
                if !v.contains(&rule.result) {
                    v.push(rule.result);
                }
            }
        }

        let mut change = true;
        while change {
            change = false;
            for (non_term, deps) in first_dependancies.iter() {
                for dep in deps {
                    if let Some(buffer) = first_table.get(*dep) {
                        change |= self.append_nexts_non_terms(*non_term, buffer);
                    }
                }
            }
            for (non_term, deps) in next_dependancies.iter() {
                for dep in deps {
                    if let Some(firsts) = self.nexts.get(dep) {
                        let buffer = firsts.clone();
                        change |= self.append_nexts(*non_term, &buffer);
                    }
                }
            }
        }
    }

    pub fn get(&self, result: N) -> Option<&Vec<grammar::Token<T,N>>> {
        self.nexts.get(&result)
    }
}