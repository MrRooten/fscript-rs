## Introduce
Running on bytecode, simple and useless program language, zero dependence script language.
## Usage

not write interface yet, unit tests have some examples

```Rust
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

b = [1, 2, 3, 4, 5]

for a in b {
    if a > 3 {
        println('bigger than 3')
    }
    println(a)
}
";
let v = Bytecode::compile("main", source_code);
let mut runtime = FSRThreadRuntime::new();
let mut vm = FSRVM::new();
runtime.start(&v, &mut vm).unwrap();
```

```
1
2
3
bigger than 3
4
bigger than 3
5
```
