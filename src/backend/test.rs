#[cfg(test)]
mod tests {

    use std::time::Instant;

    use crate::{
        backend::{
            compiler::bytecode::{BinaryOffset, Bytecode}, types::{base::FSRObject, integer::FSRInteger, module::FSRModule}, vm::{runtime::FSRVM, thread::FSRThreadRuntime}
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
        let mut runtime = FSRThreadRuntime::new();
        let mut vm = FSRVM::new();
        runtime.set_vm(&mut vm);
        runtime.start(&v, &mut vm).unwrap();
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
    fn benchmark_add() {
        // let source_code = "
        // a = 1
        // b = 1
        // while a < 3000000 {
        //     a = a + b
        // }

        // ";
        // let v = FSRModule::from_code("main", source_code).unwrap();
        let mut runtime = FSRThreadRuntime::new();
        let mut vm = FSRVM::new();
        let start = Instant::now();
        //runtime.start(&v, &mut vm).unwrap();
        
        
        runtime.set_vm(&mut vm);

        for _ in 0..3000000 {
            let obj = FSRInteger::new_inst(3);
            let obj2 = FSRInteger::new_inst(4);

            FSRObject::invoke_offset_method(
                BinaryOffset::Add,
                &vec![FSRObject::obj_to_id(&obj), FSRObject::obj_to_id(&obj2)],
                &mut runtime,
            )
            .unwrap();
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
        let mut runtime = FSRThreadRuntime::new();
        let mut vm = FSRVM::new();
        let start = Instant::now();
        //runtime.start(&v, &mut vm).unwrap();
        
        
        runtime.set_vm(&mut vm);

        for _ in 0..3000000 {
            let obj = FSRInteger::new_inst(3);
            let obj2 = FSRInteger::new_inst(4);

            FSRObject::invoke_offset_method(
                BinaryOffset::Greater,
                &vec![FSRObject::obj_to_id(&obj), FSRObject::obj_to_id(&obj2)],
                &mut runtime,
            )
            .unwrap();
        }
        let end = Instant::now();
        println!("{:?}", end - start);
    }
}
