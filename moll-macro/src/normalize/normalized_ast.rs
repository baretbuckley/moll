use proc_macro2::{Ident, TokenStream};
use std::fmt;


#[derive(Clone, PartialEq)]
pub enum Terminal {
    // Literal(usize), // ID representing the literal
    Rule(usize), // The index target rule in the parser
    Const(Ident), // Ident
    // Epsilon()
}

#[derive(Clone, PartialEq)]
pub enum Condition {
    Captured(Ident, Terminal),
    Terminal(Terminal)
}

pub struct Production {
    pub condition: Vec<Condition>,
    pub code_segment: TokenStream,
}

// #[derive(PartialEq)]
pub struct Rule {
    pub name: Ident,
    pub return_type: TokenStream,
    pub productions: Vec<Production>,
}

// #[derive(PartialEq)]
pub struct Parser {
    pub name: Ident,
    pub return_type: TokenStream,
    pub rules: Vec<Rule>,
}


impl fmt::Display for Parser {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parser \"{}\"", self.name);
        write!(f, " -> {}\n", self.return_type);
        for rule in self.rules.iter() {
            fmt_rule(rule, f, &self.rules);
        }
        Ok(())
    }
}

// Private helper functions for printing sub

fn fmt_rule(rule: &Rule, f: &mut fmt::Formatter<'_>, rule_set: &Vec<Rule>) {
    write!(f, "Rule \"{}\"", rule.name);
    write!(f, " -> {}\n", rule.return_type);
    for prod in rule.productions.iter() {
        fmt_prod(prod, f, rule_set);
        write!(f, "\n");
    }
}

fn fmt_prod(prod: &Production, f: &mut fmt::Formatter<'_>, rule_set: &Vec<Rule>) {
    for cond in prod.condition.iter() {
        fmt_cond(cond, f, rule_set);
        write!(f, " ");
    }
    write!(f, " {}", prod.code_segment);
}

fn fmt_cond(cond: &Condition, f: &mut fmt::Formatter, rule_set: &Vec<Rule>) {
    match cond {
        Condition::Captured(ident, terminal) => {
            write!(f, "{}:", ident);
            fmt_term(terminal, f, rule_set);
        },
        Condition::Terminal(terminal) => fmt_term(terminal, f, rule_set),
    }
}

fn fmt_term(term: &Terminal, f: &mut fmt::Formatter, rule_set: &Vec<Rule>) {
    match term {
        Terminal::Rule(rule_idx) => write!(f, "{}()", rule_set[*rule_idx].name),
        Terminal::Const(ident) => write!(f, "{}", ident),
        Terminal::Epsilon() => write!(f, "epsilon"),
    };
}