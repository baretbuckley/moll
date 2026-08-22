use std::marker::PhantomData;

use proc_macro2::{Ident, TokenStream};


#[derive(Clone)]
pub enum Terminal {
    Literal(usize), // ID representing the literal
    Rule(usize), // The index target rule in the parser
    Const(Ident), // Ident
    Epsilon()
}

#[derive(Clone)]
pub enum Condition {
    Captured(Ident, Terminal),
    Terminal(Terminal)
}

pub struct Production {
    pub condition: Vec<Condition>,
    pub code_segment: TokenStream,
}
pub struct Rule {
    pub name: Ident,
    pub return_type: TokenStream,
    pub productions: Vec<Production>,
}
pub struct Parser {
    pub name: Ident,
    pub return_type: TokenStream,
    pub parser_elements: Vec<Rule>,
}
