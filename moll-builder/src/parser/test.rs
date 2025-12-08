use std::collections::HashMap;

use super::grammar;
use super::builder;
use super::runtime;

pub fn test_runtime_parser() {
    // S -> S + P | P
    // P -> P * V | V
    // V -> # | i
    // let grammar = Grammar { start: 'B', rules: vec![
    //     Rule {result: 'B', rule: vec![Token::NonTerminal('S'), Token::Final]},
    //     Rule {result: 'S', rule: vec![Token::NonTerminal('S'), Token::Terminal('+'), Token::NonTerminal('P')]},
    //     Rule {result: 'S', rule: vec![Token::NonTerminal('P')]},
    //     Rule {result: 'P', rule: vec![Token::NonTerminal('P'), Token::Terminal('*'), Token::NonTerminal('V')]},
    //     Rule {result: 'P', rule: vec![Token::NonTerminal('V')]},
    //     Rule {result: 'V', rule: vec![Token::Terminal('#')]},
    //     Rule {result: 'V', rule: vec![Token::Terminal('i')]},
    // ]};
    let grammar = grammar::Grammar::new_from( 'S', vec![
        grammar::Rule {result: 'S', rule: vec![grammar::Token::NonTerminal('S'), grammar::Token::Terminal('+'), grammar::Token::NonTerminal('P')]},
        grammar::Rule {result: 'S', rule: vec![grammar::Token::NonTerminal('P')]},
        grammar::Rule {result: 'P', rule: vec![grammar::Token::NonTerminal('P'), grammar::Token::Terminal('*'), grammar::Token::NonTerminal('V')]},
        grammar::Rule {result: 'P', rule: vec![grammar::Token::NonTerminal('V')]},
        grammar::Rule {result: 'V', rule: vec![grammar::Token::Terminal('#')]},
        grammar::Rule {result: 'V', rule: vec![grammar::Token::Terminal('i')]},
        grammar::Rule {result: 'V', rule: vec![grammar::Token::NonTerminal('L')]},
        grammar::Rule {result: 'L', rule: vec![grammar::Token::Terminal('a')]},
    ]);
    // let grammar = grammar::Grammar::new_from( 'B', vec![
    //     grammar::Rule::new_from ('B', vec![grammar::Token::NonTerminal('S'), grammar::Token::Final]),
    //     grammar::Rule::new_from ('S', vec![grammar::Token::Terminal('+'), grammar::Token::NonTerminal('S'), grammar::Token::NonTerminal('P')]),
    //     grammar::Rule::new_from ('S', vec![grammar::Token::NonTerminal('P')]),
    //     grammar::Rule::new_from ('P', vec![grammar::Token::Terminal('*'), grammar::Token::NonTerminal('P'), grammar::Token::NonTerminal('V')]),
    //     grammar::Rule::new_from ('P', vec![grammar::Token::NonTerminal('V')]),
    //     grammar::Rule::new_from ('V', vec![grammar::Token::Terminal('#')]),
    //     grammar::Rule::new_from ('V', vec![grammar::Token::Terminal('i')]),
    // ]);
    // let grammar = grammar::Grammar::new_from( 'B', vec![
    //     grammar::Rule {result: 'B', rule: vec![grammar::Token::NonTerminal('S'), grammar::Token::Final]},
    //     grammar::Rule {result: 'S', rule: vec![grammar::Token::NonTerminal('X'), grammar::Token::NonTerminal('X')]},
    //     grammar::Rule {result: 'X', rule: vec![grammar::Token::Terminal('a'), grammar::Token::NonTerminal('X')]},
    //     grammar::Rule {result: 'X', rule: vec![grammar::Token::Terminal('b')]},
    // ]);

    let builder: builder::Builder<'_, char, char, runtime::LALRParser<_,_>> = builder::Builder::build_from(&grammar);
    builder.display();
    let mut parser = builder.into_parser();

    let tk_map: HashMap<char, grammar::Token<char, char>> = HashMap::from([
        ('B', grammar::Token::NonTerminal('B')),
        ('S', grammar::Token::NonTerminal('S')),
        ('P', grammar::Token::NonTerminal('P')),
        ('V', grammar::Token::NonTerminal('V')),
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