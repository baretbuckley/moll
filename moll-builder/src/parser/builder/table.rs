use std::{collections::BTreeSet, fmt::Display, hash::Hash};

use super::closure_table::ClosureTable;
use crate::parser::grammar;


pub struct Table {
    items: BTreeSet<grammar::Item>,
}

impl<'a> Table {

    pub fn new_from<T, N: Copy+Eq+Hash, Iter: IntoIterator<Item = grammar::Item>>(root_state: Iter, closure_table: &ClosureTable<N>, grammar: &grammar::Grammar<T,N>) -> Self {
        let mut table: Table = Table { items: BTreeSet::new()};
        for item in root_state {
            table.items.insert(item);
        }
        table.closure(closure_table, grammar);
        table
    }

    pub fn new_from_ref<'b, T, N: Copy+Eq+Hash, Iter: IntoIterator<Item = &'b grammar::Item>>(root_state: Iter, closure_table: &ClosureTable<N>, grammar: &grammar::Grammar<T,N>) -> Self {
        let mut table: Table = Table { items: BTreeSet::new()};
        for item in root_state {
            table.items.insert(item.clone());
        }
        table.closure(closure_table, grammar);
        table
    }

    fn closure<T, N: Copy+Eq+Hash>(&mut self, closure_table: &ClosureTable<N>, grammar: &grammar::Grammar<T,N>) -> &mut Self {
        let mut change: bool = true;
        while change {
            let mut new_set: BTreeSet<grammar::Item> = BTreeSet::new();
            for item in self.items.iter() {
                if let Some(grammar::Token::NonTerminal(start_token)) = item.next(grammar) {
                    if let Some(item_set) = closure_table.get(*start_token) {
                        for new_item in item_set {
                            new_set.insert(new_item.clone());
                        }
                    }
                }
            }
            change = false;
            let old_len = self.items.len();
            self.items.extend(new_set);
            if self.items.len() > old_len {
                change = true;
            }
        }
        self
    }

    pub fn items(&self) -> &BTreeSet<grammar::Item> {&self.items}

    pub fn display<T: Display, N: Display>(&'a self, grammar: &'a grammar::Grammar<T,N>) -> DisplayTable<'a, T, N> {
        DisplayTable {grammar, table: self}
    }

}


impl Hash for Table {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.items.hash(state);
    }
}

impl PartialEq for Table {
    fn eq(&self, other: &Self) -> bool {
        self.items == other.items
    }
}

// impl Eq for Table {}

pub struct DisplayTable<'a, T, N> {
    grammar: &'a grammar::Grammar<T,N>,
    table: &'a Table,

}

impl<T: Display, N: Display> Display for DisplayTable<'_, T, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        _ = write!(f, "{{ ");
        for item in self.table.items.iter() {
            _ = write!(f, "{{{:}}} ", item.display(self.grammar));
        }
        write!(f, "}}")
    }
}
