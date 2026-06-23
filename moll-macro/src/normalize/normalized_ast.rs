use std::marker::PhantomData;

use proc_macro2::{Ident, TokenStream};


pub enum Terminal {
    Literal(usize), // ID representing the literal
    Rule(usize), // The index target rule in the parser
    Const(Ident),
    Epsilon()
}

pub enum ProductionStep {
    Captured(Ident, Terminal),
    Terminal(Terminal)
}

pub struct Production {
    pub condition: Vec<ProductionStep>,
    pub code_segment: TokenStream,
}
pub struct Rule<T> {
    pub name: Ident,
    pub return_type: TokenStream,
    pub productions: Vec<Production>,
    _pd: PhantomData<T>,
}
pub struct Parser<T> {
    pub name: Ident,
    pub return_type: TokenStream,
    pub parser_elements: Vec<Rule<T>>,
}
