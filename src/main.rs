use fscript_rs::backend::{
    compiler::bytecode::Bytecode,
    vm::{runtime::FSRVM, thread::FSRThreadRuntime},
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
        a = 3

        for a in [1, 2, 3, 4] {
            println(a)
        }

        a = Abc('dfdf')
        println(a)
    ";
    let v = Bytecode::compile("main", source_code);
    let mut runtime = FSRThreadRuntime::new();
    let mut vm = FSRVM::new();
    runtime.set_vm(&mut vm);
    runtime.start(&v, &mut vm).unwrap();
}
