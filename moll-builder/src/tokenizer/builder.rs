// use std::collections::HashMap;
// use intmap::IntMap;

// use crate::internal;
// use crate::internal::tokenizer::Token as I_Token;
// use crate::tokenizer::fa;

// use super::fa::Action;


// struct ActionDef {
//     action_type: (usize, usize),
//     target: (usize, usize)
// }
// struct TokenDef<'a> {
//     pub name: &'a str,
//     pub transitions: Vec<(Vec<char>, &'a str)>,
//     pub other: &'a str,
// }

// impl<'a> TokenDef<'a> {
//     pub fn new(name: &'a str) -> Self {
//         TokenDef{
//             name: name,
//             transitions: Vec::new(),
//             other: ""
//         }
//     }
// }


// struct Builder<'a> {
//     tokenizer: internal::tokenizer::Tokenizer<'a>,
//     tokens: Vec<TokenDef<'a>>,
//     token_ids: HashMap<String, u32>,
//     token_names: IntMap<u32, String>,
//     token_id_count: u32,
// }

// impl<'a> Builder<'a> {

//     pub fn new(input: &'a str) -> Self {
//         Builder {
//             tokenizer: internal::tokenizer::Tokenizer::new(input),
//             tokens: Vec::new(),
//             token_ids: HashMap::new(),
//             token_names: IntMap::new(),
//             token_id_count: 1 
//         }
//     }

//     fn parse_action(& mut self) -> ActionDef {
//         ActionDef {action_type: self.tokenizer.next_identifier(), target: self.tokenizer.next_identifier()}
//     }

//     fn parse_token_statement(&mut self, tkn_ref: &mut TokenDef, lead_tkn: I_Token) {

//     }

//     // fn parse_token_statement(&mut self, tkn_ref: &mut TokenDef) {
//     //     let t = self.tokenizer.next();
//     //     self.parse_token_statement_lead_tkn(tkn_ref, t);
//     // }

//     fn parse_token(&mut self) {
//         let token = TokenDef::new(self.tokenizer.next_identifier());
//         self.tokenizer.skip_symbol('{');

//         let t = self.tokenizer.next();
//         match t {
//             I_Token::Identifier(f, t) => {
//                 if (s == "Enter") {
//                     self.tokenizer.skip_symbol(':');
//                     // token. = self.parse_char_group();
//                     self.tokenizer.skip_symbol(';');
//                 } else {
//                     // self.parse_token_statement();
//                 }
//             }
//             _ => {self.parse_action();}
//         }

//     }

//     fn parse(&mut self) {
//         if let I_Token::Identifier(next) = self.tokenizer.next() {
//             if (next == "Token") {

//             } else {
//                 panic!("Unknown Element");
//             }
//         } else {
//             panic!("Expected Identifier");
//         }

//     }

//     pub fn read() {


//     }

// }