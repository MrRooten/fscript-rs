#[cfg(test)]
pub mod tests {

    use std::{borrow::Cow, io::Read, time::Instant};

    use crate::{
        backend::{
            compiler::{bytecode::Bytecode, jit::cranelift::CraneLiftJitBackend},
            types::{
                base::{FSRObject, FSRValue},
                code::FSRCode,
                fn_def::FSRFn,
                iterator::FSRInnerIterator,
                list::FSRList,
                module::FSRModule,
            },
            vm::{
                thread::{CallFrame, FSRThreadRuntime},
                virtual_machine::FSRVM,
            },
        },
        frontend::ast::token::{
            base::{FSRPosition, FSRToken},
            module::FSRModuleFrontEnd,
        },
    };

    #[test]
    fn test_1() {
        let expr = "
        l = b.len()
        ";
        let meta = FSRPosition::new();
        let token = FSRModuleFrontEnd::parse(&expr.chars().collect::<Vec<char>>(), meta).unwrap();
        let v = Bytecode::load_ast("main", FSRToken::Module(token.0), token.1);
        println!("{:#?}", v);
    }

    #[test]
    fn test_2() {
        let expr = "
        a = 1
        println(type(a))
        ";
        let meta = FSRPosition::new();
        let token = FSRModuleFrontEnd::parse(&expr.chars().collect::<Vec<char>>(), meta).unwrap();
        let v = Bytecode::load_ast("main", FSRToken::Module(token.0), token.1);
        println!("{:#?}", v);
    }

    #[test]
    fn test_3() {
        let expr = "
        not b
        ";
        let meta = FSRPosition::new();
        let token = FSRModuleFrontEnd::parse(&expr.chars().collect::<Vec<char>>(), meta).unwrap();
        let v = Bytecode::load_ast("main", FSRToken::Module(token.0), token.1);
        println!("{:#?}", v);
    }

    #[test]
    fn test_global() {
        let expr = "
        print(true)
        ";
        let meta = FSRPosition::new();
        let token = FSRModuleFrontEnd::parse(&expr.chars().collect::<Vec<char>>(), meta).unwrap();
        let v = Bytecode::load_ast("main", FSRToken::Module(token.0), token.1);
        println!("{:#?}", v);
    }

    #[test]
    fn test_expr_method() {
        let s = "a.abc(1)\n";

        let meta = FSRPosition::new();
        let token = FSRModuleFrontEnd::parse(&s.chars().collect::<Vec<char>>(), meta).unwrap();
        let v = Bytecode::load_ast("main", FSRToken::Module(token.0), token.1);
        println!("{:#?}", v);
    }

    #[test]
    fn test_for_bc() {
        let expr = "
        fn abc() {
            return \"abc\".len()
        }
        
        a = 1 + abc()
        ";
        let meta = FSRPosition::new();
        let token = FSRModuleFrontEnd::parse(&expr.chars().collect::<Vec<char>>(), meta).unwrap();
        let v = Bytecode::load_ast("main", FSRToken::Module(token.0), token.1);
        println!("{:#?}", v);
    }

    #[test]
    fn test_if_else() {
        let expr = "
        if abc {
            println('if')
        } else if ddc {
            println('else if')
        } else {
            println('else')
        }
        ";
        let meta = FSRPosition::new();
        let token = FSRModuleFrontEnd::parse(&expr.chars().collect::<Vec<char>>(), meta).unwrap();
        let v = Bytecode::load_ast("main", FSRToken::Module(token.0), token.1);
        println!("{:#?}", v);
    }

    #[test]
    fn test_class_getter() {
        let expr = "
        class Abc {
            fn test() {
                println('123')
            }
        }

        Abc::test()
        ";
        let meta = FSRPosition::new();
        let token = FSRModuleFrontEnd::parse(&expr.chars().collect::<Vec<char>>(), meta).unwrap();
        let v = Bytecode::load_ast("main", FSRToken::Module(token.0), token.1);
        println!("{:#?}", v);
    }

    // #[test]
    // fn test_2() {
    //     let source_code = "
    //     a = 1
    //     while a < 3 {
    //         a = a + 1
    //     }
    //     println(a)
    //     ";
    //     let v = FSRModule::from_code("main", source_code).unwrap();
    //     let mut runtime = FSRThreadRuntime::new();
    //     let mut vm = FSRVM::new();
    //     runtime.set_vm(&mut vm);
    //     runtime.start(&v, &mut vm).unwrap();
    // }

    #[test]
    fn test_list() {
        let _ = FSRVM::single();
        let expr = "
        [1, 2, 3, \"abc\"]
        ";
        let meta = FSRPosition::new();
        let token = FSRModuleFrontEnd::parse(&expr.chars().collect::<Vec<char>>(), meta).unwrap();
        let v = Bytecode::load_ast("main", FSRToken::Module(token.0), token.1);
        println!("{:#?}", v);
    }


    #[test]
    fn test_new_object() {
        let s = Instant::now();
        let mut i = 0;
        let mut vs = Vec::with_capacity(300000);
        while i < 3000 {
            let v = Box::new(FSRObject::new());
            vs.push(v);
            i += 1;
            vs.pop();
        }
        let e = Instant::now();
        println!("{:#?}", e - s);
    }

    #[test]
    fn test_script() {
        FSRVM::single();
        let vs = vec![
            "test_script/test/test_class.fs",
            "test_script/test/test_expression.fs",
            "test_script/test/test_nested_call.fs",
            "test_script/test/test_closure.fs",
            "test_script/bench/bench_iter_filter.fs",
            "test_script/test/test_iter_enumerate.fs"
        ];
        for i in vs {
            println!("Running script: {}", i);
            let file = i;
            let mut f = std::fs::File::open(file).unwrap();
            let mut source_code = String::new();
            f.read_to_string(&mut source_code).unwrap();
            let mut obj: Box<FSRObject<'_>> = Box::new(FSRModule::new_object("main"));
            let obj_id = FSRVM::leak_object(obj);
            let v = FSRCode::from_code("main", &source_code, obj_id).unwrap();
            let obj = FSRObject::id_to_mut_obj(obj_id).unwrap();
            obj.as_mut_module().init_fn_map(v);
            let mut runtime = FSRThreadRuntime::new_runtime();

            let start = Instant::now();
            //runtime.start(&v, &mut vm).unwrap();

            runtime.start(obj_id, false).unwrap();
            let end = Instant::now();
            println!("{:?}", end - start);
        }
    }
    #[test]
    fn test_jit_script() {
        FSRVM::single();
        let vs = vec![
            "test_script/test/jit/alloc_array.fs",
            "test_script/test/jit/fib_test.fs",
            "test_script/test/jit/list_assign.fs",
            "test_script/test/jit/list_item_assign.fs",
            "test_script/test/jit/method_call.fs",
            "test_script/test/jit/test_ct_assign.fs",
            "test_script/test/jit/test_struct.fs",
        ];
        for i in vs {
            println!("Running script: {}", i);
            let file = i;
            let mut f = std::fs::File::open(file).unwrap();
            let mut source_code = String::new();
            f.read_to_string(&mut source_code).unwrap();
            let mut obj: Box<FSRObject<'_>> = Box::new(FSRModule::new_object("main"));
            let obj_id = FSRVM::leak_object(obj);
            let v = FSRCode::from_code("main", &source_code, obj_id).unwrap();
            let obj = FSRObject::id_to_mut_obj(obj_id).unwrap();
            obj.as_mut_module().init_fn_map(v);
            let mut runtime = FSRThreadRuntime::new_runtime();

            let start = Instant::now();
            //runtime.start(&v, &mut vm).unwrap();

            runtime.start(obj_id, false).unwrap();
            let end = Instant::now();
            println!("{:?}", end - start);
        }
    }

    #[test]
    fn test_obj_size() {
        /*
        #[derive(Debug, Clone)]
        pub enum FSRValue<'a> {
            Integer(i64),
            Float(f64),
            String(Cow<'a, str>),
            Class(Box<FSRClass>),
            ClassInst(Box<FSRClassInst<'a>>),
            Function(FSRFn<'a>),
            Bool(bool),
            List(FSRList),
            Iterator(FSRInnerIterator),
            None,
        }
                 */
        println!("FSRObject size: {}", std::mem::size_of::<FSRObject>());
        println!("CallFrame size: {}", std::mem::size_of::<CallFrame>());
        println!("FSRValue size: {}", std::mem::size_of::<FSRValue>());
        println!("Cowstr size: {}", std::mem::size_of::<Cow<str>>());
        println!("FSRFn size: {}", std::mem::size_of::<FSRFn>());
        println!("FSRList size: {}", std::mem::size_of::<FSRList>());
        println!(
            "FSRInnerIterator size: {}",
            std::mem::size_of::<FSRInnerIterator>()
        );
        println!("u8 size: {}", std::mem::size_of::<u8>());
        // test anyhow result
        println!("Result size: {}", std::mem::size_of::<anyhow::Result<()>>());
    }

    #[test]
    fn test_module() {
        let _ = FSRVM::single();
        let module1 = r#"
        fn abc() {
            println('abc')
        }

        abc()
        "#;
        let mut obj: Box<FSRObject<'_>> = Box::new(FSRModule::new_object("main"));
        let obj_id = FSRVM::leak_object(obj);
        let v = FSRCode::from_code("main", module1, obj_id).unwrap();
        let obj = FSRObject::id_to_mut_obj(obj_id).unwrap();
        obj.as_mut_module().init_fn_map(v);
        //let obj_id = FSRVM::leak_object(obj);
        let mut runtime = FSRThreadRuntime::new_runtime();
        //runtime.start(&v, &mut vm).unwrap();

        runtime.start(obj_id, false).unwrap();
    }

    #[test]
    fn test_expr2() {
        let _ = FSRVM::single();
        let module1 = r#"
        b = 10 + -1 * 10
        println(b)
        "#;

        let mut obj: Box<FSRObject<'_>> = Box::new(FSRModule::new_object("main"));
        let obj_id = FSRVM::leak_object(obj);
        let v = FSRCode::from_code("main", module1, obj_id).unwrap();
        let obj = FSRObject::id_to_mut_obj(obj_id).unwrap();
        obj.as_mut_module().init_fn_map(v);
        let mut runtime = FSRThreadRuntime::new_runtime();

        //runtime.start(&v, &mut vm).unwrap();

        runtime.start(obj_id, false).unwrap();
    }

    #[test]
    fn test_simple() {
        let _ = FSRVM::single();
        let module1 = r#"
        a = true or false
        "#;

        let mut obj: Box<FSRObject<'_>> = Box::new(FSRModule::new_object("main"));
        let obj_id = FSRVM::leak_object(obj);
        let v = FSRCode::from_code("main", module1, obj_id).unwrap();
        let obj = FSRObject::id_to_mut_obj(obj_id).unwrap();
        obj.as_mut_module().init_fn_map(v);

        let mut runtime = FSRThreadRuntime::new_runtime();

        //runtime.start(&v, &mut vm).unwrap();

        runtime.start(obj_id, false).unwrap();
    }

}
