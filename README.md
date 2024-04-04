# Usage

not write interface yet, unit tests have some examples

```Rust
fn abc() {
    a = 1
    while a < 3 {
        a = a + 1
        c = 1
        while c < 3 {
            c = c + 1
            print("c: ")
            println(c)
        }
    }

    return 'abc'
}

c = abc()
println(c)
```

```
c: 2
c: 3
c: 2
c: 3
abc
```

## Introduce
Not running on bytecode, just interpret AST, simple and useless program language, zero dependence script language