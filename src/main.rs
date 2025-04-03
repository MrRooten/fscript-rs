use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use std::io::Read;

use fscript_rs::backend::{
    types::{code::FSRCode, module::FSRModule},
    vm::{thread::FSRThreadRuntime, virtual_machine::FSRVM},
};
mod test {
    use std::sync::{Arc, Mutex};

    use fscript_rs::backend::{
        types::{code::FSRCode, module::FSRModule},
        vm::{
            thread::FSRThreadRuntime,
            virtual_machine::FSRVM,
        },
    };

    pub fn bench() {
        let module1 = r#"
        i = 0
        while i < 10000000 {
            i = i + 1
        }

        cur_id = __get_cur_thread_id()
        print("cur id: ")
        println(cur_id)

        println(i)
        "#;
        let v = FSRCode::from_code("main", module1).unwrap();
        let obj = Box::new(FSRModule::new_module("main", v));
        let obj_id = FSRVM::leak_object(obj);
        let vm = Arc::new(FSRVM::new());
        let runtime = Mutex::new(FSRThreadRuntime::new(vm.clone()));
        let tid = vm.add_thread(runtime);
        let runtime2 = Mutex::new(FSRThreadRuntime::new(vm.clone()));
        let tid2 = vm.add_thread(runtime2);
        let vm2 = vm.clone();
        //runtime.start(&v, &mut vm).unwrap();
        let th = std::thread::spawn(move || {
            println!("thread 1: {}", tid);
            vm2.clone()
                .get_thread(tid, |f| {
                    f.start(obj_id).unwrap();
                })
                .unwrap();
        });
        let vm3 = vm.clone();
        let th2 = std::thread::spawn(move || {
            println!("thread 2: {}", tid2);
            vm3.clone()
                .get_thread(tid2, |f| {
                    f.start(obj_id).unwrap();
                })
                .unwrap();
        });
        vm.stop_all_threads();
        // vm.wait_all_threads();
        std::thread::sleep(std::time::Duration::from_secs(2));
        println!("sleep 2 seconds");
        vm.continue_all_threads();
        th.join().unwrap();
        th2.join().unwrap();
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

    let vm = Arc::new(vm);
    let rt = Mutex::new(FSRThreadRuntime::new(vm.clone()));
    let tid = vm.add_thread(rt);
    // let runtime = Arc::new(rt);

    let start = Instant::now();
    let vm2 = vm.clone();
    //runtime.start(&v, &mut vm).unwrap();
    let th = std::thread::spawn(move || {
        vm2.clone()
            .get_thread(tid, |f| {
                let v = FSRCode::from_code("main", &source_code).unwrap();
                let module = Box::new(FSRModule::new_module("main", v));
                let module_id = FSRVM::leak_object(module);
                f.start(module_id).unwrap();
            })
            .unwrap();
    });
    let _ = th.join();

    let end = Instant::now();
    println!("{:?}", end - start);
}
