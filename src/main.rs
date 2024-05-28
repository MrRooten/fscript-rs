use std::time::Instant;

use fscript_rs::backend::{
    types::module::FSRModule, vm::{runtime::FSRVM, thread::FSRThreadRuntime}
};

fn main() {
    let source_code = "
    a = 1
    b = 1
    while a < 3000000 {
        a = a + b
    }

    ";
    let v = FSRModule::from_code("main", source_code).unwrap();
    let mut runtime = FSRThreadRuntime::new();
    let mut vm = FSRVM::new();
    let start = Instant::now();
    runtime.start(&v, &mut vm).unwrap();
    let end = Instant::now();
    println!("{:?}", end - start);
}
