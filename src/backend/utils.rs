


pub fn timeit_code(_code: &str, _times: usize) {
    // println!("running code:\n{}", code);
    // println!();
    // let vm = Arc::new(Mutex::new(FSRVM::new()));
    // let mut runtime = FSRThreadRuntime::new(vm.clone());
    // let mut m = Box::new(FSRCode::from_code("main", code).unwrap());

    // let m = m.remove("__main__").unwrap();
    // let code_id = FSRObject::obj_to_id(&m);
    // let mut context = ThreadContext::new_context(vm.clone(), FSRObject::obj_to_id(&m));
    // let start = Instant::now();
    // for _ in 0..times {
    //     let _ = runtime.run_with_context(code_id, &mut context);
    // }
    // let end = Instant::now();
    // println!("times: {}\nduration: {:?}\nspeed: {}/s", times, end - start, times as f64 / (end - start).as_secs_f64());
}