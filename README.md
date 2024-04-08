## Introduce
Running on bytecode, simple and useless program language, zero dependence script language
Remove all main branch's backend, rewriting with bytecode
## Usage

not write interface yet, unit tests have some examples

```Rust
let expr = "b(abc) + a + b * c + d";
let meta = FSRMeta::new();
let token = FSRExpr::parse(expr.as_bytes(), false, meta).unwrap().0;
let v = Bytecode::load_ast(token);
println!("{:#?}", v);
```

```
[
    BytecodeArg {
        operator: Load,
        arg: Variable(
            100,
            "abc",
        ),
    },
    BytecodeArg {
        operator: InsertArg,
        arg: InsertArg,
    },
    BytecodeArg {
        operator: Load,
        arg: Variable(
            100,
            "b",
        ),
    },
    BytecodeArg {
        operator: Call,
        arg: CallOperator,
    },
    BytecodeArg {
        operator: Load,
        arg: Variable(
            101,
            "a",
        ),
    },
    BytecodeArg {
        operator: Load,
        arg: Variable(
            100,
            "b",
        ),
    },
    BytecodeArg {
        operator: Load,
        arg: Variable(
            102,
            "c",
        ),
    },
    BytecodeArg {
        operator: BinaryMul,
        arg: BinaryOperator,
    },
    BytecodeArg {
        operator: Load,
        arg: Variable(
            103,
            "d",
        ),
    },
    BytecodeArg {
        operator: BinaryAdd,
        arg: BinaryOperator,
    },
    BytecodeArg {
        operator: BinaryAdd,
        arg: BinaryOperator,
    },
    BytecodeArg {
        operator: BinaryAdd,
        arg: BinaryOperator,
    },
]
```