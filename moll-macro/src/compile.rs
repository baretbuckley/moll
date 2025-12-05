use std::collections::HashMap;

use intmap::IntMap;
use quote::{TokenStreamExt, quote};
use proc_macro2::{Group, Ident, Punct, Span, TokenStream, TokenTree};

// use crate::{ast::Token, expanded_ast::Parser, parser};

use super::expanded_ast as ast;

struct ParserStateContext {
    ident: Ident,
    idx: Vec<Ident>
}

impl ParserStateContext {
    pub fn new() -> Self {
        ParserStateContext { ident: Ident::new("INVALID", Span::call_site()), idx: Vec::new() }
    }
}

struct Context {
    exec_ids: IntMap<usize, Vec<Ident>>,
    reduce_ids: IntMap<usize, Vec<Ident>>,
    exec_param_types: HashMap<(usize, usize), Vec<TokenStream>>,
    state: ParserStateContext,
    non_terms: (Ident, Vec<Ident>),
    parser_rule_ids: (Ident, Vec<Vec<Ident>>),
    non_term_type: TokenStream,
}

impl Context {
    pub fn new() -> Self {
        Context { 
            exec_ids: IntMap::new(),
            reduce_ids: IntMap::new(),
            exec_param_types: HashMap::new(),
            state: ParserStateContext::new(),
            non_terms: (Ident::new("INVALID", Span::call_site()), Vec::new()),
            parser_rule_ids: (Ident::new("INVALID", Span::call_site()), Vec::new()),
            non_term_type: TokenStream::new(),
        }
    }
}




// Compile Enums: state, non-terminals, rules ---------------------------------

/// Compiles to give the enum representing parser state
/// 
/// resulting enum is:
/// ```` ignore
/// enum {
///     __0,
///     __1,
///     __2,
///     ...
///     __<num_states-1>,
/// }
/// ````
/// Then stores identifiers in the given state_context
fn compile_parser_state(parser_desc: &ast::Parser, state_context: &mut ParserStateContext) -> TokenStream {
    // Generate Identifier for enum name
    let mut enum_ident = "__".to_string();
    enum_ident.push_str(parser_desc.parser_name.to_string().as_str());
    enum_ident.push_str("State");
    state_context.ident = Ident::new(enum_ident.as_str(), Span::call_site());
    
    // Generate identifier for each state as number prefixed by "__"
    let mut state_ident: String = "__".to_string();
    for i in 0..parser_desc.num_states {
        state_ident.push_str(i.to_string().as_str());
        state_context.idx.push(Ident::new(state_ident.as_str(), Span::call_site()));
        state_ident.truncate(2);
    }

    // Assemble and return tokenstream
    let name = &state_context.ident;
    let idxs = &state_context.idx;
    quote!{
        #[allow(non_camel_case_types)]
        #[derive(Clone, Copy)]
        enum #name {
            #(
                #[allow(non_camel_case_types)]
                #idxs,
            )*
        }
    }
}

/// Compiles to give the enum representing parser non-terminals
/// 
/// resulting enum is:
/// ```` ignore
/// enum __<ParserName>NonTerminal {
///     <NonTerm1>,
///     <NonTerm2>,
///     ...
/// }
/// ````
/// Then stores identifiers in the given state_context
fn compile_parser_non_terminals(parser_name: &Ident, non_terms:  &Vec<Ident>, state_context: &mut (Ident, Vec<Ident>)) -> TokenStream {
    state_context.1 = non_terms.clone();
    state_context.0 = Ident::new(&format!("__{parser_name}NonTerminal"), parser_name.span());
    let enum_id_ref = &state_context.0;
    let non_term_ref = &state_context.1;

    quote!{
        #[allow(non_camel_case_types)]
        enum #enum_id_ref {
            #(
                #[allow(non_camel_case_types)]
                #non_term_ref,
            )*
        }
    }
}

/// Compiles to give the enum representing parser rules
/// 
/// resulting enum is:
/// ```` ignore
/// enum __<ParserName>Rules {
///     <Rule1>_<Prod1>,
///     <Rule1>_<Prod2>,
///     <Rule2>_<Prod1>,
///     <Rule2>_<Prod2>,
///  ...
/// ````
/// Then stores identifiers in the given state_context as 2D vector indexed by \[rule_idx]\[prod_idx]
fn compile_parser_rules(parser_desc: &ast::Parser, parser_rule_ids: &mut (Ident, Vec<Vec<Ident>>)) -> TokenStream {
    parser_rule_ids.0 = Ident::new(&format!("__{}Rules", parser_desc.parser_name), parser_desc.parser_name.span());
    let mut rule_id;
    for (rule_idx, rule) in parser_desc.rules.iter().enumerate() {
        let base_len = parser_desc.non_terminals[rule_idx].to_string().len(); // Length of rule id without prod num
        let mut v = Vec::new();
        rule_id = parser_desc.non_terminals[rule_idx].to_string();
        for prod_idx in 0..rule.prods.len() {
            rule_id.push_str(format!("_{prod_idx}").as_str());
            v.push(Ident::new(rule_id.as_str(), Span::call_site()));
            rule_id.truncate(base_len);
        }
        parser_rule_ids.1.push(v);     
    }
    
    let enum_id_ref = &parser_rule_ids.0;
    let enum_entry_ref = &parser_rule_ids.1;
    quote!{
        #[allow(non_camel_case_types)]
        enum #enum_id_ref {
            
            #(#(
                #[allow(non_camel_case_types)]
                #enum_entry_ref
            ),*),*
        }
    }
}

// Compile functions: reduction -----------------------------------------------

/// Translate parser rules into exec function parameter types,
/// Fills context and returns a pair of 2 2D vectors of identifier and references to the type stored in context
///
///  organization is \[rule_idx]\[prod_idx]
/// 
/// indentifiers take the form of
/// 
/// "\_\_execute_<rule_ident>_<rule_idx>"
// 
///```` ignore
/// Rule foo() -> String {
///      ... => {...}
///      ... => {...}
/// }
/// ````
/// produces exec rules:
/// \_\_execute_foo_0 and \_\_execute_foo_1
fn translate_exec_params<'a, 'b: 'a>(parser_desc: &ast::Parser, context_exec_param_types: &'b mut HashMap<(usize, usize), Vec<TokenStream>>) -> (Vec<Vec<TokenTree>>, Vec<Vec<&'a TokenStream>>) {
    let mut idents: Vec<Vec<TokenTree>> = Vec::new();
    let mut types: Vec<Vec<&TokenStream>> = Vec::new();

    // Substantiate idents vec and contex_exec_param_types
    for (r_idx, r) in parser_desc.rules.iter().enumerate() {
        for (prod_idx, prod) in r.prods.iter().enumerate() {
            let mut ident: Vec<TokenTree> = Vec::new();
            let mut param_types: Vec<TokenStream> = Vec::new();
            for (param_ident, param_type_ref) in prod.params.iter() {
                match param_ident {
                    None => 
                        ident.push(TokenTree::Punct(Punct::new('_', proc_macro2::Spacing::Alone))),
                    Some(param_ident) => 
                        ident.push(TokenTree::Ident(param_ident.clone())),
                }
                param_types.push(parser_desc.rules[*param_type_ref].return_type.clone());
            }
            idents.push(ident);
            context_exec_param_types.insert((r_idx, prod_idx), param_types);
        }
    }

    // Substantiate types vec
    for (non_term_idx, r) in parser_desc.rules.iter().enumerate() {
        for (prod_idx, prod) in r.prods.iter().enumerate() {
            let mut v: Vec<&TokenStream> = Vec::new();
            for param_idx in 0..prod.params.len() {
                v.push(context_exec_param_types.get(&(non_term_idx, prod_idx)).unwrap().get(param_idx).unwrap())
            }
            types.push(v);
        }
    }
    
    (idents, types)
}

/// Translate parser rules into exec and redunce function identifers,
/// Fills context and returns vector of references to context (sorted by non_term idx then production idx)
/// 
/// indentifiers take form of
/// 
/// "execute_<rule_ident>_<rule_idx>"
/// 
/// "reduce_<rule_ident>_<rule_idx>"
/// 
/// Example:
// 
///```` ignore
/// Rule foo() -> String {
///      ... => {...}
///      ... => {...}
/// }
/// ````
/// produces exec rules:
/// ( {execute_foo_0, execute_food_1}, {reduce_foo_0, reduce_foo_1} )
fn translate_exec_reduce_idents<'a, 'b: 'a>(parser_desc: &ast::Parser, context_exec_ids: &'b mut IntMap<usize, Vec<Ident>>, context_reduce_ids: &'b mut IntMap<usize, Vec<Ident>>) -> (Vec<&'a Ident>, Vec<&'a Ident>) {
    // Generate idents and store in context
    let mut e_string = "execute_".to_string();
    let mut r_string: String = "reduce_".to_string();
    for r in parser_desc.rules.iter() {
        let container_exec = context_exec_ids.entry(r.non_term_idx).or_insert(Vec::new());
        let container_reduce = context_reduce_ids.entry(r.non_term_idx).or_insert(Vec::new());
        for i in 0..r.prods.len() {
            e_string.push_str(&parser_desc.non_terminals[r.non_term_idx].to_string().as_str());
            e_string.push('_');
            e_string.push_str(i.to_string().as_str());
            r_string.push_str(e_string.as_str()["execute_".len()..].as_ref());

            container_exec.push(Ident::new(&e_string, Span::call_site()));
            container_reduce.push(Ident::new(&r_string, Span::call_site()));
            e_string.truncate("execute_".len());
            r_string.truncate("reduce_".len()); 
        }
    }

    // Move to vectors of references of the data from context
    let mut exec_vec: Vec<&'a Ident> = Vec::new();
    let mut reduce_vec: Vec<&'a Ident> = Vec::new();
    for (non_term_idx, r) in parser_desc.rules.iter().enumerate() {
        let exec_ids_by_rule = context_exec_ids.get(non_term_idx).unwrap();
        let reduce_ids_by_rule = context_reduce_ids.get(non_term_idx).unwrap();
        for prod_idx in 0..r.prods.len() {
            exec_vec.push(exec_ids_by_rule.get(prod_idx).unwrap());
            reduce_vec.push(reduce_ids_by_rule.get(prod_idx).unwrap());
        }   
    }
    (exec_vec, reduce_vec)
}


/// compile reduction functions and a helper execute function
/// 
/// the execute function performs the action defined in the grammar,
/// 
/// the reduce function pops the required arguments from the value stack and calls the execute function,
/// 
/// The structure of the generated code is:
/// ```ignore
/// fn __execute_<rule_ident>_<rule_idx>(<param1>: <type1>, <param2>: <type2>, ...) -> <return_type> {
///     <exec_group>
/// }
/// fn __reduce_<rule_ident>_<rule_idx>(&mut self) {
///     let <param1> = self.value_stack.pop().unwrap().downcast_ref::< <type1> >().unwrap().to_owned();
///     let <param2> = self.value_stack.pop().unwrap().downcast_ref::< <type2> >().unwrap().to_owned();
///     ...
///     self.value_stack.push(Box::new(Parser::__execute_<rule_ident>_<rule_idx>(<param1>, <param2>, ...)));
/// }
/// ```
fn compile_exec_rules(parser_desc: &ast::Parser, context: &mut Context) -> TokenStream {
    let (exec_fn_idents, reduce_fn_idents) = translate_exec_reduce_idents(parser_desc, &mut context.exec_ids, &mut context.reduce_ids);
    let (exec_param_idents, exec_param_types) = translate_exec_params(parser_desc, &mut context.exec_param_types);
    
    let mut return_type: Vec<&TokenStream> = Vec::new();
    let mut exec_group: Vec<&Group> = Vec::new();
    for rule in parser_desc.rules.iter() {
        for prod in rule.prods.iter() {
            return_type.push(&rule.return_type);
            exec_group.push(&prod.exec);
        }
    }

    let parser_name = &parser_desc.parser_name;
    quote! {
        #(
            #[allow(non_snake_case)]
            fn #exec_fn_idents ( #(mut #exec_param_idents: #exec_param_types),* ) -> #return_type 
                #exec_group
            #[allow(non_snake_case)]
            fn #reduce_fn_idents (&mut self) {
                #(let #exec_param_idents = self.value_stack.pop().unwrap().downcast_ref::<#exec_param_types>().unwrap().to_owned();)*
                self.value_stack.push(Box::new((#parser_name ::#exec_fn_idents(#(#exec_param_idents),*))));
            }
        )*
    }
}


/// Compile the goto function for the parser using the non-terminal transitions from the parser description
///    
/// resulting function is:
/// ``` ignore
/// fn goto(state: <StateEnumIdent>, non_term: <NonTermEnumIdent>) -> <StateEnumIdent> {
///     match state {
///         <StateIdent> => match non_term {
///             <NonTerm1> => <StateIdent>,
///             <NonTerm2> => <StateIdent>,
///             ...
///         }
///         ...
///     }
/// }
/// ```
fn compile_goto_fn(parser_desc: &ast::Parser, context: &Context) -> TokenStream {
    // TODO: handle _ case for each state ? idk if there
    let state_enum_ref  = &context.state.ident;
    let non_term_enum_ref = &context.non_terms.0;
    
    let mut ret_token_stream = quote! {
        fn goto(state: #state_enum_ref, non_term: #non_term_enum_ref ) -> #state_enum_ref
    };
    let mut fn_group: TokenStream = quote! { match state };
    let mut state_match_group: TokenStream  = TokenStream::new();

    assert!(parser_desc.non_terminal_transitions.len() == parser_desc.num_states);

    for (state_idx, non_term_transitions) in parser_desc.non_terminal_transitions.iter().enumerate() {
        if non_term_transitions.is_empty() {
            continue;
        }
        let state_idx: &Ident = &context.state.idx[state_idx];
        state_match_group.extend(quote! {
            #state_enum_ref :: #state_idx => match non_term
        });

        let mut non_term_match_group: TokenStream = TokenStream::new();

        for (non_term_idx, target_state_idx) in non_term_transitions.iter() {
            let non_term_ref: &Ident = &context.non_terms.1[*non_term_idx];
            let state_ref: &Ident = &context.state.idx[*target_state_idx];
            non_term_match_group.extend(quote! {
                #non_term_enum_ref :: #non_term_ref => #state_enum_ref :: #state_ref,
            });
        }
        state_match_group.append(Group::new(proc_macro2::Delimiter::Brace, non_term_match_group));
        state_match_group.extend(quote! {,});


    }
    state_match_group.extend(quote! {
        _ => {panic!("No valid goto transition")}
    });

    fn_group.append(Group::new(proc_macro2::Delimiter::Brace, state_match_group));
    ret_token_stream.append(Group::new(proc_macro2::Delimiter::Brace, fn_group));
    

    ret_token_stream
    
}

fn compile_step_fn(parser_desc: &ast::Parser, context: &Context) -> TokenStream {


    // TODO: handle _ case for each state 

    let state_enum_ref  = &context.state.ident;
    // let non_term_enum_ref = &context.non_terms.0;
    let non_term_type: &TokenStream = &context.non_term_type;
    let mut ret_token_stream = quote! {
        fn step(&mut self, next: #non_term_type)
    };
    let mut fn_group: TokenStream = quote! { match self.state };
    let mut state_match_group: TokenStream  = TokenStream::new();

    assert!(parser_desc.terminal_transitions.len() == parser_desc.num_states);

    for (state_idx, term_transitions) in parser_desc.terminal_transitions.iter().enumerate() {
        if term_transitions.is_empty() {
            continue;
        }
        let state_idx: &Ident = &context.state.idx[state_idx];
        state_match_group.extend(quote! {
            #state_enum_ref :: #state_idx => match next
        });

        let mut term_match_group: TokenStream = TokenStream::new();

        let non_term_enum_ref = &context.non_terms.0;

        for (term_ident, action) in term_transitions.iter() {
            match action {
                ast::Action::Shift(target_state_idx) => {
                    let state_ref: &Ident = &context.state.idx[*target_state_idx];
                    term_match_group.extend(quote! {
                        #term_ident => {
                            self.state = #state_enum_ref :: #state_ref;
                            self.state_stack.push(self.state);
                        },
                    });
                },
                ast::Action::Reduce((rule_idx, prod_idx)) => {
                    let reduce_fn_ident: &Ident = &context.reduce_ids.get(*rule_idx).unwrap()[*prod_idx];
                    let token_pop_count = parser_desc.rules[*rule_idx].prods[*prod_idx].tokens_count;
                    let non_term_ident: &Ident = &context.non_terms.1[parser_desc.rules[*rule_idx].non_term_idx];
                    let parser_ident = &parser_desc.parser_name;
                    term_match_group.extend(quote! {
                        #term_ident => {
                            self. #reduce_fn_ident ();
                            self.state_stack.truncate(self.state_stack.len()- #token_pop_count );
                            self.state = #parser_ident::goto(self.state_stack.last().unwrap().clone(), #non_term_enum_ref :: #non_term_ident);
                            self.state_stack.push(self.state);
                            self.step(next);
                        },
                    });
                },
            }
        }
        term_match_group.extend(quote!{_ => panic!("Unexpected token read")});
        state_match_group.append(Group::new(proc_macro2::Delimiter::Brace, term_match_group));
        state_match_group.extend(quote! {,});

    }
    state_match_group.extend(quote! {
        _ => {panic!("No valid goto transition")}
    });

    fn_group.append(Group::new(proc_macro2::Delimiter::Brace, state_match_group));
    ret_token_stream.append(Group::new(proc_macro2::Delimiter::Brace, fn_group));
    

    ret_token_stream

}

fn compile_end_of_input_fn(parser_desc: &ast::Parser, context: &Context) -> TokenStream {
    let mut return_stream = quote! { fn end_of_input(&mut self) -> bool };
    let mut fn_group = quote! {match self.state };
    let mut state_match_group: TokenStream  = TokenStream::new();

    assert!(parser_desc.eoi_actions.len() == parser_desc.num_states);

    let state_enum_ref  = &context.state.ident;
    let non_term_enum_ref = &context.non_terms.0;
    for (state_idx, eoi_action) in parser_desc.eoi_actions.iter().enumerate() {
        let state_idx: &Ident = &context.state.idx[state_idx];
        match eoi_action {
            ast::EOIAction::Acceptance(is_accepted) => {
                state_match_group.extend(quote! {
                    #state_enum_ref :: #state_idx => #is_accepted,
                });
            },
            ast::EOIAction::Reduce((rule_idx, prod_idx)) => {
                let reduce_fn_ident: &Ident = &context.reduce_ids.get(*rule_idx).unwrap()[*prod_idx];
                let token_pop_count = parser_desc.rules[*rule_idx].prods[*prod_idx].tokens_count;
                let non_term_ident: &Ident = &context.non_terms.1[parser_desc.rules[*rule_idx].non_term_idx];
                let parser_ident = &parser_desc.parser_name;
                state_match_group.extend(quote! {
                    #state_enum_ref :: #state_idx => {
                        self.#reduce_fn_ident();
                        self.state_stack.truncate(self.state_stack.len()- #token_pop_count );
                        self.state = #parser_ident::goto(*self.state_stack.last().unwrap(), #non_term_enum_ref :: #non_term_ident);
                        self.state_stack.push(self.state);
                        self.end_of_input()
                    },
                });
            },
        }
    }
    fn_group.append(Group::new(proc_macro2::Delimiter::Brace, state_match_group));
    return_stream.append(Group::new(proc_macro2::Delimiter::Brace, fn_group));
    return_stream

    
}



pub fn compile_parser(parser_desc: &ast::Parser) -> TokenStream {
    let mut con: Context = Context::new();


    // Enums
    let parser_state = compile_parser_state(parser_desc, &mut con.state);
    let non_terms = compile_parser_non_terminals(&parser_desc.parser_name, &parser_desc.non_terminals, &mut con.non_terms);
    let parser_rules = compile_parser_rules(parser_desc, &mut con.parser_rule_ids);

    con.non_term_type = parser_desc.element_type.clone();

    // parser impl
    let reductions = compile_exec_rules(parser_desc, &mut con);
    let goto: TokenStream = compile_goto_fn(parser_desc, &con);
    let step: TokenStream = compile_step_fn(parser_desc, &con);
    let end_of_input: TokenStream = compile_end_of_input_fn(parser_desc, &con);

    // Parser struct
    let parser_name = &parser_desc.parser_name;
    let state_type = &con.state.ident;

    let return_type: &TokenStream = &parser_desc.rules[parser_desc.init_rule].return_type;
    let input_type: &TokenStream = &parser_desc.input_type;
    let iter_fn: &TokenStream = &parser_desc.iterator_fn;

    // Impliments parser expanded as is parser_tests.rs file
    quote! {
        #parser_state
        #non_terms
        #parser_rules

        pub struct #parser_name {
            state: #state_type,
            value_stack: Vec<Box<dyn Any>>,
            state_stack: Vec<#state_type>,
        }

        impl #parser_name {
            pub fn new() -> Self {
                #parser_name {
                    state: #state_type::__0,
                    value_stack: Vec::new(),
                    state_stack: vec!{#state_type::__0},
                }
            }

            #reductions
            #goto
            #step
            #end_of_input

            pub fn parse(&mut self, input: #input_type ) -> #return_type {
                for t in input.#iter_fn {
                    self.step(t);
                }
                if !self.end_of_input() {
                    println!("Error input not valid");
                    String::new()
                } else {
                    self.value_stack.pop().unwrap().downcast_ref::<String>().unwrap().to_owned()
                }
            }

            pub fn get_value(&self) -> Option<&Box<dyn Any>> {
                if self.value_stack.is_empty() {
                    None
                } else {
                    Some(self.value_stack.last().unwrap())
                }
            }
        }
    }
}

