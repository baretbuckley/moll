use std::{collections::BTreeMap, fmt::Display};
use std::fmt::Debug;
// use crate::parser;
use super::{grammar, ParserBuild};

mod closure_table;
mod follow_table;
use std::hash::Hash;
mod first_table;
mod table;




pub struct Builder<'a, T, N, P: ParserBuild<T,N>> {
    grammar: &'a grammar::Grammar<T,N>,
    tables: Vec<table::Table>,
    // table_set: HashMap<TableHashProxy<'a>, usize>, // TODO: find hash that works with borrow checker and avoids copying Table
    closure_table: closure_table::ClosureTable<N>,
    first_table: first_table::FirstTable<T,N>,
    follow_table: follow_table::FollowTable<T,N>,
    network: P,
}

impl<'a, T: Eq+Hash+Copy+Ord+Debug, N: Eq+Hash+Copy+Ord+Debug, P: ParserBuild<T,N>> Builder<'a, T, N, P> {
    pub fn build_from(grammar: &'a grammar::Grammar<T,N>) -> Self {
        let mut builder = Builder {
            grammar: &grammar,
            tables: Vec::new(),
            closure_table: closure_table::ClosureTable::new_from(&grammar),
            first_table: first_table::FirstTable::new(),
            follow_table: follow_table::FollowTable::new(),
            network: P::new()
        };

        builder.first_table.fill(&builder.closure_table, grammar);
        builder.follow_table.fill(&builder.first_table, grammar);
        if let Some(tmp) = builder.closure_table.get(*grammar.start()) {
            let table = table::Table::new_from_ref(tmp, &builder.closure_table, grammar);
            let idx = builder.register_table_force(table);
            println!("given idx is {idx}");
            // builder.display();
            builder.propagate(idx);
            // builder.display();
        }
        
        builder
    }

    // Constructs a new state in the parser network from the given table,
    // adding reductions as needed based on completed items in the table
    fn new_state(&mut self, table: &table::Table) {
        let state = self.network.new_state();
        // For each completed rule in the table, add reduction to the state, using follow table for lookaheads
        for r in table.items().iter() {
            let rule = self.grammar.get(r.rule_idx()).unwrap();
            if r.completed_rule(rule) {
                println!("Adding ruduction with rule");
                if let Some(v) = self.follow_table.get(rule.result) {
                    println!("Found follow set for rule");
                    for next in v {
                        println!("  on lookahead {:?}", next);
                        self.network.new_reduction(*next, state, r.rule_idx());
                    }
                }
            }
        }
    }

    // Regesters the table without checking if it matches an already regestered table
    fn register_table_force(&mut self, table: table::Table) -> usize {
        self.new_state(&table);
        self.tables.push(table);
        let idx = self.tables.len()-1;
        idx
    }

    fn register_table(&mut self, table: table::Table) -> Result<usize, usize> {
        for (i, t) in self.tables.iter().enumerate() {
            if *t == table {
                return Err(i);
            }
        }
        Ok(self.register_table_force(table))
    }

    fn add_connection(&mut self, cond: grammar::Token<T,N>, from: usize, to: usize) {
        self.network.add_connection(cond, from, to);
    }   

    fn add_table(&mut self, table: table::Table) -> usize {
        match  self.register_table(table) {
            Ok(idx) => {
                self.propagate(idx);
                idx
            },
            Err(idx) => idx
        }
    }

    fn propagate(&mut self, table_idx: usize) {
        println!("Propagating table {table_idx}");
        // Catagorize items by their next token
        let table: &table::Table = self.tables.get(table_idx).unwrap();
        let mut next_steps: BTreeMap<grammar::Token<T,N>, Vec<grammar::Item>> = BTreeMap::new();
        for item_ref in table.items().iter() {
            let mut item = item_ref.clone();
            if let Some(next_token) = item.step(self.grammar) {
                let v = next_steps.entry(*next_token).or_insert(Vec::new());
                v.push(item);
            }
        }

        // For each next token, create a table then add to builder
        for (step, items) in next_steps {
            // println!("Propagateing with token {step}");
            let table = table::Table::new_from(items, &self.closure_table, self.grammar);
            let idx = self.add_table(table);
            self.network.add_connection(step, table_idx, idx);
        }

    }

    pub fn into_parser(self) -> P {
        self.network
    }

    

}


impl<T: Display, N: Display, P: ParserBuild<T,N>> Builder<'_, T, N, P> {
    pub fn display(&self) {
        println!("Parsing States {{");
        for i in 0..self.tables.len() {
            println!("\t State {}: {}", i, self.tables[i].display(self.grammar)); // TODO: add display for the network
        }
        println!("}}");
    }
}
