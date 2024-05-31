use std::{io::Read, time::Instant};

use fscript_rs::backend::{
    types::module::FSRModule, vm::{runtime::FSRVM, thread::FSRThreadRuntime}
};

fn main() {
    let mut vs = vec![];
    for i in std::env::args() {
        vs.push(i);
    }
    if vs.len() < 2 {
        println!("Usage: {} ${{file}}", vs[0]);
        return ;
    }
    let file = &vs[1];
    let mut f = std::fs::File::open(file).unwrap();
    let mut source_code = String::new();
    f.read_to_string(&mut source_code).unwrap();
    let v = FSRModule::from_code("main", &source_code).unwrap();
    let mut runtime = FSRThreadRuntime::new();
    let mut vm = FSRVM::new();
    let start = Instant::now();
    runtime.start(&v, &mut vm).unwrap();
    let end = Instant::now();
    println!("{:?}", end - start);
}
