#[cfg(test)]
pub mod tests {

    use std::{
        borrow::Cow,
        io::Read,
        time::Instant,
    };

    use crate::{
        backend::{
            compiler::bytecode::Bytecode,
            types::{
                base::{FSRObject, FSRValue},
                code::FSRCode,
                fn_def::FSRFn,
                iterator::FSRInnerIterator,
                list::FSRList,
                module::FSRModule,
            },
            vm::{
                thread::{CallFrame, FSRThreadRuntime, SValue},
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
        let token = FSRModuleFrontEnd::parse(expr.as_bytes(), meta).unwrap();
        let v = Bytecode::load_ast("main", FSRToken::Module(token));
        println!("{:#?}", v);
    }

    #[test]
    fn test_2() {
        let expr = "
        a = 1
        println(type(a))
        ";
        let meta = FSRPosition::new();
        let token = FSRModuleFrontEnd::parse(expr.as_bytes(), meta).unwrap();
        let v = Bytecode::load_ast("main", FSRToken::Module(token));
        println!("{:#?}", v);
    }

    #[test]
    fn test_3() {
        let expr = "
        not b
        ";
        let meta = FSRPosition::new();
        let token = FSRModuleFrontEnd::parse(expr.as_bytes(), meta).unwrap();
        let v = Bytecode::load_ast("main", FSRToken::Module(token));
        println!("{:#?}", v);
    }

    #[test]
    fn test_global() {
        let expr = "
        print(true)
        ";
        let meta = FSRPosition::new();
        let token = FSRModuleFrontEnd::parse(expr.as_bytes(), meta).unwrap();
        let v = Bytecode::load_ast("main", FSRToken::Module(token));
        println!("{:#?}", v);
    }

    #[test]
    fn test_expr_method() {
        let s = "a.abc(1)\n";

        let meta = FSRPosition::new();
        let token = FSRModuleFrontEnd::parse(s.as_bytes(), meta).unwrap();
        let v = Bytecode::load_ast("main", FSRToken::Module(token));
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
        let token = FSRModuleFrontEnd::parse(expr.as_bytes(), meta).unwrap();
        let v = Bytecode::load_ast("main", FSRToken::Module(token));
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
        let token = FSRModuleFrontEnd::parse(expr.as_bytes(), meta).unwrap();
        let v = Bytecode::load_ast("main", FSRToken::Module(token));
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
        let token = FSRModuleFrontEnd::parse(expr.as_bytes(), meta).unwrap();
        let v = Bytecode::load_ast("main", FSRToken::Module(token));
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
        [1, 2, 3]
        ";
        let meta = FSRPosition::new();
        let token = FSRModuleFrontEnd::parse(expr.as_bytes(), meta).unwrap();
        let v = Bytecode::load_ast("main", FSRToken::Module(token));
        println!("{:#?}", v);
    }

    #[test]
    fn test_while_backend() {
        let _ = FSRVM::single();
        let source_code = "
        fn test() {
            println('abc')
            dump(println)
        }

        i = 0
        while i < 10 {
            test()
            i = i + 1
        }

        assert(i == 10)
        
        ";
        let v = FSRCode::from_code("main", source_code).unwrap();
        let obj = Box::new(FSRModule::new_module("main", v));
        let obj_id = FSRVM::leak_object(obj);
        let mut runtime = FSRThreadRuntime::new();
        runtime.start(obj_id).unwrap();
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
            "test_script/test_class.fs",
            "test_script/test_expression.fs",
            "test_script/test_nested_call.fs",
            "test_script/test_error_handle.fs",
            "test_script/test_closure.fs"
        ];
        for i in vs {
            println!("Running script: {}", i);
            let file = i;
            let mut f = std::fs::File::open(file).unwrap();
            let mut source_code = String::new();
            f.read_to_string(&mut source_code).unwrap();
            let v = FSRCode::from_code("main", &source_code).unwrap();
            let obj = Box::new(FSRModule::new_module("main", v));
            let obj_id = FSRVM::leak_object(obj);
            let mut runtime = FSRThreadRuntime::new();

            let start = Instant::now();
            //runtime.start(&v, &mut vm).unwrap();

            runtime.start(obj_id).unwrap();
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
            Class(Box<FSRClass<'a>>),
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
        println!("SValue: {}", std::mem::size_of::<SValue>());
        println!("FSRValue size: {}", std::mem::size_of::<FSRValue>());
        println!("Cowstr size: {}", std::mem::size_of::<Cow<str>>());
        println!("FSRFn size: {}", std::mem::size_of::<FSRFn>());
        println!("FSRList size: {}", std::mem::size_of::<FSRList>());
        println!(
            "FSRInnerIterator size: {}",
            std::mem::size_of::<FSRInnerIterator>()
        );
        println!("u8 size: {}", std::mem::size_of::<u8>());
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
        let v = FSRCode::from_code("main", module1).unwrap();
        let obj = Box::new(FSRModule::new_module("main", v));
        let obj_id = FSRVM::leak_object(obj);
        let mut runtime = FSRThreadRuntime::new();
        //runtime.start(&v, &mut vm).unwrap();

        runtime.start(obj_id).unwrap();
    }

    #[test]
    fn test_expr2() {
        let _ = FSRVM::single();
        let module1 = r#"
        b = 10 + -1 * 10
        println(b)
        "#;
        
        let v = FSRCode::from_code("main", module1).unwrap();
        let obj = Box::new(FSRModule::new_module("main", v));
        let obj_id = FSRVM::leak_object(obj);
        let mut runtime = FSRThreadRuntime::new();


        //runtime.start(&v, &mut vm).unwrap();

        runtime.start(obj_id).unwrap();
    }

    // #[test]
    // fn test_suspend_thread() {
    //     let module1 = r#"
    //     i = 0
    //     while i < 10000000 {
    //         i = i + 1
    //     }

    //     println(i)
    //     "#;
    //     let v = FSRCode::from_code("main", module1).unwrap();
    //     let obj = Box::new(FSRModule::new_module("main", v));
    //     let obj_id = FSRVM::leak_object(obj);
    //     let vm = FSRVM::single();
    //     let runtime = Mutex::new(FSRThreadRuntime::new());
    //     let tid = vm.add_thread(runtime);
    //     let vm2 = vm.clone();
    //     //runtime.start(&v, &mut vm).unwrap();
    //     let th = std::thread::spawn(move || {
    //         vm2.clone()
    //             .get_thread(tid, |f| {
    //                 f.start(obj_id).unwrap();
    //             })
    //             .unwrap();
    //     });
    //     vm.stop_all_threads();
    //     std::thread::sleep(std::time::Duration::from_secs(2));
    //     println!("sleep 2 seconds");
    //     vm.continue_all_threads();
    //     th.join().unwrap();
        
    // }
}
