use fscript_rs::backend::{
    types::module::FSRModule, vm::{runtime::FSRVM, thread::FSRThreadRuntime}
};

fn main() {
    let source_code = "
    class Dc {
        fn __new__(self) {
            self.ttc = 123
            dump(self)
            return self
        }
    }

    class Abc {
        fn __new__(self, abc) {
            self.abc = Dc()
            return self
        }

        fn __str__(self) {
            return 'Abc: abc = 123'
        }
    }

    fn main() {
        b = [1, 2, 3, 4, 5]

        for a in b {
            if a > 4 {
                println('bigger than 4')
            } else if a > 3 {
                println('bigger than 3')
            }
            println(a)
        }
    }

    main()

    
    ";
    let v = FSRModule::from_code("main", source_code).unwrap();
    let mut runtime = FSRThreadRuntime::new();
    let mut vm = FSRVM::new();
    runtime.start(&v, &mut vm).unwrap();
}
