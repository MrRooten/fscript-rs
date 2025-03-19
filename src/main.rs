use std::{
    sync::{Arc, Mutex},
    thread::{self},
    time::Instant,
};

use std::io::Read;

use fscript_rs::backend::{
    types::module::FSRModule,
    vm::{thread::FSRThreadRuntime, virtual_machine::FSRVM},
};
mod test {
    use fscript_rs::backend::utils::timeit_code;

    pub fn bench() {
        timeit_code(
            r#"1 + 3 + 4 + 5 + 6"#,
            30000000,
        );
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
    let file = &vs[1];
    let mut f = std::fs::File::open(file).unwrap();
    let mut source_code = String::new();
    f.read_to_string(&mut source_code).unwrap();
    let v = FSRModule::from_code("main", &source_code).unwrap();
    let base_module = FSRVM::leak_object(Box::new(v));

    let vm = Arc::new(Mutex::new(FSRVM::new()));
    let mut rt = FSRThreadRuntime::new(base_module, vm);
    // let runtime = Arc::new(rt);

    let start = Instant::now();
    let th = thread::spawn(move || {
        rt.start(base_module).unwrap();
    });
    let _ = th.join();
    let end = Instant::now();
    println!("{:?}", end - start);
}
