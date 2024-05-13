#[cfg(test)]
mod tests {

    use crate::{
        backend::{
            compiler::bytecode::Bytecode,
            vm::{runtime::FSRVM, thread::FSRThreadRuntime},
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

        fn test() {
            a = 123
            if a < 3 {
                println('abc')
            } else if a < 3 {
                println('else')
            } else {
                println('else2')
            }
        }

        test()
        ";
        let v = Bytecode::compile("main", source_code);
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
}
