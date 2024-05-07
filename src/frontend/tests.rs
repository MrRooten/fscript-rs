
#[cfg(test)]
mod frontend_tests {
    use crate::frontend::ast::parse::ASTParser;
    use crate::frontend::ast::token::base::FSRPosition;
    use crate::frontend::ast::token::class::FSRClassFrontEnd;
    use crate::frontend::ast::token::while_statement::FSRWhile;
    use crate::frontend::ast::token::function_def::FSRFnDef;
    use crate::frontend::ast::token::if_statement::FSRIf;
    use crate::frontend::ast::token::{base::FSRToken, expr::FSRExpr};
    use crate::frontend::ast::token::block::FSRBlock;
    use crate::frontend::ast::token::module::FSRModuleFrontEnd;
    use crate::frontend::ast::utils::automaton::{FSTrie, NodeType};

    #[test]
    fn expr_test() {
        let s = "a + b + c\n";
        let meta = FSRPosition::new();
        let expr = FSRExpr::parse(s.as_bytes(), false, meta).unwrap();

        println!("{:#?}", expr);

    }

    #[test]
    fn test_empty_expr() {
        let s = "( )\n";
        let meta = FSRPosition::new();
        let expr = FSRExpr::parse(s.as_bytes(), false, meta).unwrap();
        if let FSRToken::EmptyExpr = expr.0 {
            //let e: FSRExpr = e.try_into().unwrap();
            
        } else {
            unimplemented!()
        }
    }


    #[test]
    fn test_obj_attr() {
        let s = "abc.name(abc, ddc)\n";
        let meta = FSRPosition::new();
        let expr = FSRExpr::parse(s.as_bytes(), false, meta).unwrap();

        println!("{:#?}", expr)
    }

    #[test]
    fn test_assign() {
        let s = "a = 'abcdefg'.len() + 3 - 2";
        let meta = FSRPosition::new();
        let expr = FSRExpr::parse(s.as_bytes(), false, meta).unwrap();
        if let FSRToken::Assign(e) = expr.0 {
            // let e: FSRExpr = e.try_into().unwrap();
            println!("{:?}", e)
        } else {
            unimplemented!()
        }
    }

    #[test]
    fn test_bracket() {
        let s = "(abcd['abc'])";
        let meta = FSRPosition::new();
        let v = ASTParser::read_valid_bracket(s.as_bytes(), meta).unwrap();
        assert_eq!(v, s.len());

        let s = "abc(abcd['abc'])";
        let meta = FSRPosition::new();
        let v = ASTParser::read_valid_name_bracket(s.as_bytes(), meta).unwrap();
        assert_eq!(v, s.len());
    }

    #[test]
    fn test_block() {
        let s = "
        {
            print(123)
            print(abc) + 123 + 54
            {
                print(123)
            }
        }
        ";
        let meta = FSRPosition::new();
        let b = FSRBlock::parse(s.as_bytes(), meta).unwrap();
        println!("{:#?}", b);
        assert_eq!(b.get_len(), s.len());
        
    }

    #[test]
    fn test_module() {
        let s = "
        b = 1 + 3
        c = 3 + 4
        ";
        let meta = FSRPosition::new();
        let b = FSRModuleFrontEnd::parse(s.as_bytes(), meta).unwrap();
        println!("{:#?}", b);
        assert_eq!(b.get_len(), s.len());

    }

    #[test]
    fn test_trie() {
        let mut t = FSTrie::new();
        let n = t.match_token("if()".as_bytes()).unwrap();
        assert_eq!(n, &NodeType::IfState);
    }

    #[test]
    fn test_for() {
        let s = 
        "while abc==123 {
            a = print(123)
        }
        ";
        let meta = FSRPosition::new();
        let i = FSRWhile::parse(s.as_bytes(), meta).unwrap();
        println!("{:#?}", i);
    }

    #[test]
    fn test_if() {
        let s = 
        "if abc==123 {
            a = print(123)
            if abc {
                print(123)
            }
        }
        ";
        let meta = FSRPosition::new();
        let i = FSRIf::parse(s.as_bytes(), meta).unwrap();
        println!("{:#?}", i);
    }

    #[test]
    fn test_function() {
        let s = 
        "fn abc(test) {
            while a + b {
                abc
            }
            if abc == 123 {
                print(abc)
            }
            return abc
        }";
        let meta = FSRPosition::new();
        let i = FSRFnDef::parse(s.as_bytes(), meta).unwrap();
        println!("{:#?}", i);
        assert_eq!(s.len(), i.get_len())
    }

    #[test]
    fn test_comma() {
        let s = "('abc',123, dfds, (abc, 123))";
        let meta = FSRPosition::new();
        let d = FSRExpr::parse(s.as_bytes(), false , meta).unwrap();
        println!("{:#?}", d);
    }

    #[test]
    fn read_comma() {
        let s = "abc(123,123)";
        let meta = FSRPosition::new();
        let s = ASTParser::split_by_comma(s.as_bytes(), meta).expect("TODO: panic message");
        println!("abc: {:?}", s)
    }

    #[test]
    fn test_list() {
        let s = "a = [(1+1),2,3,4]";
        let meta = FSRPosition::new();
        let s = FSRExpr::parse(s.as_bytes(), false,  meta).unwrap();
        println!("{:#?}", s);
    }

    #[test]
    fn test_module_name() {
        let s = "path::test('adf')";
        let meta = FSRPosition::new();
        let s = FSRExpr::parse(s.as_bytes(), false,  meta).unwrap();
        println!("{:#?}", s);
    }

    #[test]
    fn test_class() {
        let s = "class Abc {
            a = 1
            fn abc() {

            }
        }
        ";
        let meta = FSRPosition::new();
        let s = FSRClassFrontEnd::parse(s.as_bytes(),  meta).unwrap();
        println!("{:#?}", s);
    }
}