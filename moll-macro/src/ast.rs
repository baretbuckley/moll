use proc_macro2::{Group, Ident, Literal, TokenStream};


pub enum NameSpacedCondition {
    NameSpaced(Ident, Box<NameSpacedCondition>),
    Teminal(Ident),
    Rule(Ident),
    Literal(Literal),
}

// A condition that can't be captured
// This is due to the condition being contained in
// a operator that prevents it (e.g. '(a:A())*' is invalid)
pub enum FreeCondition {
    // Captured(Ident, CapturedCondition), not allowed >:(
    NameSpaced(Ident, NameSpacedCondition),
    Group(Vec<FreeCondition>),
    RepeatMinZero(Box<FreeCondition>),
    RepeatMinOne(Box<FreeCondition>),
    Optional(Box<FreeCondition>),
    Teminal(Ident),
    Rule(Ident),
    Literal(Literal),
}

pub enum CapturedCondition {
    NameSpaced(Ident, NameSpacedCondition),
    Group(Vec<CapturedCondition>),
    RepeatMinZero(Box<CapturedCondition>),
    RepeatMinOne(Box<CapturedCondition>),
    Optional(Box<CapturedCondition>),
    Teminal(Ident),
    Rule(Ident),
    Literal(Literal),
}

pub enum Condition {
    Captured(Ident, CapturedCondition),
    NameSpaced(Ident, NameSpacedCondition),
    Group(Vec<Condition>),
    RepeatMinZero(Box<FreeCondition>),
    RepeatMinOne(Box<FreeCondition>),
    Optional(Box<FreeCondition>),
    Teminal(Ident),
    Rule(Ident),
    Literal(Literal),
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

pub enum ParserElement {
    Rule(Rule),
    Group{name: Ident, ret: TokenStream, rules: Vec<ParserElement>},
}
pub struct Parser {
    pub name: Ident,
    pub return_type: TokenStream,
    pub parser_elements: Vec<ParserElement>,
}