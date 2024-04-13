#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use crate::{backend::{compiler::bytecode::Bytecode, vm::{runtime::FSRVM, thread::FSRThreadRuntime}}, frontend::ast::token::{base::{FSRMeta, FSRToken}, expr::FSRExpr, module::FSRModuleFrontEnd}};

    #[test]
    fn test_1() {
        let expr = "
        class Abc {
            fn __new__(self, abc) {
                self.abc = 123
                return self
            }

        }

        a = Abc(123123)
        ";
        let meta = FSRMeta::new();
        let token = FSRModuleFrontEnd::parse(expr.as_bytes(), meta).unwrap();
        let v = Bytecode::load_ast("main", FSRToken::Module(token));
        println!("{:#?}", v);
    }

    #[test]
    fn test_2() {
        let source_code = "
        class Abc {
            fn __new__(self, abc) {
                self.abc = 123
                return self
            }

        }

        a = Abc(123123)
        dump(a)
        ";
        let v = Bytecode::compile("main", source_code);
        let mut runtime = FSRThreadRuntime::new();
        let mut vm = FSRVM::new();
        runtime.start(&v, &mut vm);
    }
}