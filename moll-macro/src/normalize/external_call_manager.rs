use std::collections::HashMap;
use proc_macro2::Ident;

use super::normalized_ast::Rule;

struct ExternalCallManager<T> {
    rules: HashMap<Ident, Rule<T>>,
}

impl<T> ExternalCallManager<T> {
    pub fn get_rule(name_space: Vec<Ident>, call: Ident) -> Option<Rule> {
        None
    }
}



