use fscript_rs::
    backend::{
        compiler::bytecode::Bytecode,
        vm::{runtime::FSRVM, thread::FSRThreadRuntime},
    }
;

fn main() {
    let source_code = "
class Abc {
    fn __new__(self, abc) {
        self.abc = 123
        return self
    }

    fn __str__(self) {
        return 'Abc: abc = 123'
    }
}
c = Abc('456')
println(c)
a = [1, 2, 3, c]
println(a)";

        println!("Running code:");
        println!("{}", source_code);
        println!("\n\n\n---------------------");
        let v = Bytecode::compile("main", source_code);
        let mut runtime = FSRThreadRuntime::new();
        let mut vm = FSRVM::new();
        runtime.start(&v, &mut vm).unwrap();
}
