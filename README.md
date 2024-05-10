## Introduce
Running on bytecode, simple and useless program language, zero dependence script language.
## Usage

not write interface yet, unit tests have some examples

```Rust
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
runtime.set_vm(&mut vm);
runtime.start(&v, &mut vm).unwrap();
```

```
Running code:

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
println(a)
        



---------------------
Abc: abc = 123
[1, 2, 3, Abc: abc = 123]
```
