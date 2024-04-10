#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use crate::{backend::{compiler::bytecode::Bytecode, vm::{runtime::FSRVM, thread::FSRThreadRuntime}}, frontend::ast::token::{base::{FSRMeta, FSRToken}, expr::FSRExpr, module::FSRModuleFrontEnd}};

    #[test]
    fn test_1() {
        let expr = "
        fn abc(ddc) {
            println(ddc)
        }
        ddc = 1
        abc(ddc)
        fn abc(ddc) {
            println(ddc)
        }
        ";
        let meta = FSRMeta::new();
        let token = FSRModuleFrontEnd::parse(expr.as_bytes(), meta).unwrap();
        let v = Bytecode::load_ast(FSRToken::Module(token));
        println!("{:#?}", v);
    }

    #[test]
    fn test_2() {
        let expr = "
        fn abc(ddc) {
            println(ddc)
        }
        ddc = 'asdf'
        abc(ddc)
        ";
        let meta = FSRMeta::new();
        let token = FSRModuleFrontEnd::parse(expr.as_bytes(), meta).unwrap();
        let v = Bytecode::load_ast(FSRToken::Module(token));
        let mut runtime = FSRThreadRuntime::new();
        let mut vm = FSRVM::new();
        runtime.start(v, &mut vm);
    }
}