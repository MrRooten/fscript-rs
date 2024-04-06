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

    fn __new__(self) {
        self.cccc = 123 
    }

    fn t(self, cdf) {
        return cdf
    }
}

b = Abc()
c = b.t('sdf')
println(c)
d = b.t('asfd')
println(d)
";

vm.run_code(code.as_bytes(), &mut thread);
```

```
sdf
asfs
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

    fn __new__(self, cdf) {
        self.cccc = cdf 
    }

    fn t(self, cdf) {
        return cdf
    }

    fn __str__(self) {
        return 'this is __str__'
    }
}

b = 1 + 1
println(b)

c = Abc('')
println(c)

f = c.t('this is t func')
println(f)
";

vm.run_code(code.as_bytes(), &mut thread);
```

```
2
this is __str__
this is t func
test backend::tests::backend_tests::test_class ... ok
```