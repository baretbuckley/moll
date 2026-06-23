
mod kickstarter_parser;
mod ast;
mod expanded_ast;
mod grammar;
mod compile;
mod translate;
mod normalize;
// mod parserrust;

// use moll_builder::parser::grammar::Grammar;

use std::vec;

use proc_macro;
use proc_macro2::{Group, Ident, Span};
use quote::quote;

use crate::translate::translate_grammar;



#[proc_macro]
pub fn parser(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    println!("Testing exec fn compilation");
    let mut test = expanded_ast::Parser { 
        parser_name: Ident::new("Parser", Span::call_site()),
        num_states: 6,
        non_terminals: vec![Ident::new("A", Span::call_site()), Ident::new("B", Span::call_site())],
        rules: vec![
            expanded_ast::Rule { 
                non_term_idx: 0,
                prods: vec![
                    expanded_ast::Production {
                        params:vec![(Some(Ident::new("a",Span::call_site())),0)],
                        exec:Group::new(proc_macro2::Delimiter::Brace,quote!{a.push('b');a}), 
                        tokens_count: 2,
                    },

                    expanded_ast::Production {
                        params: vec![(Some(Ident::new("b", Span::call_site())), 1)], 
                        exec: Group::new(proc_macro2::Delimiter::Brace, quote!{b}),
                        tokens_count: 1,
                    },
                ],
                return_type: quote!{String}, 
            },
            expanded_ast::Rule { 
                non_term_idx: 1,
                prods: vec![
                    expanded_ast::Production {
                        params: vec![(Some(Ident::new("b", Span::call_site())), 1)],
                        exec: Group::new(proc_macro2::Delimiter::Brace, quote!{b.push('a'); b}),
                        tokens_count: 2,
                    },

                    expanded_ast::Production {
                        params: vec![], 
                        exec: Group::new(proc_macro2::Delimiter::Brace, quote!{"a".to_string()}),
                        tokens_count: 1,
                    },
                ],
                return_type: quote!{String}, 
            },
        ],
        terminal_transitions: vec![
            vec![ (quote!{'b'}, expanded_ast::Action::Shift(4)) ],
            vec![ (quote!{'a'}, expanded_ast::Action::Shift(5)) ],
            vec![ (quote!{'a'}, expanded_ast::Action::Reduce((0,1))), (quote! {'b'}, expanded_ast::Action::Shift(3)) ],
            vec![ (quote!{'a' | 'b'}, expanded_ast::Action::Reduce((1,0))) ],
            vec![ (quote!{'b' | 'a'}, expanded_ast::Action::Reduce((1,1))) ],
            vec![ (quote!{'a'}, expanded_ast::Action::Reduce((0,0))) ],
        ],
        non_terminal_transitions: vec![
            vec![(0, 1), (1, 2)],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
        ],
        eoi_actions: vec![
            expanded_ast::EOIAction::Acceptance(false),
            expanded_ast::EOIAction::Acceptance(true),
            expanded_ast::EOIAction::Reduce((0,1)),
            expanded_ast::EOIAction::Reduce((1,0)),
            expanded_ast::EOIAction::Acceptance(false),
            expanded_ast::EOIAction::Reduce((0,0)),
        ],
        init_rule: 0,
        input_type: quote!{&str},
        iterator_fn: quote!{chars()},
        element_type: quote!{char},
    };

    // println!("compiled grammar: \n{}", compile::compile_parser(&mut test));

    // _ = translate_grammar(grammar::Grammar::new_from( 'S', vec![
    //     grammar::Rule {result: 'S', rule: vec![grammar::Token::NonTerminal('S'), grammar::Token::Terminal('+'), grammar::Token::NonTerminal('P')]},
    //     grammar::Rule {result: 'S', rule: vec![grammar::Token::NonTerminal('P')]},
    //     grammar::Rule {result: 'P', rule: vec![grammar::Token::NonTerminal('P'), grammar::Token::Terminal('*'), grammar::Token::NonTerminal('V')]},
    //     grammar::Rule {result: 'P', rule: vec![grammar::Token::NonTerminal('V')]},
    //     grammar::Rule {result: 'V', rule: vec![grammar::Token::Terminal('#')]},
    //     grammar::Rule {result: 'V', rule: vec![grammar::Token::Terminal('i')]},
    // ]));
    
    compile::compile_parser(&mut test).into()
    // TokenStream::new()
}

