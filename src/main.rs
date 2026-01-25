use std::time::Instant;

use std::io::Read;

use fscript_rs::backend::{
    compiler::bytecode::Bytecode,
    types::{base::FSRObject, code::FSRCode, module::FSRModule},
    vm::{thread::FSRThreadRuntime, virtual_machine::FSRVM},
};

use frontend::ast::token::{
    base::{FSRPosition, FSRToken},
    module::FSRModuleFrontEnd,
};

fn bench_compile() {
    let mut line = vec![];
    for _ in 0..100000 {
        line.push(format!("asdfsdf: u64 = 1"));
        line.push(
            r#"
        if asdfsdf > 10 {
            asdfsdf = asdfsdf + 1
        } else {
            asdfsdf = asdfsdf - 1
        }
        "#
            .to_string(),
        );
    }

    let source_code = line.join("\n");
    let start = Instant::now();
    let meta = FSRPosition::new();
    let chars = source_code.chars().collect::<Vec<char>>();
    let token = FSRModuleFrontEnd::parse(&chars, meta).unwrap();
    let end = Instant::now();
    println!("AST Parse Time: {:?}", end - start);
    let start = Instant::now();
    Bytecode::load_ast("main", FSRToken::Module(token.0), token.1);
    let end = Instant::now();
    println!("Bytecode Compile Time: {:?}", end - start);
}

fn main() {
    let mut vs = vec![];
    for i in std::env::args() {
        vs.push(i);
    }

    if vs.len() < 2 {
        println!("Usage: {} ${{file}}", vs[0]);
        return;
    }

    let mut just_bc = false;
    if vs.iter().any(|x| x.eq("-bc")) {
        just_bc = true;
    }

    let mut debugger = false;

    if vs.iter().any(|x| x.eq("-dbg")) {
        debugger = true;
    }

    let mut ast = false;
    if vs.iter().any(|x| x.eq("-ast")) {
        ast = true;
    }

    if vs.iter().any(|x| x.eq("-bench-compile")) {
        bench_compile();
        return;
    }

    let vm = FSRVM::single();
    let file = &vs[1];
    let mut f = std::fs::File::open(file).unwrap();
    let mut source_code = String::new();
    f.read_to_string(&mut source_code).unwrap();

    if ast {
        let meta = FSRPosition::new();
        let chars = source_code.chars().collect::<Vec<char>>();
        let token = FSRModuleFrontEnd::parse(&chars, meta).unwrap();
        println!("{:#?}", token);
        return;
    }

    if just_bc {
        let meta = FSRPosition::new();
        let chars = source_code.chars().collect::<Vec<char>>();
        let token = FSRModuleFrontEnd::parse(&chars, meta).unwrap();
        let v = Bytecode::load_ast("main", FSRToken::Module(token.0), token.1);
        println!("{:#?}", v);

        return;
    }

    let rt = FSRThreadRuntime::new_runtime();
    let tid = vm.add_thread(rt);
    // let runtime = Arc::new(rt);

    let start = Instant::now();
    let thread = vm.get_thread(tid).unwrap();

    let obj: Box<FSRObject<'_>> = Box::new(FSRModule::new_object("main"));
    let obj_id = FSRVM::leak_object(obj);
    let v = FSRCode::from_code("main", &source_code, obj_id).unwrap();
    let obj = FSRObject::id_to_mut_obj(obj_id).unwrap();
    obj.as_mut_module().init_fn_map(v);
    thread.start(obj_id, debugger).unwrap();

    let end = Instant::now();
    println!("{:?}", end - start);
}
