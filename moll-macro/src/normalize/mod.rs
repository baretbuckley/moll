use std::{collections::HashMap, iter::zip, unreachable};

// use crate::{ast::{BaseProduction, CapturedProduciton}, translate::normalize};

use super::ast as parse_ast;
pub mod normalized_ast;
use normalized_ast::{Parser, Rule, Production, Condition, Terminal};

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;


struct RuleManager {
    // TODO: optimize this to not duplicate rule ident for hashmap
    rules: Vec<Rule>,
    hash_idents: HashMap<Ident, usize>,

    // TMP, this really shouldn't be needed, but, I rust don't want to give me static tokenstreams :(
    // literal_tokenstream: TokenStream,
    const_tokenstream: TokenStream,
}

impl RuleManager {
    fn new() -> Self {
        RuleManager { 
            rules: Vec::new(), 
            hash_idents: HashMap::new(),
            // literal_tokenstream: quote!{Vec<T>},
            const_tokenstream: quote!{Ident},
        }
    }
    // Registers a rule with the manager, returning its index. 
    // Panics if a rule with the same name is already registered
    fn register(&mut self, rule: Rule) -> usize {
        if self.hash_idents.contains_key(&rule.name) {
            panic!("Duplicate rule name: {}", rule.name);
        }
        let idx = self.rules.len();
        self.hash_idents.insert(rule.name.clone(), idx);
        self.rules.push(rule);
        idx
    }
    // Update 
    fn update(&mut self, idx: usize, rule: Rule) {
        // Confirm idx is in bounds
        if idx >= self.rules.len() {
            panic!("Rule index out of bounds: {}", idx);
        }
        // Get a reference to the rule, and save the old rule's ident
        let rule_ref = &mut self.rules[idx];
        let old_ident = &rule_ref.name;

        print!("Updating rule {}, old: {}, new: {}\n", idx, old_ident, rule.name);

        // Attempt to remove the old rule from 'hash_idents', panic if it fails to find it
        _ = self.hash_idents.remove(old_ident);
        
        // Update the value to rule in the vector and insert it in hash_idents with the new name
        *rule_ref = rule;
        self.hash_idents.insert(rule_ref.name.clone(), idx);
    }
    
    // Registers a rule if it does not exist, or updates the
    // existing rule the identifier is already registered.
    // Returns the index of the registered or updated rule.
    fn regester_or_update(&mut self, rule: Rule) -> usize {
        match self.hash_idents.get(&rule.name) {
            Some(idx) => {
                // Note: rule was found by name, so that doesn;t need to be updated
                // just the rule value
                let idx_tmp = *idx;

                self.rules[*idx] = rule;
                idx_tmp
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
        print!("Reserved rule at idx: {}\n", idx);
        self.rules.push( Rule {
                name: Ident::new("Reserved", Span::call_site()), 
                return_type: quote!{()}, 
                productions: Vec::new()
        } );
        idx
    }
    fn reserve_ident(&mut self, ident: Ident) -> usize {
        if self.hash_idents.contains_key(&ident) {
            panic!("Duplicate rule name: {}", ident);
        }
        let idx = self.rules.len();
        self.rules.push(Rule{name: ident.clone(), return_type: quote!{()}, productions: Vec::new()});
        self.hash_idents.insert(ident, idx);
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
            None => self.reserve_ident(ident.clone()),
        }
    }
    fn get_rule(&self, idx: usize) -> &Rule {
        &self.rules[idx]
    }
    fn get_rule_mut(&mut self, idx: usize) -> &mut Rule {
        &mut self.rules[idx]
    }
    fn len(&self) -> usize {
        self.rules.len()
    }
    fn dump_rules(&mut self) -> Vec<Rule> {
        self.hash_idents.clear();
        self.rules.drain(..).collect()
    }

    fn get_rule_type(&self, id: usize) -> &TokenStream {
        &self.rules[id].return_type
    }
}


// static LiteralConst: TokenStream = quote!{Vec<T>};
fn terminal_type<'a>(terminal: &Terminal, rule_manager: &'a RuleManager) -> &'a TokenStream {
    // TODO: fix to return correct type for literals and consts
    match terminal {
        // Terminal::Literal(_) => &rule_manager.literal_tokenstream,
        Terminal::Rule(idx) => rule_manager.get_rule_type(*idx),
        Terminal::Const(_) => &rule_manager.const_tokenstream,
        // Terminal::Epsilon() => &rule_manager.epsilon_tokenstream,
    }
}

// Generate a rule for a group of captured productions inserting it into
// the given Rule Manager. Returning a terminal referencing the generated rule 
fn generate_typed_group_rule(captured: Vec<parse_ast::CapturedCondition>, rule_manager: &mut RuleManager) -> Terminal {
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
    for (step, ident) in zip(captured.into_iter(), arg_idents.iter()) {
        normalized_tokens.push(Condition::Captured((*ident).clone(), normalize_captured(step, rule_manager)));
    }

    // Construct generated Rule for the group
    
    let rule_name = Ident::new(&format!("InternalGroupRule{}", rule_manager.len()), Span::call_site());
    let types = normalized_tokens.iter().map(|step| match step {
        Condition::Captured(_, terminal) => terminal_type(terminal, rule_manager),
        Condition::Terminal(_) => panic!("Generated typed group should not have non-captured terminals"),
    }).collect::<Vec<_>>();
    let production = Production {
        condition: normalized_tokens,
        code_segment: quote!{{(#(#arg_idents),*)}},
    };

    let rule = Rule {
        name: rule_name,
        return_type: quote!{(#(#types),*)},
        productions: vec![production]
    };

    // Register the rule with the manager and return its id
    let id = rule_manager.register(rule);
    Terminal::Rule(id)

}

// Generate a rule for a repeated captured production inserting it into
// the given Rule Manager. Returning a terminal referencing the generated rule
fn generate_typed_repeat_rule(captured: parse_ast::CapturedCondition, min: usize, rule_manager: &mut RuleManager) -> Terminal {
    
    // Normalize the captured production
    let normalized_prod = normalize_captured(captured, rule_manager);

    // Reserve a slot for the rule to get its id for use in recursive production conditions
    let rule_id = rule_manager.reserve();

    // Get type now so that the production step can be given to one of productions without clone
    let rule_type = terminal_type(&normalized_prod, rule_manager);

    // Construct terminal production for the repeat rule based on the min value
    // for min=0: S -> ε
    // for min=1: S -> P
    let terminal_prod = match min {
        0 => Production {
            condition: Vec::new(),
            // condition: vec![Condition::Terminal(Terminal::Epsilon())],
            code_segment: quote!{{Vec::new()}},
        },
        1 => Production {
            condition: vec![Condition::Captured(Ident::new("term", Span::call_site()), normalized_prod.clone())],
            code_segment: quote!{{vec![term]}},
        },
        _ => panic!("Unsupported min value for repeat: {}", min),
    };

    // Construct repeat production for the repeat rule
    // S -> P S
    let repeat_prod = Production {
        condition: vec![
            Condition::Captured(Ident::new("new", Span::call_site()), normalized_prod),
            Condition::Captured(Ident::new("v", Span::call_site()), Terminal::Rule(rule_id)),],
        code_segment: quote!{{
            v.push(new);
            v
        }},
    };

    // Construct rule from the productions and update the reserved rule in the manager
    let rule_name = Ident::new(&format!("InternalRepeatRule{}", rule_id), Span::call_site());
    let rule = Rule {
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
fn generate_untyped_repeat_rule(repeated_condition: parse_ast::FreeCondition, min: usize, rule_manager: &mut RuleManager) -> Terminal {
    
    // Recursively normalize the production steps in the repeated condition
    let mut normalized_conds = Vec::new();
    normalize_free_condition(repeated_condition, &mut normalized_conds, rule_manager);

    // Reserve a slot for the rule to get its id for use in recursive production conditions
    let rule_id = rule_manager.reserve();

    // Construct terminal production for the repeat rule based on the min value
    // for min=0: S -> ε
    // for min=1: S -> P
    let terminal_prod = match min {
        0 => Production {
            condition: Vec::new(),
            // condition: vec![Condition::Terminal(Terminal::Epsilon())],
            code_segment: quote!{{}},
        },
        1 => Production {
            condition: normalized_conds.clone(),
            code_segment: quote!{{}},
        },
        _ => panic!("Unsupported min value for repeat: {}", min),
    };

    // Construct repeat production for the repeat rule
    // S -> P S
    normalized_conds.push(Condition::Terminal(Terminal::Rule(rule_id)));
    let repeat_prod = Production {
        condition: normalized_conds,
        code_segment: quote!{{}},
    };

    let rule_name = Ident::new(&format!("InternalRepeatRule{}", rule_id), Span::call_site());
    let rule = Rule {
        name: rule_name,
        return_type: quote!{()},
        productions: vec![terminal_prod, repeat_prod],
    };
    rule_manager.update(rule_id, rule);
    Terminal::Rule(rule_id)

}

fn generate_typed_optional_rule(captured: parse_ast::CapturedCondition, rule_manager: &mut RuleManager) -> Terminal {
    // Normalize the captured production
    let normalized_prod = normalize_captured(captured, rule_manager);

    let rule_id = rule_manager.reserve();

    // Get type now so that the production step can be given to one of productions without clone
    let rule_type = terminal_type(&normalized_prod, rule_manager);

    // Construct productions for the optional rule
    // S -> ε
    // S -> P
    let epsilon_prod = Production {
        condition: Vec::new(),
        // condition: vec![Condition::Terminal(Terminal::Epsilon())],
        code_segment: quote!{None},
    };
    let prod = Production {
        condition: vec![Condition::Captured(Ident::new("term", Span::call_site()), normalized_prod)],
        code_segment: quote!{Some(term)},
    };

    // Construct rule from the productions and update the reserved rule in the manager
    let rule_name = Ident::new(&format!("InternalOptionalRule{}", rule_id), Span::call_site());
    let rule = Rule {
        name: rule_name,
        return_type: quote!{Option<#rule_type>},
        productions: vec![epsilon_prod, prod],
    };
    rule_manager.update(rule_id, rule);
    
    Terminal::Rule(rule_id)
}

fn generate_untyped_optional_rule(
    condition: parse_ast::FreeCondition,
    rule_manager: &mut RuleManager,
) -> Terminal {
    // recursively normalize the production steps in the condition
    let mut normalized_conds: Vec<Condition> = Vec::new();
    normalize_free_condition(condition, &mut normalized_conds, rule_manager);
    
    // convert normalized_prod to a vec of terminals for the production condition
    // let normalized_terminals = normalized_prod.into_iter().map(ProductionStep::Terminal).collect();

    // Construct productions for the optional rule
    // S -> ε
    // S -> P
    let epsilon_prod = Production {
        condition: Vec::new(),
        // condition: vec![Condition::Terminal(Terminal::Epsilon())],
        code_segment: quote!{{}},
    };
    let prod = Production {
        condition: normalized_conds,
        code_segment: quote!{{}},
    };

    let rule_id = rule_manager.reserve();
    let rule_name = Ident::new(&format!("InternalOptionalRule{}", rule_id), Span::call_site());
    let rule = Rule {
        name: rule_name,
        return_type: quote!{()},
        productions: vec![epsilon_prod, prod],
    };
    rule_manager.update(rule_id, rule);
    Terminal::Rule(rule_id)
}

fn normalize_captured(captured: parse_ast::CapturedCondition, rule_manager: &mut RuleManager) -> Terminal {
    match captured {
        parse_ast::CapturedCondition::NameSpaced(ident, name_spaced_production) => {
            _ = ident;
            _ = name_spaced_production;
            panic!("Namespaces not yet supported");
            unreachable!();
        },
        parse_ast::CapturedCondition::Group(captured_producitons) => {
            generate_typed_group_rule(captured_producitons, rule_manager)
        },
        parse_ast::CapturedCondition::RepeatMinZero(captured_produciton) => {
            generate_typed_repeat_rule(*captured_produciton, 0, rule_manager)
        },
        parse_ast::CapturedCondition::RepeatMinOne(captured_produciton) => {
            generate_typed_repeat_rule(*captured_produciton, 1, rule_manager)
        },
        parse_ast::CapturedCondition::Optional(captured_produciton) => {
            generate_typed_optional_rule(*captured_produciton, rule_manager)
        },
        parse_ast::CapturedCondition::Terminal(ident) => {
            Terminal::Const(ident.clone())
        },
        parse_ast::CapturedCondition::Rule(ident) => {
            Terminal::Rule(rule_manager.get_rule_id_or_reserve(&ident))
        },
        parse_ast::CapturedCondition::Literal(literal) => {
            // Terminal::Literal(TM.fromLiteral(literal))
            _ = literal;
            panic!("Literal not yet supported");
            unreachable!();
        },
    }
}

fn normalize_free_condition(
    condition: parse_ast::FreeCondition, 
    normalized_tokens: &mut Vec<Condition>,
    // TM: &mut TM, ECM: ExternalCallManager, 
    rule_manager: &mut RuleManager
) {
    match condition {
        // parse_ast::FreeCondition::Captured(ident, captured_produciton) => {
        //     normalized_tokens.push(Condition::Captured(*ident, normalize_captured(captured_produciton, rule_manager)));
        // },
        parse_ast::FreeCondition::NameSpaced(ident, name_spaced_production) => {
            _ = ident;
            _ = name_spaced_production;
        },
        parse_ast::FreeCondition::Group(base_productions) => {
            for step in base_productions {
                normalize_free_condition(step, normalized_tokens, rule_manager);
            }
        },
        parse_ast::FreeCondition::RepeatMinZero(base_production) => {
            let terminal = generate_untyped_repeat_rule(*base_production, 0, rule_manager);
            normalized_tokens.push(Condition::Terminal(terminal))
        },
        parse_ast::FreeCondition::RepeatMinOne(base_production) => {
            let terminal = generate_untyped_repeat_rule(*base_production, 1, rule_manager);
            normalized_tokens.push(Condition::Terminal(terminal))
        },
        parse_ast::FreeCondition::Optional(base_production) => {
            let terminal = generate_untyped_optional_rule(*base_production, rule_manager);
            normalized_tokens.push(Condition::Terminal(terminal))
        },
        parse_ast::FreeCondition::Terminal(ident) => {
            normalized_tokens.push(Condition::Terminal(Terminal::Const(ident.clone())))
        },
        parse_ast::FreeCondition::Rule(ident) => {
            let terminal = Terminal::Rule(rule_manager.get_rule_id_or_reserve(&ident));
            normalized_tokens.push(Condition::Terminal(terminal))
        },
        parse_ast::FreeCondition::Literal(literal) => {
            // let terminal = Terminal::Literal(TM.fromLiteral(literal));
            // normalized_tokens.push(Condition::Terminal(terminal))
            _ = literal;
            panic!("Literal not yet supported");
        },
    }
}

fn normalize_condition(
    condition: parse_ast::Condition, 
    normalized_tokens: &mut Vec<Condition>,
    // TM: &mut TM, ECM: ExternalCallManager, 
    rule_manager: &mut RuleManager
) {
    match condition {
        parse_ast::Condition::Captured(ident, captured_produciton) => {
            normalized_tokens.push(Condition::Captured(ident, normalize_captured(captured_produciton, rule_manager)));
        },
        parse_ast::Condition::NameSpaced(ident, name_spaced_production) => {
            _ = ident;
            _ = name_spaced_production
        },
        parse_ast::Condition::Group(base_productions) => {
            for step in base_productions {
                normalize_condition(step, normalized_tokens, rule_manager);
            }
        },
        parse_ast::Condition::RepeatMinZero(base_production) => {
            let terminal = generate_untyped_repeat_rule(*base_production, 0, rule_manager);
            normalized_tokens.push(Condition::Terminal(terminal))
        },
        parse_ast::Condition::RepeatMinOne(base_production) => {
            let terminal = generate_untyped_repeat_rule(*base_production, 1, rule_manager);
            normalized_tokens.push(Condition::Terminal(terminal))
        },
        parse_ast::Condition::Optional(base_production) => {
            let terminal = generate_untyped_optional_rule(*base_production, rule_manager);
            normalized_tokens.push(Condition::Terminal(terminal))
        },
        parse_ast::Condition::Terminal(ident) => {
            normalized_tokens.push(Condition::Terminal(Terminal::Const(ident)))
        },
        parse_ast::Condition::Rule(ident) => {
            let terminal = Terminal::Rule(rule_manager.get_rule_id_or_reserve(&ident));
            normalized_tokens.push(Condition::Terminal(terminal))
        },
        parse_ast::Condition::Literal(literal) => {
            // let terminal = Terminal::Literal(TM.fromLiteral(literal));
            // normalized_tokens.push(Condition::Terminal(terminal))
            _ = literal;
            panic!("Literal not yet supported");
        },
    }
}

fn normalize_production(src_prod: parse_ast::Production, rule_manager: &mut RuleManager) -> Production {
    let mut normalized_conditions = Vec::new();
    for cond in src_prod.condition {
        normalize_condition(cond, &mut normalized_conditions, rule_manager);
    }

    Production { 
        condition: normalized_conditions,
        code_segment: src_prod.code_segment
    }
}

fn normalize_rule(src_rule: parse_ast::Rule, rule_manager: &mut RuleManager) {
    let mut norm_prods = Vec::with_capacity(src_rule.productions.len());
    for prod in src_rule.productions {
        norm_prods.push(normalize_production(prod, rule_manager));
    }
    rule_manager.regester_or_update(
        Rule {
            name: src_rule.name,
            return_type: src_rule.return_type,
            productions: norm_prods,
        }
    );
}

fn initialize_rule(name: Ident, return_type: TokenStream, rule_manager: &mut RuleManager) -> usize {
    rule_manager.register(
        Rule {
            name: name,
            return_type: return_type,
            productions: Vec::new(),
        }
    )
}

fn fill_rule(rule_id: usize, prods: Vec<parse_ast::Production>, rule_manager: &mut RuleManager) {
    let mut norm_prods = Vec::with_capacity(prods.len());
    for prod in prods {
        norm_prods.push(normalize_production(prod, rule_manager));
    }
    let rule_ref = rule_manager.get_rule_mut(rule_id);
    rule_ref.productions = norm_prods;
}

fn initialize_parser_element(element: parse_ast::ParserElement, rule_manager: &mut RuleManager, cached_prods: &mut Vec<(Vec<parse_ast::Production>, usize)>) {
    match element {
        parse_ast::ParserElement::Rule(rule) => {
            let id = initialize_rule(rule.name, rule.return_type, rule_manager);
            cached_prods.push((rule.productions, id));
        },  
        parse_ast::ParserElement::Group{name, ret, rules} => {
            _ = name;
            _ = ret;
            _ = rules;
            panic!("We don't do groups yet, please change your mind :)");
        },
    }
}

pub fn normalize_ast(parser: parse_ast::Parser) -> Parser {
    let mut rule_manager = RuleManager::new();

    // Note: Rule normalization is split into two steps: 
    //  initialization: regester each rule with its name and return type
    //  filling: normalize the rules productions and add it to the rule
    // The split allows the types of rules to be accessed before they are normalized

    // Cached prods while initializing rules with name and return type
    // This prevents needing to index through the Parse elements tree twice, which would force cloning the name and return types
    let mut cached_prods: Vec<(Vec<parse_ast::Production>, usize)> = Vec::new();
   
    // Initialize all normalized rules with a name and return type (without productions)
    for element in parser.parser_elements {
        initialize_parser_element(element, &mut rule_manager, &mut cached_prods);
    }

    // normalize the cached productions and fill the rules with them
    for (prods, rule_id) in cached_prods {
        fill_rule(rule_id, prods, &mut rule_manager);
    }

    Parser {
        name: parser.name,
        return_type: parser.return_type,
        rules: rule_manager.dump_rules(),
    }
}



#[cfg(test)]
mod tests {
    use super::*;



    #[test]
    fn captured_repeat() {
        // S -> a:(A()*) ( "ident" C() ) {return a.len();} | "term" {return 0;} -> i32 
        // A -> std::String
        // C -> u8
       
        // conditions contained within the 
        let captured_a_star = parse_ast::CapturedCondition::Rule(Ident::new("A", Span::call_site()));
       
        let parsed_ast = parse_ast::Parser {
            name: Ident::new("parser_name", Span::call_site()),
            return_type: quote!{Vec<i32>},
            parser_elements: vec![
                parse_ast::ParserElement::Rule (parse_ast::Rule {
                    name: Ident::new("S", Span::call_site()),
                    return_type: quote!{i32},
                    productions: vec![
                        // S -> a:(A()*) ( "ident" C() ) {return a.len();}
                        parse_ast::Production {
                            code_segment: quote!{{return a.len();}},
                            condition: vec![
                                parse_ast::Condition::Captured(Ident::new("a", Span::call_site()),
                                    parse_ast::CapturedCondition::RepeatMinZero(Box::new(captured_a_star))
                                ),
                                parse_ast::Condition::Group(vec![
                                    parse_ast::Condition::Terminal(Ident::new("ident", Span::call_site())),
                                    parse_ast::Condition::Rule(Ident::new("C", Span::call_site())),
                                ])
                            ]
                        },
                        // S -> "term" {return 0;}
                        parse_ast::Production {
                            code_segment: quote!{{return 0;}},
                            condition: vec![
                                parse_ast::Condition::Terminal(Ident::new("term", Span::call_site())),
                            ]
                        }
                    ],
                }),
                parse_ast::ParserElement::Rule (parse_ast::Rule {
                    name: Ident::new("A", Span::call_site()),
                    return_type: quote!{std::String},
                    productions: vec![],
                }),
                parse_ast::ParserElement::Rule (parse_ast::Rule {
                    name: Ident::new("C", Span::call_site()),
                    return_type: quote!{u8},
                    productions: vec![],
                }),
                
            ],
        };

        let target_ast = Parser {
            name: Ident::new("parser_name", Span::call_site()),
            return_type: quote!{Vec<i32>},
            rules: vec![
                Rule {
                    name: Ident::new("S", Span::call_site()),
                    return_type: quote!{i32},
                    productions: vec![
                        Production {
                            // S -> a:InternalRepeatRule3() "ident" C() {return a.len();}
                            condition: vec![
                                Condition::Captured(Ident::new("a", Span::call_site()), Terminal::Rule(3)), // S:InternalRepeatRule1
                                Condition::Terminal(Terminal::Const(Ident::new("ident", Span::call_site()))),
                                Condition::Terminal(Terminal::Rule(2)),
                            ],
                            code_segment: quote!{{return a.len();}}
                        },
                        // S -> "term" {return 0;}
                        Production {
                            code_segment: quote!{{return 0;}},
                            condition: vec![
                                Condition::Terminal(Terminal::Const(Ident::new("term", Span::call_site()))),
                            ]
                        }
                    ]
                },
                Rule {
                    name: Ident::new("A", Span::call_site()),
                    return_type: quote!{std::String},
                    productions: vec![]
                },
                Rule {
                    name: Ident::new("C", Span::call_site()),
                    return_type: quote!{u8},
                    productions: vec![]
                },
                Rule {
                    name: Ident::new("InternalRepeatRule3", Span::call_site()),
                    return_type: quote!{Vec<std::String>},
                    productions: vec![
                        // InternalRepeatRule3 -> epsilon {Vec::new()}
                        Production {
                            condition: Vec::new(),
                            // vec![
                            //     Condition::Terminal(Terminal::Epsilon())
                            // ],
                            code_segment: quote!{{Vec::new()}}
                        },
                        // InternalRepeatRule3 -> new:A() v:InternalRepeatRule3() {c.push(new); v}
                        Production {
                            condition: vec![
                                Condition::Captured(Ident::new("new", Span::call_site()), Terminal::Rule(1)), // new:A()
                                Condition::Captured(Ident::new("v", Span::call_site()), Terminal::Rule(3)), // v:InternalRepeatRule3()
                            ],
                            code_segment: quote!{{v.push(new); v}}
                        },
                    ]
                }
            ]
        };

        let normalized_ast = normalize_ast(parsed_ast);

        print!("{}\n", normalized_ast);
        print!("{}\n", target_ast);

        let normalized_str = std::format!("{}", normalized_ast);
        let target_str = std::format!("{}", target_ast);

        assert!(normalized_str == target_str);
    }

    #[test]
    fn captured_group() {
        // S -> impl_tuple:("value" A() B()) {impl_tuple} -> (Ident, usize, Vec<i8>) 
        // A -> usize
        // C -> Vec<i8>
       

        let parsed_ast = parse_ast::Parser {
            name: Ident::new("parser_name", Span::call_site()),
            return_type: quote!{Vec<i32>},
            parser_elements: vec![
                parse_ast::ParserElement::Rule (parse_ast::Rule {
                    name: Ident::new("S", Span::call_site()),
                    return_type: quote!{(Ident, usize, Vec<i8>)},
                    productions: vec![
                        // S -> impl_tuple:("value" A() B()) {impl_tuple} 
                        parse_ast::Production {
                            code_segment: quote!{{impl_tuple}},
                            condition: vec![
                                parse_ast::Condition::Captured(Ident::new("impl_tuple", Span::call_site()),
                                    parse_ast::CapturedCondition::Group(vec![
                                        parse_ast::CapturedCondition::Terminal(Ident::new("value", Span::call_site())),
                                        parse_ast::CapturedCondition::Rule(Ident::new("A", Span::call_site())),
                                        parse_ast::CapturedCondition::Rule(Ident::new("B", Span::call_site())),
                                    ])
                                ),
                            ]
                        },
                    ],
                }),
                parse_ast::ParserElement::Rule (parse_ast::Rule {
                    name: Ident::new("A", Span::call_site()),
                    return_type: quote!{usize},
                    productions: vec![],
                }),
                parse_ast::ParserElement::Rule (parse_ast::Rule {
                    name: Ident::new("B", Span::call_site()),
                    return_type: quote!{Vec<i8>},
                    productions: vec![],
                }),
                
            ],
        };

        let target_ast = Parser {
            name: Ident::new("parser_name", Span::call_site()),
            return_type: quote!{Vec<i32>},
            rules: vec![
                Rule {
                    name: Ident::new("S", Span::call_site()),
                    return_type: quote!{(Ident, usize, Vec<i8>)},
                    productions: vec![
                        Production {
                            // S -> a:InternalRepeatRule3() "ident" C() {return a.len();}
                            condition: vec![
                                Condition::Captured(Ident::new("impl_tuple", Span::call_site()), Terminal::Rule(3)), // S:InternalRepeatRule1
                            ],
                            code_segment: quote!{{impl_tuple}}
                        },
                    ]
                },
                Rule {
                    name: Ident::new("A", Span::call_site()),
                    return_type: quote!{usize},
                    productions: vec![]
                },
                Rule {
                    name: Ident::new("B", Span::call_site()),
                    return_type: quote!{Vec<i8>},
                    productions: vec![]
                },
                Rule {
                    name: Ident::new("InternalGroupRule3", Span::call_site()),
                    return_type: quote!{(Ident, usize, Vec<i8>)},
                    productions: vec![
                        // InternalRepeatRule3 -> new:A() v:InternalRepeatRule3() {c.push(new); v}
                        Production {
                            condition: vec![
                                Condition::Captured(Ident::new("arg0", Span::call_site()), Terminal::Const(Ident::new("value", Span::call_site()))), // new:A()
                                Condition::Captured(Ident::new("arg1", Span::call_site()), Terminal::Rule(1)), // arg1:A()
                                Condition::Captured(Ident::new("arg2", Span::call_site()), Terminal::Rule(2)), // arg2:B()
                            ],
                            code_segment: quote!{{(arg0, arg1, arg2)}}
                        },
                    ]
                }
            ]
        };

        let normalized_ast = normalize_ast(parsed_ast);
        
        let normalized_str = std::format!("{}", normalized_ast);
        let target_str = std::format!("{}", target_ast);
        
        print!("{}\n", normalized_str);
        print!("{}\n", target_str);

        assert!(normalized_str == target_str);
    }

    #[test]
    fn captured_repeate_min_one() {
        // S -> a:(A())+ B() -> usize
        // A -> i8
        // B -> u8

        let captured_a_plus = parse_ast::CapturedCondition::Rule(Ident::new("A", Span::call_site()));
       
        let parsed_ast = parse_ast::Parser {
            name: Ident::new("parser_name", Span::call_site()),
            return_type: quote!{Vec<i32>},
            parser_elements: vec![
                parse_ast::ParserElement::Rule (parse_ast::Rule {
                    name: Ident::new("S", Span::call_site()),
                    return_type: quote!{(Ident, usize, Vec<i8>)},
                    productions: vec![
                         // S -> a:(A())+ B() -> usize {}
                        parse_ast::Production {
                            code_segment: quote!{{a.len()}},
                            condition: vec![
                                parse_ast::Condition::Captured(Ident::new("a", Span::call_site()),
                                    parse_ast::CapturedCondition::RepeatMinOne(Box::new(captured_a_plus))
                                ),
                                parse_ast::Condition::Rule(Ident::new("B", Span::call_site()))
                            ]
                        },
                    ],
                }),
                parse_ast::ParserElement::Rule (parse_ast::Rule {
                    name: Ident::new("A", Span::call_site()),
                    return_type: quote!{i8},
                    productions: vec![],
                }),
                parse_ast::ParserElement::Rule (parse_ast::Rule {
                    name: Ident::new("B", Span::call_site()),
                    return_type: quote!{u8},
                    productions: vec![],
                }),
                
            ],
        };

        let target_ast = Parser {
            name: Ident::new("parser_name", Span::call_site()),
            return_type: quote!{Vec<i32>},
            rules: vec![
                Rule {
                    name: Ident::new("S", Span::call_site()),
                    return_type: quote!{(Ident, usize, Vec<i8>)},
                    productions: vec![
                        Production {
                            // S -> a:InternalRepeatRule3() B() {return a.len();}
                            condition: vec![
                                Condition::Captured(Ident::new("a", Span::call_site()), Terminal::Rule(3)), // S:InternalRepeatRule1
                                Condition::Terminal(Terminal::Rule(2)) , // S:InternalRepeatRule1
                            ],
                            code_segment: quote!{{a.len()}}
                        },
                    ]
                },
                Rule {
                    name: Ident::new("A", Span::call_site()),
                    return_type: quote!{i8},
                    productions: vec![]
                },
                Rule {
                    name: Ident::new("B", Span::call_site()),
                    return_type: quote!{u8},
                    productions: vec![]
                },
                Rule {
                    name: Ident::new("InternalRepeatRule3", Span::call_site()),
                    return_type: quote!{Vec<i8>},
                    productions: vec![
                        // InternalRepeatRule3 -> new:A() {vec![new]}
                        Production {
                            condition: vec![
                                Condition::Captured(Ident::new("term", Span::call_site()), Terminal::Rule(1)), // new:A()
                            ],
                            code_segment: quote!{{vec![term]}}
                        },
                        // InternalRepeatRule3 -> new:A() v:InternalRepeatRule3() {c.push(new); v}
                        Production {
                            condition: vec![
                                Condition::Captured(Ident::new("new", Span::call_site()), Terminal::Rule(1)), // new:A()
                                Condition::Captured(Ident::new("v", Span::call_site()), Terminal::Rule(3)), // v:InternalRepeatRule3()
                            ],
                            code_segment: quote!{{v.push(new); v}}
                        },
                    ]
                }
            ]
        };

        let normalized_ast = normalize_ast(parsed_ast);
        
        let normalized_str = std::format!("{}", normalized_ast);
        let target_str = std::format!("{}", target_ast);
        
        print!("{}\n", normalized_str);
        print!("{}\n", target_str);

        assert!(normalized_str == target_str);
    }
}