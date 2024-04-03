use fscript_rs::backend::vm::{module::FSRRuntimeModule, vm::FSRVirtualMachine};

fn main() {
    let mut vm = FSRVirtualMachine::new().unwrap();
    let mut context = FSRRuntimeModule::new();
    let code = "
        fn abc(b) {
            c = b + 3
            if c > 2 {
                println(\"c bigger than 2\")
                dump_obj(c)
            }
            println(c)
            println(b)
        }
        a = 1
        abc(a)
        ";

    vm.run_code(code.as_bytes(), &mut context);
}
