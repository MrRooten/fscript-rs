use std::time::Instant;

use std::io::Read;

use fscript_rs::{backend::{
    compiler::bytecode::Bytecode, types::{base::FSRObject, code::FSRCode, module::FSRModule}, vm::{thread::FSRThreadRuntime, virtual_machine::FSRVM}
}, frontend::ast::token::{base::{FSRPosition, FSRToken}, module::FSRModuleFrontEnd}};

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

    
    let vm = FSRVM::single();
    let file = &vs[1];
    let mut f = std::fs::File::open(file).unwrap();
    let mut source_code = String::new();
    f.read_to_string(&mut source_code).unwrap();

    if just_bc {
        let meta = FSRPosition::new();
        let token = FSRModuleFrontEnd::parse(source_code.as_bytes(), meta).unwrap();
        let v = Bytecode::load_ast("main", FSRToken::Module(token.0), token.1);
        println!("{:#?}", v);

        return ;
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
    thread.start(obj_id).unwrap();

    let end = Instant::now();
    println!("{:?}", end - start);
}
