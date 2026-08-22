use super::ast;



moll::moll_define!{

    Parser MollParser for MollTokenIter -> ast::MollRoot {

        Rule Surface() -> ast::Parser {
            "Parser" parser_name:moll::Ident "for" return_type:moll::RSType() '{' body:ParserElement() '}' MyEnum->A() => {
                ast::Parser{name: parser_name, return_type: return_type, parser_elements: body}
            }
        }

        Rule ParserElement() -> ast::Rule { // Do I want to do this to make varients easier? could make typing harder
            "Rule" rule_name:moll::Ident "->" return_type:Type() '{' body:RuleElement() '}' => {
                ParserElement::Rule(
                    Rule{
                        name: rule_name, 
                        return_type: return_type,
                        productions: body,
                    }
                )
            }
            "Group" group_name:moll::Ident "->" return_type:Type() '{' body:ParserElement()* '}' => {
                ParsrElment::Group{
                    name: group_name,
                    return_type: return_type,
                    rules: body,
                }
            } 
        }
        
        Group RuleElement() -> ast::Production {

            Rule RuleElement() -> ast::Production {
                prod_step:second:BaseProductionStep()* "=>" code_block:moll::CodeBlock() => {
                    prod_step
                }
            }

            /// param_name:namespace::this
            /// ProductionStep -> param:Captured
            /// ProdcutionStep -> (ProductionStep*)    Captured -> (Captured*)
            /// ProdcutionStep -> ProductionStep */+    Captured -> Captured */+
            /// ProdcutionStep -> ProductionStep?    Captured -> Captured?
            /// ProductionStep | Captured -> namespace::NameSpaced
            /// ProductionStep | Captured | Namespaced -> const | rule()
            /// (NOT IMPLIMENTED) ProductionStep | Captured -> This()
            /// ProdcutionStep | Captured -> "literal"
            /// 

            Rule BaseProductionStep() -> ast::BaseProduction {
                param:moll::Ident ':' prod:Captured() => {

                }
                '(' prods:ProductionStep()* ')' => {

                }
                prod:ProductionStep() '*' => {

                } 
                prod:ProductionStep() '+' => {

                } 
                prod:ProductionStep() '?' => {

                }
                namespace:moll::Ident "::" prod:Namespaced() => {

                }
                const_terminal:moll::Ident => {

                }
                rule:moll::Ident "()" => {

                }
                literal:moll::StringConst => {

                }
                literal:moll::CharacterConst => {

                }
            }
            /// param_name:namespace::this
            /// ProductionStep -> param:Captured
            /// ProdcutionStep -> (ProductionStep*)    Captured -> (Captured*)
            /// ProdcutionStep -> ProductionStep */+    Captured -> Captured */+
            /// ProdcutionStep -> ProductionStep?    Captured -> Captured?
            /// ProductionStep | Captured | NameSpaced -> namespace::NameSpaced
            /// ProductionStep | Captured | Namespaced -> const | rule()
            /// (NOT IMPLIMENTED) ProductionStep | Captured -> This()
            /// ProdcutionStep | Captured -> "literal"
            /// 
            Rule Captured() => ast::CapturedProduction {
                '(' prods:ProductionStep()* ')' => {

                }
                prod:ProductionStep() '*' => {

                } 
                prod:ProductionStep() '+' => {

                } 
                prod:ProductionStep() '?' => {

                }
                namespace:moll::Ident "::" prod:Namespaced() => {

                }
                const_terminal:moll::Ident => {

                }
                rule:moll::Ident "()" => {

                }
                literal:moll::StringConst => {

                }
                literal:moll::CharacterConst => {

                }
            }

            Rule NameSpaced() => ast::NameSpacedProduction {
                namespace:moll::Ident "::" prod:Namespaced() => {

                }
                const_terminal:moll::Ident => {

                }
                rule:moll::Ident "()" => {

                }
                literal:moll::StringConst => {

                }
                literal:moll::CharacterConst => {

                }
            }
        }
    }
}