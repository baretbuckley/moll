use proc_macro2::{Group, Ident, TokenStream};



pub struct Production {
    pub params: Vec<(Option<Ident>, usize)>, // params for the prod rule, None means ignored, interpet as (param name, type idx)
    pub tokens_count: usize, // Number of tokens replaced by the production
    pub exec: Group, // Execution code block
}

pub struct Rule {
    pub non_term_idx: usize,
    pub prods: Vec<Production>, 
    pub return_type: TokenStream,
}

pub enum Action {
    Shift(usize), // target state idx
    Reduce((usize, usize)), // reduce (rule idx, production idx)
}

pub enum EOIAction {
    Acceptance(bool), // if input is accepted at current state
    Reduce((usize, usize)), // reduce (rule idx, production idx)
}

pub struct Parser {
    pub parser_name: Ident,
    pub num_states: usize,
    pub non_terminals: Vec<Ident>,
    pub rules: Vec<Rule>,
    pub init_rule: usize, // idx of initial rule in rules
    pub input_type: TokenStream, // Type of input recieved
    pub iterator_fn: TokenStream,
    pub element_type: TokenStream,
    pub terminal_transitions: Vec<Vec<(TokenStream, Action)>>, // [state idx][_] -> (terminal_tokensteam, action)
    pub non_terminal_transitions: Vec<Vec<(usize, usize)>>, // [state idx][_] -> (non-term idx, target state idx)
    pub eoi_actions : Vec<EOIAction>, // [state idx] -> end of input action
}