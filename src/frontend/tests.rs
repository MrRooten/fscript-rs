#[cfg(test)]
mod frontend_tests {
    use std::collections::HashMap;
    use std::time::Instant;

    use crate::frontend::ast::parse::ASTParser;
    use crate::frontend::ast::token::base::FSRPosition;
    use crate::frontend::ast::token::block::FSRBlock;
    use crate::frontend::ast::token::class::FSRClassFrontEnd;
    use crate::frontend::ast::token::for_statement::FSRFor;
    use crate::frontend::ast::token::function_def::FSRFnDef;
    use crate::frontend::ast::token::if_statement::FSRIf;
    use crate::frontend::ast::token::import::FSRImport;
    use crate::frontend::ast::token::module::FSRModuleFrontEnd;
    use crate::frontend::ast::token::while_statement::FSRWhile;
    use crate::frontend::ast::token::ASTContext;
    use crate::frontend::ast::token::{base::FSRToken, expr::FSRExpr};
    use crate::frontend::ast::utils::automaton::{FSTrie, NodeType};

    #[test]
    fn expr_test() {
        let s = "a + b + c\n";
        let meta = FSRPosition::new();
        let mut context = ASTContext::new_context();
        let expr = FSRExpr::parse(s.as_bytes(), false, meta, &mut context).unwrap();

        println!("{:#?}", expr);
    }

    #[test]
    fn test_expr_method() {
        let s = "a.abc(1)\n";

        let meta = FSRPosition::new();
        let mut context = ASTContext::new_context();
        let expr = FSRExpr::parse(s.as_bytes(), false, meta, &mut context).unwrap();

        println!("{:#?}", expr);
    }

    #[test]
    fn not_expr_test() {
        let s = "not not not c\n";
        let meta = FSRPosition::new();
        let mut context = ASTContext::new_context();
        let expr = FSRExpr::parse(s.as_bytes(), false, meta, &mut context).unwrap();

        println!("{:#?}", expr);
    }

    #[test]
    fn test_empty_expr() {
        let s = "( )\n";
        let meta = FSRPosition::new();
        let mut context = ASTContext::new_context();
        let expr = FSRExpr::parse(s.as_bytes(), false, meta, &mut context).unwrap();
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
        let mut context = ASTContext::new_context();
        let expr = FSRExpr::parse(s.as_bytes(), false, meta, &mut context).unwrap();

        println!("{:#?}", expr)
    }

    #[test]
    fn test_assign() {
        let s = "a = 1 > 3 && 1 < 3";
        let meta = FSRPosition::new();
        let mut context = ASTContext::new_context();
        let expr = FSRExpr::parse(s.as_bytes(), false, meta, &mut context).unwrap();
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
        let mut context = ASTContext::new_context();
        let v = ASTParser::read_valid_bracket(s.as_bytes(), meta).unwrap();
        assert_eq!(v, s.len());

        let s = "abc(abcd['abc'])";
        let meta = FSRPosition::new();
        let mut context = ASTContext::new_context();
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
        let mut context = ASTContext::new_context();
        let b = FSRBlock::parse(s.as_bytes(), meta, &mut context).unwrap();
        println!("{:#?}", b);
        assert_eq!(b.get_len(), s.len());
    }

    #[test]
    fn test_module() {
        let s = "
        b = [1, 2, 3]
        l = b.len()
        println(l)
        ";
        let meta = FSRPosition::new();
        let mut context = ASTContext::new_context();
        let b = FSRModuleFrontEnd::parse(s.as_bytes(), meta).unwrap();
        println!("{:#?}", b);
        assert_eq!(b.get_len(), s.len());
    }

    #[test]
    fn test_trie() {
        let mut t = FSTrie::new();
        let mut context = ASTContext::new_context();
        let n = t.match_token("if()".as_bytes()).unwrap();
        assert_eq!(n, &NodeType::IfState);
    }

    #[test]
    fn test_while() {
        let s = "while abc==123 {
            a = print(123)
            if a > 3 {
                continue
            }
        }
        ";
        let meta = FSRPosition::new();
        let mut context = ASTContext::new_context();
        let i = FSRWhile::parse(s.as_bytes(), meta, &mut context).unwrap();
        println!("{:#?}", i);
    }

    #[test]
    fn test_if() {
        let s = "if abc==123 {
            a = print(123)
            if abc {
                print(123)
            }
        }
        ";
        let meta = FSRPosition::new();
        let mut context = ASTContext::new_context();
        let i = FSRIf::parse(s.as_bytes(), meta, &mut context).unwrap();
        println!("{:#?}", i);
    }

    #[test]
    fn test_function() {
        let s = "fn abc(test) {
            while a + b {
                abc
            }
            if abc == 123 {
                print(abc)
            }
            return abc
        }";
        let meta = FSRPosition::new();
        let mut context = ASTContext::new_context();
        let i = FSRFnDef::parse(s.as_bytes(), meta, &mut context).unwrap();
        println!("{:#?}", i);
        assert_eq!(s.len(), i.get_len())
    }

    #[test]
    fn test_comma() {
        let s = "('abc',123, dfds, (abc, 123))";
        let meta = FSRPosition::new();
        let mut context = ASTContext::new_context();
        let d = FSRExpr::parse(s.as_bytes(), false, meta, &mut context).unwrap();
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
        let mut context = ASTContext::new_context();
        let s = FSRExpr::parse(s.as_bytes(), false, meta, &mut context).unwrap();
        println!("{:#?}", s);
    }

    #[test]
    fn test_list2() {
        let s = "a == [(1+1),2,3,4]";
        let meta = FSRPosition::new();
        let mut context = ASTContext::new_context();
        let s = FSRExpr::parse(s.as_bytes(), false, meta, &mut context).unwrap();
        println!("{:#?}", s);
    }

    #[test]
    fn test_module_name() {
        let s = "path::test('adf')";
        let meta = FSRPosition::new();
        let mut context = ASTContext::new_context();
        let s = FSRExpr::parse(s.as_bytes(), false, meta, &mut context).unwrap();
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
        let mut context = ASTContext::new_context();
        let s = FSRClassFrontEnd::parse(s.as_bytes(), meta, &mut context).unwrap();
        println!("{:#?}", s);
    }

    #[test]
    fn test_if_else() {
        let s = "
        if abc == 123 {
            
        } else {
            abc = 1
        }
        ";
        let meta = FSRPosition::new();
        let mut context = ASTContext::new_context();
        let s = FSRModuleFrontEnd::parse(s.as_bytes(), meta).unwrap();
        println!("{:#?}", s);
    }

    #[test]
    fn test_logic_else() {
        let s = "a > 3 && b < 4";
        let meta = FSRPosition::new();
        let mut context = ASTContext::new_context();
        let s = FSRExpr::parse(s.as_bytes(), false, meta, &mut context).unwrap();
        println!("{:#?}", s);
    }

    #[test]
    fn test_for() {
        let s = "for abc in [1, 2, 3] {

        }
        ";
        let meta = FSRPosition::new();
        let mut context = ASTContext::new_context();
        let s = FSRFor::parse(s.as_bytes(), meta, &mut context).unwrap();
        println!("{:#?}", s);
    }

    #[test]
    fn test_import() {
        let s = "import abc.def";
        let meta = FSRPosition::new();
        let mut context = ASTContext::new_context();
        let s = FSRImport::parse(s.as_bytes(), meta, &mut context).unwrap();
        println!("{:#?}", s);
    }

    #[test]
    fn test_bracket_in_string() {
        let a = "p(\"a(e) \")";
        let meta = FSRPosition::new();
        let mut context = ASTContext::new_context();
        let s = FSRExpr::parse(a.as_bytes(), false, meta, &mut context).unwrap();
        println!("{:#?}", s);
    }

    #[test]
    fn test_chars() {
        let c = "你好";
        for i in c.as_bytes() {
            println!("{}", i)
        }
    }

    #[test]
    fn test_comment() {
        let s = "
while i < b { # while test
    i = i + one
} # test
";
        let meta = FSRPosition::new();
        let i = FSRModuleFrontEnd::parse(s.as_bytes(), meta).unwrap();
        println!("{:#?}", i);
        assert_eq!(s.len(), i.get_len())
    }

    #[test]
    fn test_neg_number() {
        let s = "a = -1";
        let meta = FSRPosition::new();
        let mut context = ASTContext::new_context();
        let expr = FSRExpr::parse(s.as_bytes(), false, meta, &mut context).unwrap();

        println!("{:#?}", expr);
    }

    #[test]
    fn test_try_block() {
        let s = r#"
try {
    e = 123
    for i in [1,2,3] {
    }
} catch {
    e = get_error()
}
"#;

        let meta = FSRPosition::new();
        let i = FSRModuleFrontEnd::parse(s.as_bytes(), meta).unwrap();
        println!("{:#?}", i);
        assert_eq!(s.len(), i.get_len())
    }

    #[test]
    fn test_try_expr() {
        let code = "try { a = 1 + 1 } catch {}";
        let meta = FSRPosition::new();
        let i = FSRModuleFrontEnd::parse(code.as_bytes(), meta).unwrap();
        println!("{:#?}", i);
    }

    #[test]
    fn test_clousure() {
        let code = "fn abc() {
            a = 1
            fn bcd() {
                return a
            }
        }";
        let meta = FSRPosition::new();
        let i = FSRModuleFrontEnd::parse(code.as_bytes(), meta).unwrap();
        println!("{:#?}", i);
    }

    #[test]
    fn test_class_2() {
        let code = r#"
        class Ddc {
            fn __new__(self) {
                self.ddc = 123 + 1
                return self
            }
        }

        class Abc {
            fn __new__(self) {
                self.abc = Ddc()
                return self
            }

            fn __str__(self) {
                return 'return string'
            }

            fn test(self) {
                return 323
            }

            fn not_self() {
                println("not self")
                return 1
            }
        }



        a = Abc()
        println(a.__str__()) # will prin 'return string'


        if a.abc.ddc < 323 {
            a.abc.ddc = a.test() + a.abc.ddc
        }

        assert(a.abc.ddc == 447)

        println(a.abc.ddc)

        Abc::not_self()

        println(a)
        "#;
        let meta = FSRPosition::new();
        let i = FSRModuleFrontEnd::parse(code.as_bytes(), meta).unwrap();
        println!("{:#?}", i);
    }

    #[test]
    fn test_lambda() {
        let code = r#"
        fn abc() {
            a = 1
            b = 1
            ddc = || {
                return a + b
            }

            return ddc
        }



        a = abc()
        "#;
        let meta = FSRPosition::new();
        let i = FSRModuleFrontEnd::parse(code.as_bytes(), meta).unwrap();
        println!("{:#?}", i);
    }

    #[test]
    fn test_nested_call() {
        let code = r#"fn abc() {
    fn fib(n) {
        if n == 1 or n == 2 {
            return 1
        } else {
            return fib(n - 1) + fib(n - 2)
        }
    }
    result = fib(30)
    println(result)

    gc_info()
}

abc()
        "#;
        let meta = FSRPosition::new();
        let i = FSRModuleFrontEnd::parse(code.as_bytes(), meta).unwrap();
        println!("{:#?}", i);
    }

    #[test]
    fn test_expr() {
        let c = r#"b = 10 + -1 * 10"#;
        let meta = FSRPosition::new();
        let i = FSRModuleFrontEnd::parse(c.as_bytes(), meta).unwrap();
        println!("{:#?}", i);
    }

    #[test]
    fn test_assign2() {
        let c = r#"a[0] = 1"#;
        let meta = FSRPosition::new();
        let i = FSRModuleFrontEnd::parse(c.as_bytes(), meta).unwrap();
        println!("{:#?}", i);
    }

    #[test]
    fn test_call() {
        let c = r#"|x| {},1"#;
        let meta = FSRPosition::new();
        let i = FSRModuleFrontEnd::parse(c.as_bytes(), meta).unwrap();
        println!("{:#?}", i);
    }

    #[test]
    fn not_attr() {
        let c = r#"1 + -1"#;
        let meta = FSRPosition::new();
        let i = FSRModuleFrontEnd::parse(c.as_bytes(), meta).unwrap();
        println!("{:#?}", i);
    }
}
