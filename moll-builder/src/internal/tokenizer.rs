use std::collections::HashMap;
use std::fs;

pub enum Token {
    Identifier(usize, usize),
    Symbol(char),
    Literal(usize, usize),
    EOF,
}

enum SpecialChars {
    Symbol,
    LiteralOpen,
    Whitespace   
}

pub struct Tokenizer<'a> {
    input: &'a str,
    pos: usize,
    special_chars: HashMap<char, SpecialChars>
}

impl<'a> Tokenizer<'a> {
    pub fn new(input_string: &'a str) -> Self {
        Tokenizer{
            input: input_string,
            pos: 0,
            special_chars: HashMap::from([
                ('>', SpecialChars::Symbol),
                (':', SpecialChars::Symbol),
                (';', SpecialChars::Symbol),
                (',', SpecialChars::Symbol),
                ('$', SpecialChars::Symbol),
                ('{', SpecialChars::Symbol),
                ('}', SpecialChars::Symbol),
                ('(', SpecialChars::Symbol),
                (')', SpecialChars::Symbol),

                ('\'', SpecialChars::LiteralOpen),
                ('"', SpecialChars::LiteralOpen),

                ('\n', SpecialChars::Whitespace),
                (' ', SpecialChars::Whitespace),
                ('\t', SpecialChars::Whitespace),
                ('\r', SpecialChars::Whitespace),
            ])
        }
    }

    pub fn access(&self, from: usize, to: usize) -> &'a str {
        &self.input[from..to]
    }

    fn cut_input(&mut self, n: usize) {
        self.input = &self.input[n..];
        self.pos += n;
    }
    fn cut_rest(&mut self) {
        self.input = &self.input[self.input.len()..];
        self.pos = self.input.len();
    }

    fn cur_input(&self) -> &str {
        &self.input[self.pos..]
    }


    fn cur_input_from(&self, offset: usize) -> &str {
        &self.input[self.pos+offset..]
    }

    // fn splice_input

    fn get_next_literal(&mut self, closer: char) -> Token {
        let mut idx: usize = 0;
        for c in self.cur_input_from(1).chars() {
            if c == closer {
                let ret: Token = Token::Literal(self.pos+1, self.pos+idx);
                self.cut_input(idx+1);
                return ret;
            }
            idx += c.len_utf8();
        }
        panic!("Unexpected end of file in reading string literal");
    }

    fn skip_whitespace(&mut self) {
        // While input hasn't reached eof and continues return whitespace
        let mut i = 0;
        for c in self.cur_input().chars() {
            if let Some(SpecialChars::Whitespace) = self.special_chars.get(&c) {
                i += c.len_utf8();
                continue;
            }
            self.cut_input(i);
            return;
        }
        self.cut_rest();
    }

    fn get_next_identifier(&mut self) -> Token {
        let mut idx = 0;
        for c in self.cur_input().chars() {
            if let Some(_) = self.special_chars.get(&c) {
                let ret = Token::Identifier(self.pos, self.pos + idx);
                self.cut_input(idx);
                return ret;
            }
            idx += c.len_utf8();
        }
        let ret = Token::Identifier(self.pos, self.pos+self.input.len());
        self.input = &self.input[self.input.len()-1..];
        ret
    }

    fn get_next_symbol(&mut self, c: char) -> Token {
        self.cut_input(1);
        Token::Symbol(c)
    } 

    pub fn next(&mut self) -> Token {
        // let str = self.input.as_str();
        let c = match self.cur_input().chars().nth(0) {
            Some(c_ret) => c_ret,
            None => return Token::EOF
        };
        match self.special_chars.get(&c) {
            Some(c_type) => match *c_type {
                SpecialChars::Symbol => self.get_next_symbol(c), 
                SpecialChars::LiteralOpen => self.get_next_literal(c), 
                SpecialChars::Whitespace => {
                    self.skip_whitespace();
                    self.next()
                } 
            },
            None => self.get_next_identifier()
        }
    }


    pub fn next_identifier(&mut self) -> (usize, usize) {
        
        let ret = self.next();

        if let Token::Identifier(f, t) = ret {(f, t)}
        else {panic!("Expected Identifier")}
    }

    pub fn skip_symbol(&mut self, c: char) {
        if let Token::Symbol(fc) = self.next() {
            if fc != c {
                panic!("Wrong symbol encountered")
            }
        } else {
            panic!("Expected symbol");
        }
    }

}


pub fn test_internal_tokenizer() {
    let contents = fs::read_to_string("examples/tokenizer.tkn")
        .expect("Should have been able to read the file");

    let mut tk = Tokenizer::new(contents.as_str());

    
    loop {
        match tk.next() {
            Token::Identifier(f, t) => println!("Identifier: {:?}", &contents[f..t]),
            Token::Literal(f, t) => println!("Literal: {:?}", &contents[f..t]),
            Token::Symbol(c) => println!("Symbol: {c}"),
            Token::EOF => break,
        }
    }
}