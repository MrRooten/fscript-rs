use std::{
    sync::{Arc, Mutex},
    thread::{self},
    time::Instant,
};

use std::io::Read;

use fscript_rs::backend::{
    types::{code::FSRCode, module::FSRModule},
    vm::{thread::FSRThreadRuntime, virtual_machine::FSRVM},
};
mod test {
    use fscript_rs::backend::utils::timeit_code;

    pub fn bench() {
        timeit_code(r#"1 + 3 + 4 + 5 + 6"#, 30000000);
    }
}

fn main() {
    let mut vs = vec![];
    for i in std::env::args() {
        vs.push(i);
    }

    if vs.iter().any(|x| x.eq("-b")) {
        test::bench();
        return;
    }
    if vs.len() < 2 {
        println!("Usage: {} ${{file}}", vs[0]);
        return;
    }
    let vm = FSRVM::new();
    let file = &vs[1];
    let mut f = std::fs::File::open(file).unwrap();
    let mut source_code = String::new();
    f.read_to_string(&mut source_code).unwrap();

    let vm = Arc::new(Mutex::new(vm));
    let mut rt = FSRThreadRuntime::new(vm);
    // let runtime = Arc::new(rt);
    

    let start = Instant::now();
    let th = thread::spawn(move || {
        let v = FSRCode::from_code("main", &source_code).unwrap();
        let module = Box::new(FSRModule::new("main", v));
        let module_id = FSRVM::leak_object(module);
        rt.start(module_id).unwrap();
    });
    let _ = th.join();

    let end = Instant::now();
    println!("{:?}", end - start);
}
