use std::time::Instant;

use std::io::Read;

use fscript_rs::backend::{
    types::{base::FSRObject, code::FSRCode, module::FSRModule},
    vm::{thread::FSRThreadRuntime, virtual_machine::FSRVM},
};
mod test {
    use std::{
        thread::sleep,
        time::{Duration, Instant},
    };

    use fscript_rs::backend::{
        types::{base::FSRObject, code::FSRCode, module::FSRModule},
        vm::{thread::FSRThreadRuntime, virtual_machine::FSRVM},
    };

    pub fn bench() {
        let _ = FSRVM::single();
        let p_st = Instant::now();
        let module1 = r#"
        import thread
        fn abc1() {
            a = 1 + 1
        }
        fn abc() {
            id = __get_cur_thread_id()
            i = 0
            sleep(3000)
            println("done id: ", id)
            println("done res: ", i)
        }

        th = thread::Thread(abc)
        th.join()
        println("hello world")
        "#;
        let obj: Box<FSRObject<'_>> = Box::new(FSRModule::new_module("main"));
        let obj_id = FSRVM::leak_object(obj);
        let v = FSRCode::from_code("main", module1, obj_id).unwrap();
        let obj = FSRObject::id_to_mut_obj(obj_id).unwrap();
        obj.as_mut_module().init_fn_map(v);
        let vm = FSRVM::single();
        let vm2 = vm.clone();
        let runtime = FSRThreadRuntime::new_runtime();
        let tid = vm2.add_thread(runtime);
        let th = std::thread::spawn(move || {
            let binding = vm2.clone();
            let th = binding.get_thread(tid).unwrap();
            let _ = th.start(obj_id);
        });
        let st = Instant::now();
        sleep(std::time::Duration::from_millis(100));
        println!("stop all threads: {:?}", (Instant::now()) - p_st);
        vm.stop_all_threads();
        // vm.wait_all_threads();

        std::thread::sleep(std::time::Duration::from_secs(1));
        println!("sleep 1 second");
        println!("continue all threads");
        vm.continue_all_threads();
        println!("run 2 second");
        sleep(Duration::from_secs(2));
        println!("stop all threads: {:?} again", (Instant::now()) - p_st);

        let et = Instant::now();
        println!("elapsed time: {:?}", et - st);
        th.join().unwrap();
        let et2 = Instant::now();
        println!("all program elapsed time: {:?}", et2 - p_st);
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
    let vm = FSRVM::single();
    let file = &vs[1];
    let mut f = std::fs::File::open(file).unwrap();
    let mut source_code = String::new();
    f.read_to_string(&mut source_code).unwrap();

    let rt = FSRThreadRuntime::new_runtime();
    let tid = vm.add_thread(rt);
    // let runtime = Arc::new(rt);

    let start = Instant::now();
    let thread = vm.get_thread(tid).unwrap();

    let obj: Box<FSRObject<'_>> = Box::new(FSRModule::new_module("main"));
    let obj_id = FSRVM::leak_object(obj);
    let v = FSRCode::from_code("main", &source_code, obj_id).unwrap();
    let obj = FSRObject::id_to_mut_obj(obj_id).unwrap();
    obj.as_mut_module().init_fn_map(v);
    thread.start(obj_id).unwrap();

    let end = Instant::now();
    println!("{:?}", end - start);
}
