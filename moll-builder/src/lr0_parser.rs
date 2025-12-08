// use std::{collections::{BTreeMap, BTreeSet, HashMap, HashSet}, fmt::Display, result};
// use std::hash::Hash;
// use intmap::IntMap;

// use crate::fa;






// /// Mapping between nonterminal symbols and the set of all valid Items to start the given symbol
// /// E.G.
// /// S -> A | a
// /// A -> B | b
// /// B -> c
// /// get(B) -> {
// ///     S -> •A,
// ///     S -> •a,
// ///     A -> •B,
// ///     A -> •b,
// ///     B -> •c
// /// }
// /// get(A) -> {
// ///     A -> •B,
// ///     A -> •b,
// ///     B -> •c
// /// }



// struct Node {
//     next: BTreeMap<Token, usize>
// }

// impl Node {
//     fn new() -> Self {
//         Node {next: BTreeMap::new() }
//     }
// }

// impl Display for Node {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         _ = write!(f, "{{ ");
//         for (condition, to) in self.next.iter() {
//             _ = write!(f, "{{ {} -> {} }} ", condition, to);
//         }
//         write!(f, "}}")
//     }
// }

// struct Builder<'a, FA: fa::FAProducer<Token, usize, Option<usize>>> {
//     grammar: &'a Grammar,
//     tables: Vec<Table>,
//     // table_set: HashMap<TableHashProxy<'a>, usize>, // TODO: find hash that works with borrow checker and avoids copying Table
//     first_table: FirstTable,
//     network: FA,
// }

// impl<'a, FA: fa::FAProducer<Token, usize, Option<usize>>> Builder<'a, FA> {
//     pub fn build_from(grammar: &'a Grammar) -> Self {
//         let mut builder = Builder {
//             grammar: &grammar,
//             tables: Vec::new(),
//             first_table: FirstTable::new_from(&grammar),
//             network: FA::new()
//         };
//         if let Some(tmp) = builder.first_table.get(grammar.start) {
//             let table: Table = Table::new_from_ref(tmp, &builder.first_table, grammar);
//             let idx = builder.register_table_force(table);
//             println!("given idx is {idx}");
//             builder.display();
//             builder.propagate(idx);
//             builder.display();
//         }
        
//         builder
//     }

//     fn new_state(&mut self, table: &Table) {
//         let mut result: Option<usize> = None;
//         for r in table.items.iter() {
//             if r.completed(self.grammar) {
//                 if let Some(o) = result {
//                     println!("Warning Reduce/Reduce colision between {} and {}.", Item {rule_idx: o, pos: 0}.display(self.grammar), r.display(self.grammar));
//                     break;
//                 }
//                 result = Some(r.rule_idx);
//             }
//         }
//         self.network.new_state(result);
//     }

//     // Regesters the table without checking it matching table already regestered
//     fn register_table_force(&mut self, table: Table) -> usize {
//         self.new_state(&table);
//         self.tables.push(table);
//         let idx = self.tables.len()-1;
//         let table_ref = self.tables.last_mut().unwrap();
//         table_ref.idx = idx;
//         idx
//     }

//     fn register_table(&mut self, table: Table) -> Result<usize, usize> {
//         for (i, t) in self.tables.iter().enumerate() {
//             if *t == table {
//                 return Err(i);
//             }
//         }
//         Ok(self.register_table_force(table))
//     }

//     fn add_connection(&mut self, cond: Token, from: usize, to: usize) {
//         self.network.new_connection(cond, from, to);
//     }   

//     fn add_table(&mut self, table: Table) -> usize {
//         match  self.register_table(table) {
//             Ok(idx) => {
//                 self.propagate(idx);
//                 idx
//             },
//             Err(idx) => idx
//         }
//     }

//     fn propagate(&mut self, table_idx: usize) {
//         println!("Propagating table {table_idx}");
//         // Catagorize items by their next token
//         let table: &Table = self.tables.get(table_idx).unwrap();
//         let mut next_steps: BTreeMap<Token, Vec<Item>> = BTreeMap::new();
//         for item_ref in table.items.iter() {
//             let mut item = item_ref.clone();
//             if let Some(next_token) = item.step(self.grammar) {
//                 let v = next_steps.entry(*next_token).or_insert(Vec::new());
//                 v.push(item);
//             }
//         }

//         // For each next token, create a table then add to builder
//         // Issue in cyclical stalling, is handled in 
//         for (step, items) in next_steps {
//             println!("Propagateing with token {step}");
//             let table: Table = Table::new_from(items, &self.first_table, self.grammar);
//             let idx = self.add_table(table);
//             self.add_connection(step, table_idx, idx);
//         }

//     }

//     #[inline]
//     pub fn into_parser(self) -> FA {
//         self.network
//     }

//     pub fn display(&self) {
//         println!("Parsing States {{");
//         for i in 0..self.tables.len() {
//             println!("\t State {}: {}", i, self.tables[i].display(self.grammar)); // TODO: add display for the network
//         }
//         println!("}}");
//     }

// }



// struct ProdNode {
//     parant: usize,
//     symbol: Token,
//     instantiated: bool,
//     prod: usize,
//     branches: Vec<usize>
// }

// impl ProdNode {

//     fn new(symbol: Token) -> Self {
//         ProdNode {parant: 0, symbol, instantiated: false, prod: 0, branches: Vec::new()}
//     }

// }



// struct ProdTree {
//     nodes: Vec<ProdNode>
// }



// struct TestFA {}

// impl fa::FAProducer<Token, usize, Option<usize>> for TestFA {

//     fn new() -> Self {
//         todo!()
//     }

//     fn new_state(&mut self, result: Option<usize>) -> usize {
//         todo!()
//     }

//     fn set_state_result(&mut self, state: usize, result: Option<usize>) -> usize {
//         todo!()
//     }

//     fn new_connection(&mut self, condition: Token, from: usize, to: usize) -> bool {
//         todo!()
//     }
// }

// impl Parser for TestFA {
//     fn parse<'a, A: Assembler>(&self, assembler: &'a A, collection: impl IntoIterator<Item = Token>) -> &'a A {
//         todo!()
//     }
// }

// struct testAsm {}

// impl Assembler for testAsm {
    


// }


 
// // impl ProdTree {

// //     fn new() -> Self {
// //         ProdTree {nodes: Vec::new()}
// //     }

// //     fn instantiate_node(&mut self, node_idx: usize, grammar: &Grammar) {
// //         let node = self.nodes.get(node_idx).unwrap();
// //         if let Some(rule) = grammar.get(node.prod) {
// //             for t in rule.rule.iter() {
// //                 if Token::NonTerminal(())
// //             }
// //         }
// //     } 

// //     fn start(&mut self, grammar: &Grammar) -> usize {
// //         self.nodes.push(ProdNode::new(Token::NonTerminal(grammar.start)));
// //         self.nodes.len()-1
// //     }

// // }


// pub fn test_first_table() {
//     // S -> S + P | P
//     // P -> P * V | V
//     // V -> # | i
//     // let grammar = Grammar { start: 'B', rules: vec![
//     //     Rule {result: 'B', rule: vec![Token::NonTerminal('S'), Token::Final]},
//     //     Rule {result: 'S', rule: vec![Token::NonTerminal('S'), Token::Terminal('+'), Token::NonTerminal('P')]},
//     //     Rule {result: 'S', rule: vec![Token::NonTerminal('P')]},
//     //     Rule {result: 'P', rule: vec![Token::NonTerminal('P'), Token::Terminal('*'), Token::NonTerminal('V')]},
//     //     Rule {result: 'P', rule: vec![Token::NonTerminal('V')]},
//     //     Rule {result: 'V', rule: vec![Token::Terminal('#')]},
//     //     Rule {result: 'V', rule: vec![Token::Terminal('i')]},
//     // ]};
//     let grammar = Grammar { start: 'B', rules: vec![
//         Rule {result: 'B', rule: vec![Token::NonTerminal('S'), Token::Final]},
//         Rule {result: 'S', rule: vec![Token::Terminal('+'), Token::NonTerminal('S'), Token::NonTerminal('P')]},
//         Rule {result: 'S', rule: vec![Token::NonTerminal('P')]},
//         Rule {result: 'P', rule: vec![Token::Terminal('*'), Token::NonTerminal('P'), Token::NonTerminal('V')]},
//         Rule {result: 'P', rule: vec![Token::NonTerminal('V')]},
//         Rule {result: 'V', rule: vec![Token::Terminal('#')]},
//         Rule {result: 'V', rule: vec![Token::Terminal('i')]},
//     ]};

//     let builder: Builder<'_, TestFA> = Builder::build_from(&grammar);
//     let parser = builder.into_parser();


//     println!("Newly created Builder");

// }

// /*
// Parsing States {
//     State 0: { {B : • S  $} {S : • S + P} {S : • P} {P : • P * V} {P : • V} {V : • #} {V : • i} }, { { P -> 1 } { S -> 4 } { V -> 8 } { # -> 9 } { i -> 10 } }
//     State 1: { {S : P •} {P : P • * V} }, { { * -> 2 } }
//     State 2: { {P : P * • V} }, { { V -> 3 } }
//     State 3: { {P : P * V •} }, { }
//     State 4: { {B : S •  $} {S : S • + P} }, { {  $ -> 5 } { + -> 6 } }
//     State 5: { {B : S  $ •} }, { }
//     State 6: { {S : S + • P} }, { { P -> 7 } }
//     State 7: { {S : S + P •} }, { }
//     State 8: { {P : V •} }, { }
//     State 9: { {V : # •} }, { }
//     State 10: { {V : i •} }, { }
// }
// */