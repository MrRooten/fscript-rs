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

for a in [1, 2, 3, 4] {
    println(a)
}

a = Abc('dfdf')
println(a)
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
4
FSRObject {
    obj_id: 5232438800,
    value: ClassInst(
        FSRClassInst {
            name: "Dc",
            attrs: {
                "ttc": 5232438912,
            },
        },
    ),
    ref_count: 1,
    cls: "Dc",
}
Abc: abc = 123
```
