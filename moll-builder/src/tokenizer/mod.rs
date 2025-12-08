use std::collections::HashMap;

use int_prebuild_test::number;
use intmap::IntMap;

mod fa;
mod builder;
mod int_prebuild_test;

struct Tokenizer<'a> {
    fa_network: fa::FiniteAutomata,
    token_ids: HashMap<String, u32>,
    token_names: IntMap<u32, String>,
    idx: usize, 
    input: &'a str,   
}

impl<'a> Tokenizer<'a> {

    fn new(sm: fa::FiniteAutomata) -> Self {
        let mut ids: HashMap<String, u32> = HashMap::new();
        ids.insert("Reject".to_string(), 0);
        let mut names: IntMap<u32, String> = IntMap::new();
        names.insert(0, "Reject".to_string());
        Tokenizer {
            fa_network: sm,
            token_ids: HashMap::from([("Reject".to_string(), 0)]),
            token_names: names,
            idx: 0,
            input: "",
        }
    }

    pub fn access(&self, from: usize, to: usize) -> &str {
        &self.input[from..to]
    }

    fn add_token_type(&mut self, name: &str, id: u32) {
        if id == 0 {panic!("Token ID of 0 not allowed")}
        self.token_ids.insert(name.to_string(), id);
        self.token_names.insert(id, name.to_string());
    }

    pub fn load_input(& mut self, input: &'a str) {
        self.input = input;
    }

    pub fn next(&mut self) -> Option<(usize, usize, u32)> {
        
        if self.idx == self.input.len() {return Option::None}
        let (len, val) = self.fa_network.parse(&self.input[self.idx..]);

        if len == 0 {
            // If no token found increment idx so if need be parser
            // can progress past current index
            self.idx += 1;
            Option::None
        } else {
            let start = self.idx;
            self.idx += len;
            Option::Some((start, start+len, val))
        }
    }
    
    
    pub fn complete(&self) -> bool {self.idx == self.input.len()}

    pub fn token_name(&self, id: u32) -> Option<&String> {self.token_names.get(id)}

    pub fn token_id(&self, str: &str) -> Option<&u32> {self.token_ids.get(str)}


}


pub fn test_tokenizer() {
    // println!("Testing Tokenizer");
    // let mut network = fa::FiniteAutomata::new();

    // let s = network.create_default_node();
    // let i = network.create_default_node();
    // network.set_fail_action(s, fa::Action::Next(i));
    // network.set_fail_action(i, fa::Action::Next(i));

    // // Set white space transition from start and loop back
    // let w = network.create_node(fa::Action::Terminate(0));
    // network.add_transition_between(' ' as u32, s, w);
    // network.add_transition_between('\t' as u32, s, w);
    // network.add_transition_between('\n' as u32, s, w);
    // network.add_transition_between(' ' as u32, w, w);
    // network.add_transition_between('\t' as u32, w, w);
    // network.add_transition_between('\n' as u32, w, w);

    // // Have Identifer terminate on whitespace
    // network.add_transition(' ' as u32, i, fa::Action::Terminate(1));
    // network.add_transition('\t' as u32, i, fa::Action::Terminate(1));
    // network.add_transition('\n' as u32, i, fa::Action::Terminate(1));

    // // Set up symbol nodes branching from start 
    // for (i, c) in ['<', '>', '{', '}', '$', ';'].iter().enumerate() {
    //     let node = network.create_node(fa::Action::Terminate((i+2).try_into().unwrap()));
    //     network.add_transition_between(*c as u32, s, node);
    //     network.add_transition(*c as u32, i, fa::Action::Terminate(1));
    // }

    // // Handle quotation
    // let sq = network.create_node(fa::Action::TerminateJump(0));
    // network.add_transition_between('\'' as u32, s, sq);
    // let dq = network.create_node(fa::Action::Terminate(0));
    // network.add_transition_between('"' as u32, s, dq);


    

    // let mut tk = Tokenizer::new(network);
    // tk.add_token_type("Identifier", 1);
    // tk.add_token_type("Left Angle Bracket", 2);
    // tk.add_token_type("Right Angle Bracket", 3);
    // tk.add_token_type("Left Bracket", 4);
    // tk.add_token_type("Right Bracket", 5);
    // tk.add_token_type("Dollar Sign", 6);
    // tk.add_token_type("Semi Colon", 7);
    // tk.add_token_type("Literal", 8);

    // tk.load_input("Token node {enter < $Leter; let . > Self; use entrance otherNode; Other > Terminate;}");


    // // Token <name> {
    // //     enter < <Char group>
    // //     let <char group> > <action>
    // //     use entrance <element>
    // //     Other > <action>
    // // }


    // println!("Output:");
    // while let Some((f, t, i)) = tk.next() {
    //     println!("Found {:?} Token: {:?}", tk.token_name(i).expect("Unrecognized token"), tk.access(f, t));
    // }

    // println!("End");

    let (l, v) = number("311. tmp", 0);
    println!("Length of token: {l}, value: {v}");

    

}