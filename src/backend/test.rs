#[cfg(test)]
pub mod tests {

    use std::{borrow::Cow, io::Read, time::Instant};

    use crate::{
        backend::{
            compiler::bytecode::{BinaryOffset, Bytecode},
            types::{base::{FSRObject, FSRValue}, fn_def::FSRFn, integer::FSRInteger, iterator::FSRInnerIterator, list::FSRList, module::FSRModule},
            vm::{runtime::FSRVM, thread::{CallFrame, FSRThreadRuntime, SValue}},
        },
        frontend::ast::token::{
            base::{FSRPosition, FSRToken}, module::FSRModuleFrontEnd
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
        let source_code = "
        fn test() {
            println('abc')
        }

        i = 0
        while i < 10 {
            test()
            i = i + 1
        }
        
        ";
        let v = FSRModule::from_code("main", source_code).unwrap();
        let base_module = FSRVM::leak_object(Box::new(v));
        let mut runtime = FSRThreadRuntime::new(base_module);
        let mut vm = FSRVM::new();
        runtime.set_vm(&mut vm);
        runtime.start(base_module, &mut vm).unwrap();
    }

    #[test]
    fn test_class() {
        FSRVM::new();
        let source_code = "
        class Abc {
    fn __new__(self) {
        self.abc = 0
        return self
    }
}

a = Abc()
a.abc = 1

dump(a)
        
        ";
        let v = FSRModule::from_code("main", source_code).unwrap();

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

    #[allow(unused)]
    pub fn benchmark_add() {
        // let source_code = "
        // a = 1
        // b = 1
        // while a < 3000000 {
        //     a = a + b
        // }

        // ";
        // let v = FSRModule::from_code("main", source_code).unwrap();
        let mut runtime = FSRThreadRuntime::new(0);
        let mut vm = FSRVM::new();
        let start = Instant::now();
        //runtime.start(&v, &mut vm).unwrap();

        runtime.set_vm(&mut vm);
        let obj = FSRInteger::new_inst(3);
        let obj2 = FSRInteger::new_inst(4);
        for _ in 0..3000000 {
            let v = FSRObject::invoke_offset_method(
                BinaryOffset::Add,
                &[FSRObject::obj_to_id(&obj), FSRObject::obj_to_id(&obj2)],
                &mut runtime,
                None,
            )
            .unwrap();

            match v {
                crate::backend::types::base::FSRRetValue::Value(fsrobject) => vm.allocator.free_object(fsrobject),
                crate::backend::types::base::FSRRetValue::GlobalId(_) => todo!(),
                crate::backend::types::base::FSRRetValue::GlobalIdTemp(_) => todo!(),
            }
        }
        let end = Instant::now();
        println!("{:?}", end - start);
    }

    #[allow(unused)]
    fn benchmark_compare() {
        // let source_code = "
        // a = 1
        // b = 1
        // while a < 3000000 {
        //     a = a + b
        // }

        // ";
        // let v = FSRModule::from_code("main", source_code).unwrap();
        let mut runtime = FSRThreadRuntime::new(0);
        let mut vm = FSRVM::new();
        let start = Instant::now();
        //runtime.start(&v, &mut vm).unwrap();

        runtime.set_vm(&mut vm);

        for _ in 0..3000000 {
            let obj = FSRInteger::new_inst(3);
            let obj2 = FSRInteger::new_inst(4);

            FSRObject::invoke_offset_method(
                BinaryOffset::Greater,
                &[FSRObject::obj_to_id(&obj), FSRObject::obj_to_id(&obj2)],
                &mut runtime,
                None,
            )
            .unwrap();
        }
        let end = Instant::now();
        println!("{:?}", end - start);
    }

    #[test]
    fn test_script() {
        let vs = vec![
            "test_script/test_class.fs",
            "test_script/test_expression.fs",
            "test_script/test_nested_call.fs"];
        for i in vs {
            let file = i;
            let mut f = std::fs::File::open(file).unwrap();
            let mut source_code = String::new();
            f.read_to_string(&mut source_code).unwrap();
            let v = FSRModule::from_code("main", &source_code).unwrap();
            let base_module = FSRVM::leak_object(Box::new(v));
            let mut runtime = FSRThreadRuntime::new(base_module);
            let mut vm = FSRVM::new();
            let start = Instant::now();
            runtime.start(base_module, &mut vm).unwrap();
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
        println!("FSRInnerIterator size: {}", std::mem::size_of::<FSRInnerIterator>());
        println!("u8 size: {}", std::mem::size_of::<u8>());
    }

    #[test]
    fn test_module() {
        let module1 = r#"
        fn abc() {
            println('abc')
        }

        abc()
        "#;
        let v = FSRModule::from_code("module1", module1).unwrap();
        let base_module = FSRVM::leak_object(Box::new(v));
        let mut runtime = FSRThreadRuntime::new(base_module);
        let mut vm = FSRVM::new();

        runtime.start(base_module, &mut vm).unwrap();
    }

}
