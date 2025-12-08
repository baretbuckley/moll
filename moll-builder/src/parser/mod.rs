


mod builder;
pub mod grammar;
pub mod runtime;
mod test;

pub trait ParserBuild<Term, NonTerm> {

    fn new() -> Self;

    fn new_state(&mut self) -> usize;

    fn add_connection(&mut self, condition: grammar::Token<Term,NonTerm>, from: usize, to: usize);

    fn new_reduction(&mut self, condition: grammar::Token<Term,NonTerm>, from: usize, rule: grammar::RuleIdx);

}

