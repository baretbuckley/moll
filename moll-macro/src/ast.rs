use proc_macro2::{Group, Ident, Literal, TokenStream};


pub enum NameSpacedProduction {
    NameSpaced(Ident, Box<NameSpacedProduction>),
    Teminal(Ident),
    Rule(Ident),
    Literal(Literal),
}

pub enum CapturedProduciton {
    NameSpaced(Ident, NameSpacedProduction),
    Group(Vec<CapturedProduciton>),
    RepeatMinZero(Box<CapturedProduciton>),
    RepeatMinOne(Box<CapturedProduciton>),
    Optional(Box<CapturedProduciton>),
    Teminal(Ident),
    Rule(Ident),
    Literal(Literal),
}

pub enum BaseProduction {
    Captured(Ident, CapturedProduciton),
    NameSpaced(Ident, NameSpacedProduction),
    Group(Vec<BaseProduction>),
    RepeatMinZero(Box<BaseProduction>),
    RepeatMinOne(Box<BaseProduction>),
    Optional(Box<BaseProduction>),
    Teminal(Ident),
    Rule(Ident),
    Literal(Literal),
}

pub struct Production {
    pub condition: Vec<BaseProduction>,
    pub code_segment: TokenStream,
}

pub struct Rule {
    pub name: Ident,
    pub return_type: TokenStream,
    pub productions: Vec<Production>,
}

pub enum ParserElement {
    Rule(Rule),
    Group{name: Ident, ret: TokenStream, rules: Vec<ParserElement>},
}
pub struct Parser {
    pub name: Ident,
    pub return_type: TokenStream,
    pub parser_elements: Vec<ParserElement>,
}