use std::{collections::HashMap, iter::zip};

// use crate::{ast::{BaseProduction, CapturedProduciton}, translate::normalize};

use super::ast as parse_ast;
mod normalized_ast;
use normalized_ast::{Parser, Rule, Production, ProductionStep, Terminal};
mod external_call_manager;

use proc_macro2::{Group, Ident, Literal, Span, TokenStream};
use quote::quote;


struct RuleManager<T> {
    // TODO: optimize this to not duplicate rule ident for hashmap
    rules: Vec<Rule<T>>,
    hash_idents: HashMap<Ident, usize>,
}

impl<T> RuleManager<T> {
    fn new() -> Self {
        RuleManager { rules: Vec::new(), hash_idents: HashMap::new() }
    }
    // Registers a rule with the manager, returning its index. 
    // Panics if a rule with the same name is already registered
    fn register(&mut self, rule: Rule<T>) -> usize {
        if self.hash_idents.contains_key(&rule.name) {
            panic!("Duplicate rule name: {}", rule.name);
        }
        let idx = self.rules.len();
        self.rules.push(rule);
        self.hash_idents.insert(rule.name, idx);
        idx
    }
    fn update(&mut self, idx: usize, rule: Rule<T>) {
        if idx >= self.rules.len() {
            panic!("Rule index out of bounds: {}", idx);
        }
        let old_ident = self.rules[idx].name;
        self.hash_idents.remove(&old_ident);
        
        self.rules[idx] = rule;
        if self.hash_idents.contains_key(&rule.name) {
            panic!("Duplicate rule name: {}", rule.name);
        }
        self.hash_idents.insert(rule.name, idx);
    }
    // Registers a rule if it does not exist, or updates the
    // existing rule the identifier is already registered.
    // Returns the index of the registered or updated rule.
    fn regester_or_update(&mut self, rule: Rule<T>) -> usize {
        match self.hash_idents.get(&rule.name) {
            Some(idx) => {
                self.update(*idx, rule);
                *idx
            },
            None => self.register(rule),
        }
    }
    // Reserves a slot for a rule in the manager, returning its index.
    // The reserved slot will have a placeholder rule with the name 
    // "Reserved{idx}" and an empty production list.
    // Can be set after using the update function.
    fn reserve(&mut self) -> usize {
        let idx = self.rules.len();
        self.rules.push(Rule::<T>{name: Ident::new(&format!("Reserved{}", idx), Span::call_site()), return_type: quote!{()}, productions: Vec::new()});
        idx
    }
    fn reserve_ident(&mut self, ident: &Ident) -> usize {
        if self.hash_idents.contains_key(ident) {
            panic!("Duplicate rule name: {}", ident);
        }
        let idx = self.rules.len();
        self.rules.push(Rule::<T>{name: *ident, return_type: quote!{()}, productions: Vec::new()});
        self.hash_idents.insert(*ident, idx);
        idx
    }
    fn get_rule_id(&self, ident: &Ident) -> usize {
        match self.hash_idents.get(ident) {
            Some(idx) => *idx,
            None => panic!("Rule not found: {}", ident),
        }
    }
    // Gets the index of a rule with the given ident,
    // reserving a new slot if it does not exist
    fn get_rule_id_or_reserve(&mut self, ident: &Ident) -> usize {
        match self.hash_idents.get(ident) {
            Some(idx) => *idx,
            None => self.reserve(),
        }
    }
    fn get_rule(&self, ident: &Ident) -> &Rule<T> {
        let idx = self.get_rule_idx(ident);
        &self.rules[idx]
    }
    fn len(&self) -> usize {
        self.rules.len()
    }
    fn dump_rules(&mut self) -> Vec<Rule<T>> {
        self.hash_idents.clear();
        self.rules.drain(..).collect()
    }

    fn get_rule_type(&self, id: usize) -> &TokenStream {
        &self.rules[id].return_type
    }
}

// pub enum Terminal<T> {
//     Literal(Vec<T>),
//     Rule(Ident),
//     Const(Ident),
// }

// pub enum ProductionStep<T> {
//     Captured(Ident, Terminal<T>),
//     Terminal(Terminal<T>)
// }


fn terminal_type<T>(terminal: &Terminal<T>, rule_manager: &RuleManager<T>) -> &TokenStream {
    // TODO: fix to return correct type for literals and consts
    match terminal {
        Terminal::Literal(_) => &quote!{Vec<T>},
        Terminal::Rule(idx) => rule_manager.get_rule_type(*idx),
        Terminal::Const(_) => &quote!{Ident},
        Terminal::Epsilon() => &quote!{()},
    }
}

// Generate a rule for a group of captured productions inserting it into
// the given Rule Manager. Returning a terminal referencing the generated rule 
fn generate_typed_group_rule<T, TM: TokenManager<T>>(captured: &Vec<CapturedProduciton>, TM: &mut TM, rule_manager: &mut RuleManager<T>) -> Terminal<T> {
    let mut normalized_tokens = Vec::new();

    // Assemble idents for the captured productions as "arg{idx}"
    // This is safe because each captured production will result in exactly one
    // normalized step, even if normalization results in rule generation
    let mut arg_idents = Vec::new();
    for idx in 0..captured.len() {
        let ident = Ident::new(&format!("arg{}", idx), Span::call_site());
        arg_idents.push(ident);
    }
    
    // Normalize captured productions steps
    for (step, ident) in zip(captured.iter(), arg_idents.iter()) {
        normalized_tokens.push(ProductionStep::Captured(*ident, normalize_captured(step, TM, rule_manager)));
    }

    // Construct generated Rule for the group
    let production = Production::<T> {
        condition: normalized_tokens,
        code_segment: quote!{(#(#arg_idents),*)},
    };
    let rule_name = Ident::new(&format!("InternalGroupRule{}", rule_manager.len()), Span::call_site());
    let types = normalized_tokens.iter().map(|step| match step {
        ProductionStep::Captured(_, terminal) => terminal_type(terminal, rule_manager),
        ProductionStep::Terminal(terminal) => panic!("Generated typed group should not have non-captured terminals"),
    }).collect::<Vec<_>>();
    
    let rule = Rule::<T> {
        name: rule_name,
        return_type: quote!{(#(#types),*)},
        productions: Vec::new()
    };

    // Register the rule with the manager and return its id
    let id = rule_manager.regester(rule);
    Terminal::Rule(id)

}

// Generate a rule for a repeated captured production inserting it into
// the given Rule Manager. Returning a terminal referencing the generated rule
fn generate_typed_repeat_rule<T, TM: TokenManager<T>>(captured: &CapturedProduciton, min: usize, TM: &mut TM, rule_manager: &mut RuleManager<T>) -> Terminal<T> {
    
    // Normalize the captured production
    let mut normalized_prod = normalize_captured(captured, TM, rule_manager);

    // Reserve a slot for the rule to get its id for use in recursive production conditions
    let rule_id = rule_manager.reserve();

    // Get type now so that the production step can be given to one of productions without clone
    let rule_type = terminal_type(&normalized_prod, rule_manager);

    // Construct terminal production for the repeat rule based on the min value
    // for min=0: S -> ε
    // for min=1: S -> P
    let terminal_prod = match min {
        0 => Production::<T> {
            condition: vec![ProductionStep::Terminal(Terminal::Epsilon())],
            code_segment: quote!{Vec::new()},
        },
        1 => Production::<T> {
            condition: vec![ProductionStep::Captured(Ident::new("term", Span::call_site()), normalized_prod.clone())],
            code_segment: quote!{vec![term]},
        },
        _ => panic!("Unsupported min value for repeat: {}", min),
    };

    // Construct repeat production for the repeat rule
    // S -> P S
    let repeat_prod = Production::<T> {
        condition: vec![
            ProductionStep::Captured(Ident::new("new", Span::call_site()), normalized_prod),
            ProductionStep::Captured(Ident::new("v", Span::call_site()), Terminal::Rule(rule_id)),],
        code_segment: quote!{{
            v.push(new);
            v
        }},
    };

    // Construct rule from the productions and update the reserved rule in the manager
    let rule_name = Ident::new(&format!("InternalRepeatRule{}", rule_id), Span::call_site());
    let rule = Rule::<T> {
        name: rule_name,
        return_type: quote!{Vec<#rule_type>},
        productions: vec![terminal_prod, repeat_prod],
    };
    rule_manager.update(rule_id, rule);
    
    Terminal::Rule(rule_id)
}

// Generate a rule for a repeated captured production inserting it into
// the given Rule Manager. Returning a terminal referencing the generated rule
// Resulting rule does not return a value nor does it execute any code,
// it is only used for parsing the repeated production
fn generate_untyped_repeat_rule<T>(repeated_condition: &BaseProduction, min: usize, TM: &mut TM, rule_manager: &mut RuleManager<T>) -> Terminal<T> {
    
    // Recursively normalize the production steps in the repeated condition
    let normalized_prod = Vec::new();
    normalize_condition(repeated_condition, normalized_prod, TM, rule_manager, );
    
    // convert normalized_prod to a vec of terminals for the production condition
    let normalized_terminals = normalized_prod.into_iter().map(ProductionStep::Terminal).collect();

    // Reserve a slot for the rule to get its id for use in recursive production conditions
    let rule_id = rule_manager.reserve();

    // Construct terminal production for the repeat rule based on the min value
    // for min=0: S -> ε
    // for min=1: S -> P
    let terminal_prod = match min {
        0 => Production::<T> {
            condition: vec![ProductionStep::Terminal(Terminal::Epsilon())],
            code_segment: quote!{{}},
        },
        1 => Production::<T> {
            condition: vec![ProductionStep::Terminal(normalized_prod.clone())],
            code_segment: quote!{{}},
        },
        _ => panic!("Unsupported min value for repeat: {}", min),
    };

    // Construct repeat production for the repeat rule
    // S -> P S
    let repeat_prod = Production::<T> {
        condition: normalized_terminals.push(
            ProductionStep::Terminal(Terminal::Rule(rule_id))),
        code_segment: quote!{{}},
    };

    let rule_name = Ident::new(&format!("InternalRepeatRule{}", rule_id), Span::call_site());
    let rule = Rule::<T> {
        name: rule_name,
        return_type: quote!{()},
        productions: vec![terminal_prod, repeat_prod],
    };
    rule_manager.update(rule_id, rule);
    Terminal::Rule(rule_id)

}

fn generate_typed_optional_rule<T, TM: TokenManager<T>>(captured: &CapturedProduciton, TM: &mut TM, rule_manager: &mut RuleManager<T>) -> Terminal<T> {
    // Normalize the captured production
    let normalized_prod = normalize_captured(captured, TM, rule_manager);

    let rule_id = rule_manager.reserve();

    // Get type now so that the production step can be given to one of productions without clone
    let rule_type = terminal_type(&normalized_prod, rule_manager);

    // Construct productions for the optional rule
    // S -> ε
    // S -> P
    let epsilon_prod = Production::<T> {
        condition: vec![ProductionStep::Terminal(Terminal::Epsilon())],
        code_segment: quote!{None},
    };
    let prod = Production::<T> {
        condition: vec![ProductionStep::Captured(Ident::new("term", Span::call_site()), normalized_prod)],
        code_segment: quote!{Some(term)},
    };

    // Construct rule from the productions and update the reserved rule in the manager
    let rule_name = Ident::new(&format!("InternalOptionalRule{}", rule_id), Span::call_site());
    let rule = Rule::<T> {
        name: rule_name,
        return_type: quote!{Option<#rule_type>},
        productions: vec![epsilon_prod, prod],
    };
    rule_manager.update(rule_id, rule);
    
    Terminal::Rule(rule_id)
}

fn generate_untyped_optional_rule<T, TM: TokenManager>(
    condition: &BaseProduction,
    TM: &mut TM,
    ECM: ExternalCallManager,
    rule_manager: &mut RuleManager<T>,
) -> Terminal<T> {
    // recursively normalize the production steps in the condition
    let normalized_prod: Vec<ProductionStep<T>> = Vec::new();
    normalize_condition(condition, &normalized_prod, TM, ECM, rule_manager);
    
    // convert normalized_prod to a vec of terminals for the production condition
    let normalized_terminals = normalized_prod.into_iter().map(ProductionStep::Terminal).collect();

    // Construct productions for the optional rule
    // S -> ε
    // S -> P
    let epsilon_prod = Production::<T> {
        condition: vec![ProductionStep::Terminal(Terminal::Epsilon())],
        code_segment: quote!{{}},
    };
    let prod = Production::<T> {
        condition: normalized_terminals,
        code_segment: quote!{{}},
    };

    let rule_id = rule_manager.reserve();
    let rule_name = Ident::new(&format!("InternalOptionalRule{}", rule_id), Span::call_site());
    let rule = Rule::<T> {
        name: rule_name,
        return_type: quote!{()},
        productions: vec![epsilon_prod, prod],
    };
    rule_manager.update(rule_id, rule);
    Terminal::Rule(rule_id)
}

fn normalize_captured<T, TM: TokenManager<T>>(captured: &ast::CapturedProduciton, TM: &mut TM, rule_manager: &mut RuleManager<T>) -> Terminal<T> {
    match captured {
        ast::CapturedProduciton::NameSpaced(ident, name_spaced_production) => {
            todo!()
        },
        ast::CapturedProduciton::Group(captured_producitons) => {
            generate_typed_group_rule(captured_producitons, TM, rule_manager)
        },
        ast::CapturedProduciton::RepeatMinZero(captured_produciton) => {
            generate_typed_repeat_rule(captured_produciton, 0, TM, rule_manager)
        },
        ast::CapturedProduciton::RepeatMinOne(captured_produciton) => {
            generate_typed_repeat_rule(captured_produciton, 1, TM, rule_manager)
        },
        ast::CapturedProduciton::Optional(captured_produciton) => {
            generate_typed_optional_rule(captured_produciton, TM, rule_manager)
        },
        ast::CapturedProduciton::Teminal(ident) => {
            Terminal::Const(*ident)
        },
        ast::CapturedProduciton::Rule(ident) => {
            Terminal::Rule(rule_manager.get_rule_id_or_reserve(ident))
        },
        ast::CapturedProduciton::Literal(literal) => {
            Terminal::Literal(TM.fromLiteral(literal))
        },
    }
}

fn normalize_condition<T, TM: TokenManager<T>>(
    condition: &ast::BaseProduction, 
    normalized_tokens: &Vec<ProductionStep<T>>,
    TM: &mut TM, ECM: ExternalCallManager, 
    rule_manager: &mut RuleManager<T>
) {
    match condition {
        ast::BaseProduction::Captured(ident, captured_produciton) => {
            normalized_tokens.push(ProductionStep::<T>::Captured(*ident, normalize_captured(*captured_produciton)));
        },
        ast::BaseProduction::NameSpaced(ident, name_spaced_production) => {

        },
        ast::BaseProduction::Group(base_productions) => {
            for step in base_productions {
                normalize_condition::<T,TM>(step, normalized_tokens, TM, ECM, rule_manager);
            }
        },
        ast::BaseProduction::RepeatMinZero(base_production) => {
            let terminal = generate_untyped_repeat_rule(&**base_production, 0, TM, rule_manager);
            normalized_tokens.push(ProductionStep::Terminal(terminal))
        },
        ast::BaseProduction::RepeatMinOne(base_production) => {
            let terminal = generate_untyped_repeat_rule(&**base_production, 1, TM, rule_manager)
            normalized_tokens.push(ProductionStep::Terminal(terminal))
        },
        ast::BaseProduction::Optional(base_production) => {
            let terminal = generate_untyped_optional_rule(&**base_production, TM, ECM, rule_manager)
            normalized_tokens.push(ProductionStep::Terminal(terminal))
        },
        ast::BaseProduction::Teminal(ident) => {
            normalized_tokens.push(ProductionStep::Terminal(Terminal::Const(*ident)))
        },
        ast::BaseProduction::Rule(ident) => {
            let terminal = Terminal::Rule(rule_manager.get_rule_id_or_reserve(ident));
            normalized_tokens.push(ProductionStep::Terminal(terminal))
        },
        ast::BaseProduction::Literal(literal) => {
            let terminal = Terminal::Literal(TM.fromLiteral(literal));
            normalized_tokens.push(ProductionStep::Terminal(terminal))
        },
    }
}

fn normalize_production<T, TM: TokenManager<T>>(src_prod: &mut ast::Production, token_manager: &mut TM) -> Token<T> {
    
    for cond in src_prod.condition {

    }
}

fn normalize_rule<T>(src_rule: &mut ast::Rule) {
    for prod in src_rule.productions {

    }
}

fn normalize_parser_element<T>(element: &mut ast::ParserElement, rules: &mut Vec<Rule<T>>, prefix: &mut String) {
    match element {
        ast::ParserElement::Rule(rule) => return,
        ast::ParserElement::Group{name, ret, rules} => {
            for elem in rules {
                normalize_parser_element(elem);
            }
        },
    }
}

pub fn normalize_ast<Token>(ast: &mut ast::Parser) -> &mut Parser {
    let mut ast_out = Parser::<Token>{name: ast.name, return_type: ast.return_type, parser_elements: vec::new()};
    for element in ast.parser_elements {
        
    }
    ast
}