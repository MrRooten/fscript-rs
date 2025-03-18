use std::{sync::{Arc, Mutex}, time::Instant};

use crate::backend::{types::base::FSRObject, vm::thread::ThreadContext};

use super::{types::module::FSRModule, vm::{thread::FSRThreadRuntime, virtual_machine::FSRVM}};


pub fn timeit_code(code: &str, times: usize) {
    println!("running code:\n{}", code);
    println!("");
    let vm = Arc::new(Mutex::new(FSRVM::new()));
    let mut runtime = FSRThreadRuntime::new(0, vm.clone());
    let m = Box::new(FSRModule::from_code("main", code).unwrap());

    let module_id = FSRObject::obj_to_id(&m);
    let mut context = ThreadContext::new_context(vm.clone(), FSRObject::obj_to_id(&m));
    let start = Instant::now();
    for _ in 0..times {
        let _ = runtime.run_with_context(module_id, &mut context);
    }
    let end = Instant::now();
    println!("times: {}\nduration: {:?}\nspeed: {}/s", times, end - start, times as f64 / (end - start).as_secs_f64());
}