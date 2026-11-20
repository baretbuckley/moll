use std::fmt::Display;

#[derive (Ord, PartialEq, Copy, Clone, PartialOrd, Eq, Hash, Debug)]
pub enum Token<T, N> {
    Terminal(T),
    NonTerminal(N),
    Final, // special token representing end of input
    Root, // special token representing the root of the grammar
}

impl<T: std::fmt::Display, N: std::fmt::Display> Display for Token<T,N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Terminal(c) => write!(f, "{c}"),
            Token::NonTerminal(c) => write!(f, "{c}"),
            Token::Final => write!(f, "$"),
            Token::Root => write!(f, "§"),
        }
    }
}

// impl Ord for Token {
//     fn cmp(&self, other: &Self) -> std::cmp::Ordering {
//         match (*self, other) {
//             (Token::Terminal(lc), Token::Terminal(rc)) => lc.cmp(rc),
//             (Token::NonTerminal(lc), Token::NonTerminal(rc))=> lc.cmp(rc),
//             (Token::Terminal(_), _) => std::cmp::Ordering::Greater,
//             (_, Token::Terminal(_)) => std::cmp::Ordering::Less,
//             (Token::NonTerminal(_), Token::Final) => std::cmp::Ordering::Greater,
//             (Token::Final, Token::NonTerminal(_)) => std::cmp::Ordering::Less,
//             (Token::Final, Token::Final) => std::cmp::Ordering::Equal,
//         }
//     }
// }

pub struct Rule<T,N> {
    pub result: N,
    pub rule: Vec<Token<T, N>>
}

impl<T,N> Rule<T,N> {
    pub fn new_from(result: N, rule: Vec<Token<T,N>>) -> Self {
        Rule{ result, rule }
    }
}


pub struct Grammar<Terminal, NonTerminal> {
    rules: Vec<Rule<Terminal,NonTerminal>>,
    start: NonTerminal,
}

impl<T,N: Copy> Grammar<T,N> {
    pub fn new_from(start: N, rules: Vec<Rule<T,N>>) -> Self {
        let mut grammar = Grammar {rules, start};
        // Ensure grammar does not contain final or root tokens in rules
        for rule in grammar.rules.iter() {
            for token in rule.rule.iter() {
                match token {
                    Token::Final | Token::Root => {
                        panic!("Grammar rules cannot contain Final or Root tokens");
                    },
                    _ => {}
                }
            }
        }

        // Add implicit start rule
        let start_rule: Rule<T, N> = Rule {
            result: grammar.start,
            rule: vec![Token::NonTerminal(grammar.start.clone()), Token::Final],
        };
        grammar.rules.insert(0, start_rule);

        grammar
    }
}

impl<T,N> Grammar<T,N> {

    pub fn rules(&self) -> &Vec<Rule<T,N>> {&self.rules}
    pub fn start(&self) -> &N {&self.start}

    pub fn get(&self, rule_idx: RuleIdx) -> Option<&Rule<T,N>> {
        self.rules.get(rule_idx.idx)
    }
}

#[derive(Eq, PartialEq, Hash, Clone, Copy, PartialOrd, Ord)]
pub struct RuleIdx {
    pub idx: usize
}

impl RuleIdx {
    pub fn new(idx: usize) -> Self {RuleIdx {idx}}
    pub fn new_option(idx: Option<usize>) -> Option<Self> {
        Some(Self::new(idx?))
    }
}

/// Item is a rule with a position marker
/// Conventionaly annotated with a • in the position
/// E.G "S -> a b D e" with a position of 2 is:
/// S -> a b • D e
#[derive(Eq, PartialEq, Hash, Clone, Copy, PartialOrd, Ord)]
pub struct Item {
    // Reference to rule in grammar
    rule: RuleIdx,
    // Position within referenced rule
    pos: usize,
}

impl Item {

    pub fn new(rule_idx: RuleIdx, pos: usize) -> Self {
        Item { rule: rule_idx, pos }
    }

    pub fn next<'a, T, N>(&self, grammar: &'a Grammar<T,N>) -> Option<&'a Token<T,N>> {
        grammar.get(self.rule)?
            .rule.get(self.pos)
    }

    pub fn first<'a, T, N>(&self, grammar: &'a Grammar<T,N>) -> Option<&'a Token<T,N>> {
        grammar.get(self.rule)?
            .rule.first()
    }

    pub fn step<'a, T, N>(&mut self, grammar: &'a Grammar<T,N>) -> Option<&'a Token<T,N>> {
        let ret: &Token<T,N> = grammar.get(self.rule)?
            .rule.get(self.pos)?;
        self.pos += 1;
        Some(ret)
    }

    pub fn completed<T,N>(&self, grammar: &Grammar<T,N>) -> bool{
        if let Some(r) = grammar.get(self.rule) {
            r.rule.len() == self.pos
        } else {
            false
        }
    }

    pub fn completed_rule<T,N>(&self, rule: &Rule<T,N>) -> bool {
        rule.rule.len() == self.pos
    }


    pub fn rule_idx(&self) -> RuleIdx {self.rule}

    pub fn display<'a, T, N>(&'a self, grammar: &'a Grammar<T,N>) -> DisplayItem<'a, T, N> {
        DisplayItem {item: self, grammar: grammar}
    }
}

pub struct DisplayItem<'a, T, N> {
    item: &'a Item,
    grammar: &'a Grammar<T,N>
}

impl<T: std::fmt::Display, N: std::fmt::Display> Display for DisplayItem<'_, T, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let rule = self.grammar.get(self.item.rule).unwrap();
        _ = write!(f, "{:} :", rule.result);
        for i in 0..self.item.pos {
            _ = write!(f, " {}", rule.rule[i]);
        }
        _ = write!(f, " •");
        for i in self.item.pos..rule.rule.len() {
            _ = write!(f, " {}", rule.rule[i]);
        }
        write!(f, "");
        Ok(())
    }
}


