#[cfg(test)]
mod tests {

    use crate::{
        backend::{
            compiler::bytecode::Bytecode, types::module::FSRModule, vm::{runtime::FSRVM, thread::FSRThreadRuntime}
        },
        frontend::ast::token::{
            base::{FSRPosition, FSRToken},
            module::FSRModuleFrontEnd,
        },
    };

    #[test]
    fn test_1() {
        let expr = "
        if a < 3 {
            println('abc')
        } else if a < 3 {
            println('else')
        } else if a > 3 {
            println('else2')
        }
        ";
        let meta = FSRPosition::new();
        let token = FSRModuleFrontEnd::parse(expr.as_bytes(), meta).unwrap();
        let v = Bytecode::load_ast("main", FSRToken::Module(token));
        println!("{:#?}", v);
    }

    #[test]
    fn test_for_bc() {
        let expr = "
        class Dc {
            fn __new__(self) {
                self.ttc = 123
                dump(self)
                return self
            }
        }

        class Abc {
            fn __new__(self, abc) {
                self.abc = Dc()
                return self
            }

            fn __str__(self) {
                return 'Abc: abc = 123'
            }
        }
        a = 3

        b = [1, 2, 3, 4, 5]

        for a in b {
            if a > 2 {
                break
            } else {
                println('ok')
            }
            println(a)
        }
        ";
        let meta = FSRPosition::new();
        let token = FSRModuleFrontEnd::parse(expr.as_bytes(), meta).unwrap();
        let v = Bytecode::load_ast("main", FSRToken::Module(token));
        println!("{:#?}", v);
    }

    #[test]
    fn test_2() {
        let source_code = "
        class Dc {
            fn __new__(self) {
                self.ttc = 123
                dump(self)
                return self
            }
        }

        class Abc {
            fn __new__(self, abc) {
                self.abc = Dc()
                return self
            }

            fn __str__(self) {
                return 'Abc: abc = 123'
            }
        }
        a = 1
        while a < 300000 {
            a = a + 1
        }
        ";
        let v = FSRModule::from_code("main", source_code).unwrap();
        let mut runtime = FSRThreadRuntime::new();
        let mut vm = FSRVM::new();
        runtime.set_vm(&mut vm);
        runtime.start(&v, &mut vm).unwrap();
    }

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
        a = 0
        while a < 1000000 {
            a = a + 1
        }
        ";
        let v = FSRModule::from_code("main", source_code).unwrap();
        let mut runtime = FSRThreadRuntime::new();
        let mut vm = FSRVM::new();
        runtime.set_vm(&mut vm);
        runtime.start(&v, &mut vm).unwrap();
    }
}
