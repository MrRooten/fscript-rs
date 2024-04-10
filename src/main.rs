use fscript_rs::{
    backend::{
        compiler::bytecode::Bytecode,
        vm::{runtime::FSRVM, thread::FSRThreadRuntime},
    },
    frontend::ast::token::{
        base::{FSRMeta, FSRToken},
        module::FSRModuleFrontEnd,
    },
};

fn main() {
    let expr = "
        b = 3 * 4
        println(b)
        while b < 30 {
            println('abc')
            b = b + 1
        }
        ";
        let meta = FSRMeta::new();
        let token = FSRModuleFrontEnd::parse(expr.as_bytes(), meta).unwrap();
        let v = Bytecode::load_ast(FSRToken::Module(token));
        let mut runtime = FSRThreadRuntime::new();
        let mut vm = FSRVM::new();
        runtime.start(v, &mut vm);
}
