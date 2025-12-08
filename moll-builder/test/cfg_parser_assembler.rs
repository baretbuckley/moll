use std::collections::HashMap;

use super::grammar;
use super::builder;
use super::runtime;

Tokens {
    S,
    P,
    V,
}

pub fn test_runtime_parser() {
    // S -> S + P | P
    // P -> P * V | V
    // V -> # | i
    let grammar = grammar::Grammar::new_from( Tokens::S, vec![
        grammar::Rule {result: Tokens::S, rule: vec![grammar::Token::NonTerminal(Tokens::S), grammar::Token::Terminal('+'), grammar::Token::NonTerminal(Tokens::P)]},
        grammar::Rule {result: Tokens::S, rule: vec![grammar::Token::NonTerminal(Tokens::P)]},
        grammar::Rule {result: Tokens::P, rule: vec![grammar::Token::NonTerminal(Tokens::P), grammar::Token::Terminal('*'), grammar::Token::NonTerminal(Tokens::V)]},
        grammar::Rule {result: Tokens::P, rule: vec![grammar::Token::NonTerminal(Tokens::V)]},
        grammar::Rule {result: Tokens::V, rule: vec![grammar::Token::Terminal('#')]},
        grammar::Rule {result: Tokens::V, rule: vec![grammar::Token::Terminal('i')]},
    ]);

    let builder: builder::Builder<'_, char, Tokens, runtime::LALRParser<_,_>> = builder::Builder::build_from(&grammar);
    builder.display();
    let mut parser = builder.into_parser();

    let tk_map: HashMap<char, grammar::Token<char, Tokens>> = HashMap::from([
        ('S', grammar::Token::NonTerminal(Tokens::S)),
        ('P', grammar::Token::NonTerminal(Tokens::P)),
        ('V', grammar::Token::NonTerminal(Tokens::P)),
        ('#', grammar::Token::Terminal('#')),
        ('i', grammar::Token::Terminal('i')),
        ('+', grammar::Token::Terminal('+')),
        ('*', grammar::Token::Terminal('*')),
    ]);

    let str = "# + i".to_string();
    // let str = "# + i".to_string();
    let mut vec_input: Vec<grammar::Token<char, char>> = Vec::new();
    
    for c in str.chars() {
        if c == ' ' {continue;}
        match tk_map.get(&c) {
            Some(t) => vec_input.push(*t),
            None => unreachable!(),
        }
    }
    println!("Parser: {}", parser.display(&grammar));
    parser.parse::<runtime::AssemTmp>(vec_input, &grammar);
    


    println!("Newly created Builder");

}