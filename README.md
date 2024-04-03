# Usage

not write interface yet, unit tests have some examples

```Rust
fn abc(b) {
    c = b + 3
    if c > 2 {
        println("c bigger than 2")
        dump_obj(c)
    }
    println(c)
    println(b)
}
a = 1
abc(a)
```

```
c bigger than 2
FSRObject {
    id: 1019,
    obj_type: Object,
    cls: Some(
        FSRClass {
            name: "Integer",
            attrs: {
                "sub": 1002,
                "greater_equal": 1007,
                "less": 1008,
                "add": 1001,
                "to_string": 1005,
                "greater": 1006,
                "mul": 1003,
                "eq": 1004,
                "less_equal": 1009,
            },
        },
    ),
    ref_count: 3,
    value: Integer(
        FSRInteger {
            value: 4,
        },
    ),
    attrs: {},
}
4
1
```

## Introduce
Not running on bytecode, just interpret AST, simple and useless program language, zero dependence script language