## Introduce
Not running on bytecode, just interpret AST, simple and useless program language, zero dependence script language

## Usage

not write interface yet, unit tests have some examples

```Rust
let mut vm = FSRVirtualMachine::new().unwrap();
let mut thread = FSRThreadRuntime::new();
let code = "

fn abc() {
    a = 1
    while a < 3 {
        a = a + 1
        c = 1
        while c < 3 {
            c = c + 1
            print(\"c: \")
            println(c)
        }
    }

    return 'abc'
}

c = abc()
println(c)
";

vm.run_code(code.as_bytes(), &mut thread);
```

```
c: 2
c: 3
c: 2
c: 3
abc
```

```rust

fn test_fn() {
    let mut vm = FSRVirtualMachine::new().unwrap();
    let mut thread = FSRThreadRuntime::new();
    let code = "

    fn abc(bbc, ddc) {
        
        println(bbc)
        println(ddc)
        return 'abc'
    }

    abc(45, 56)
    ";
    
    vm.run_code(code.as_bytes(), &mut thread);
}

```

```
45
56
```

```rust
let mut vm = FSRVirtualMachine::new().unwrap();
let mut thread = FSRThreadRuntime::new();
let code = "
class Abc {
    abc = 1

    fn test(self) {
        println('abc')
    }

    fn bbc(self) {
        println(self)
    }

    fn __new__(self, test) {
        self.cccc = 123 
    }
}

b = Abc('abc')
b.test()
dump_obj(b)
";

vm.run_code(code.as_bytes(), &mut thread);
```

```
abc
FSRObject {
    id: 1024,
    obj_type: Object,
    cls: None,
    ref_count: 4,
    value: ClassInst(
        FSRClassInstance {
            attrs: {
                "abc": 1023,
                "cccc": 1025,
            },
            cls: FSRClassBackEnd {
                name: "Abc",
                attrs: {
                    "abc": Constant(
                        FSRConstant {
                            constant: Integer(
                                1,
                            ),
                            len: 0,
                            single_op: None,
                            meta: FSRMeta {
                                offset: 39,
                            },
                        },
                    ),
                },
                cls_attrs: {
                    "test": 1018,
                    "bbc": 1019,
                    "__new__": 1020,
                },
            },
        },
    ),
    attrs: {},
}
```

