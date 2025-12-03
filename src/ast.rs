use proc_macro2::{Group, Ident, Literal, TokenStream};



pub enum Token {
    Literal(Literal), 
    Rule(Option<Ident>, TokenStream, Group)
}

pub struct Production {
    condition: Vec<Token>,
    code_segment: TokenStream,
}
pub enum Rule {
    Rule{ident: Ident, param: Group, ret: TokenStream, prod: Vec<Production>},
    Namespace(Ident, Vec<Rule>)
}