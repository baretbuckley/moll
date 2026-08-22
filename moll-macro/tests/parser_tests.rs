use moll_macro;
use std::any::Any;

moll::parser!{
    Parser parser for String {
        Init Rule A -> String {
            a:A() "a" => {a.push("b"); a}
            b:B() => {b}
        }
        Rule B -> String {
            b:B() "b" => {b.push("a"); b}
            "b" => {"a".to_string()}
        }
    }
}

/// {A -> .Aa, A -> .B, B-> .Bb, B->b}
/// {A -> A.a}
/// {A -> B., B -> B.b}
/// {B -> Bb.}
/// {B -> b.}
/// {A -> Aa.}









// #[derive(Clone, Copy)]
// enum __ParserState {
//     __0,
//     __1,
//     __2,
//     __3,
//     __4,
//     __5,
// }

// enum __ParserNonTerminal {
//     A,
//     B,
// }

// enum __ParserRules {
//     A_0,
//     A_1,
//     B_0,
//     B_1,
// }

// enum __ParserToken {
//     Terminal(char),
//     NonTerminal(__ParserNonTerminal),
//     EOF
// }

// pub struct Parser {
//     state: __ParserState,
//     value_stack: Vec<Box<dyn Any>>,
//     state_stack: Vec<__ParserState>,
// }



// impl Parser {

//     pub fn new() -> Self {
//         Parser {state: __ParserState::__0, value_stack: Vec::new(), state_stack: vec![__ParserState::__0]}
//     }

//     fn execute_A_0(mut a: String) -> String {
//         a.push('b'); a
//     }
//     fn execute_A_1(a: String) -> String {
//         a
//     }
//     fn execute_B_0(mut b: String) -> String {
//         b.push('a'); b
//     }
//     fn execute_B_1() -> String {
//         "a".to_string()
//     }

//     fn reduce(&mut self, rule: __ParserRules) {
//         match rule {
//             __ParserRules::A_0 => {
//                 let arg0 = self.value_stack.pop().unwrap().downcast_ref::<String>().unwrap().to_owned();
//                 self.value_stack.push(Box::new(Parser::execute_A_0(arg0)));
//             },
//             __ParserRules::A_1 => {
//                 let arg0 = self.value_stack.pop().unwrap().downcast_ref::<String>().unwrap().to_owned();
//                 self.value_stack.push(Box::new(Parser::execute_A_1(arg0)));
//             },
//             __ParserRules::B_0 => {
//                 let arg0 = self.value_stack.pop().unwrap().downcast_ref::<String>().unwrap().to_owned();
//                 self.value_stack.push(Box::new(Parser::execute_B_0(arg0)));
//             },
//             __ParserRules::B_1 => {
//                 self.value_stack.push(Box::new(Parser::execute_B_1()));
//             },
//         }
//     }

//     fn goto(state: __ParserState, non_term: __ParserNonTerminal) -> __ParserState {
//         match state {
//             __ParserState::__0 => {
//                 match non_term {
//                     __ParserNonTerminal::A => __ParserState::__1,
//                     __ParserNonTerminal::B => __ParserState::__2,
//                 }
//             },
//             _ => {state}
//         }
//     }

//     fn step(&mut self, next: char) {
//         match self.state {
//             __ParserState::__0 => {
//                 match next {
//                     'b' => {
//                         self.state = __ParserState::__4;
//                         self.state_stack.push(self.state);
//                     }
//                     _ => {println!("Error state 0 unexpected input");}
//                 }
//             },
//             __ParserState::__1 => {
//                 match next {
//                     'a' => {
//                         self.state = __ParserState::__5;
//                         self.state_stack.push(self.state);
//                     }
//                     _ => {println!("Error state 1 unexpected input");}
//                 }
//             },
//             __ParserState::__2 => {
//                 match next {
//                     'b' => {
//                         self.state = __ParserState::__3;
//                         self.state_stack.push(self.state);
//                     }
//                     'a' => {
//                         self.reduce(__ParserRules::A_1);
//                         self.state_stack.truncate(self.state_stack.len()-1);
//                         self.state = Parser::goto(*self.state_stack.last().unwrap(), __ParserNonTerminal::A);
//                         self.state_stack.push(self.state);
//                         self.step(next);
//                     }
//                     _ => {println!("Error state 2 unexpected input");}
//                 }
//             }
//             __ParserState::__3 => {
//                 match next {
//                     'b' | 'a' => {
//                         self.reduce(__ParserRules::B_0);
//                         self.state_stack.truncate(self.state_stack.len()-2);
//                         self.state = Parser::goto(*self.state_stack.last().unwrap(), __ParserNonTerminal::B);
//                         self.state_stack.push(self.state);
//                         self.step(next);
//                     }
//                     _ => {println!("Error state 3 unexpected input");}
//                 }
//             }
//             __ParserState::__4 => {
//                 match next {
//                     'b' | 'a' => {
//                         self.reduce(__ParserRules::B_1);
//                         self.state_stack.truncate(self.state_stack.len()-1);
//                         self.state = Parser::goto(*self.state_stack.last().unwrap(), __ParserNonTerminal::B);
//                         self.state_stack.push(self.state);
//                         self.step(next);
//                     }
//                     _ => {println!("Error state 4 unexpected input");}
//                 }
//             }
//             __ParserState::__5 => {
//                 match next {
//                     'a' => {
//                         self.reduce(__ParserRules::A_0);
//                         self.state_stack.truncate(self.state_stack.len()-2);
//                         self.state = Parser::goto(*self.state_stack.last().unwrap(), __ParserNonTerminal::A);
//                         self.state_stack.push(self.state);
//                         self.step(next);
//                     }
//                     _ => {println!("Error state 5 unexpected input");}
//                 }
//             }
//         }
//     }

//     fn end_of_input(&mut self) -> bool {
//         match self.state {
//             __ParserState::__0 => false,
//             __ParserState::__1 => true,
//             __ParserState::__2 => {
//                 self.reduce(__ParserRules::A_1);
//                 self.state_stack.truncate(self.state_stack.len()-1);
//                 self.state = Parser::goto(*self.state_stack.last().unwrap(), __ParserNonTerminal::A);
//                 self.state_stack.push(self.state);
//                 self.end_of_input()
//             },
//             __ParserState::__3 => {
//                 self.reduce(__ParserRules::B_0);
//                 self.state_stack.truncate(self.state_stack.len()-2);
//                 self.state = Parser::goto(*self.state_stack.last().unwrap(), __ParserNonTerminal::B);
//                 self.state_stack.push(self.state);
//                 self.end_of_input()
//             },
//             __ParserState::__4 => false,
//             __ParserState::__5 => {
//                 self.reduce(__ParserRules::A_0);
//                 self.state_stack.truncate(self.state_stack.len()-2);
//                 self.state = Parser::goto(*self.state_stack.last().unwrap(), __ParserNonTerminal::A);
//                 self.state_stack.push(self.state);
//                 self.end_of_input()
//             },
//         }
//     }

//     pub fn parse(&mut self, input: String) -> String {
//         for t in input.chars() {
//             self.step(t);
//         }
//         if !self.end_of_input() {
//             println!("Error input not valid");
//             for i in self.value_stack.iter() {
                
//                 let v = i.downcast_ref::<String>().unwrap().to_owned();
//                 println!("string: {}", v);
//                 match self.state {
//                     __ParserState::__0 => println!("On state 0"),
//                     __ParserState::__1 => println!("On state 1"),
//                     __ParserState::__2 => println!("On state 2"),
//                     __ParserState::__3 => println!("On state 3"),
//                     __ParserState::__4 => println!("On state 4"),
//                     __ParserState::__5 => println!("On state 5"),
//                 }
//             }
//             String::new()
//         } else {
//             self.value_stack.pop().unwrap().downcast_ref::<String>().unwrap().to_owned()
//         }
//     }


    
// }


moll_macro::parser!{}


#[cfg(test)]
mod tests {
    use super::*;



    

    #[test]
    fn cfg_parser_table_assembler() {
        fn foo<'a>(x: &'a i32) -> &'a i32 {
            x
        }
        let bar = foo::<'static>;
        bar(&1);
    }
}