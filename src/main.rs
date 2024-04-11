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
    fn abc(ddc) {
        println(ddc)
    }
    ddc = 'asdf'
    abc(ddc)

    fn ccddefg(ddc) {
        println(ddc)
    }

    ccddefg('sdfsdfsdf')
    println('okokokokok')
    ";
    let meta = FSRMeta::new();
    let token = FSRModuleFrontEnd::parse(expr.as_bytes(), meta).unwrap();
    let v = Bytecode::load_ast("main", FSRToken::Module(token));
    let mut runtime = FSRThreadRuntime::new();
    let mut vm = FSRVM::new();
    runtime.start(&v, &mut vm);
}
