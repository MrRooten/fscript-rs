use std::{
    sync::{Arc, Mutex},
    thread::{self, Thread},
    time::Instant,
};

use fscript_rs::backend::{
    types::module::FSRModule,
    vm::{runtime::FSRVM, thread::FSRThreadRuntime},
};
mod test {
    use std::{
        sync::{Arc, Mutex},
        time::Instant,
    };

    use fscript_rs::backend::{
        compiler::bytecode::BinaryOffset,
        types::{base::FSRObject, integer::FSRInteger},
        vm::{runtime::FSRVM, thread::FSRThreadRuntime},
    };

    pub fn bench() {
        
        let mut vm = Arc::new(Mutex::new(FSRVM::new()));
        let mut runtime = FSRThreadRuntime::new(0, vm.clone());
        let start = Instant::now();
        //runtime.start(&v, &mut vm).unwrap();

        let obj = FSRInteger::new_inst(3);
        let obj2 = FSRInteger::new_inst(4);
        for _ in 0..3000000 {
            let v = FSRObject::invoke_offset_method(
                BinaryOffset::Add,
                &[FSRObject::obj_to_id(&obj), FSRObject::obj_to_id(&obj2)],
                &mut runtime,
                None,
            )
            .unwrap();

            match v {
                fscript_rs::backend::types::base::FSRRetValue::Value(fsrobject) => {
                    vm.lock().unwrap().allocator.free_object(fsrobject)
                }
                fscript_rs::backend::types::base::FSRRetValue::GlobalId(_) => todo!(),
                fscript_rs::backend::types::base::FSRRetValue::GlobalIdTemp(_) => todo!(),
            }
        }
        let end = Instant::now();
        println!("{:?}", end - start);
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

    let base_module = base_module.clone();
    let start = Instant::now();
    let th = thread::spawn(move || {
        rt.start(base_module).unwrap();
    });
    let _ = th.join();
    let end = Instant::now();
    println!("{:?}", end - start);

    
    // runtime.start(base_module, &mut vm).unwrap();
    // let end = Instant::now();

    use std::io::Read;

    // let input: Option<i32> = std::io::stdin()
    //     .bytes()
    //     .next()
    //     .and_then(|result| result.ok())
    //     .map(|byte| byte as i32);

    // println!("{:?}", input);
}
