## Introduce
Running on bytecode, simple and useless program language, zero dependence script language
Remove all main branch's backend, rewriting with bytecode
## Usage

not write interface yet, unit tests have some examples

```Rust
let expr = "
fn abc(ddc) {
    println(ddc)
}
ddc = 'asdf'
abc(ddc)

fn ccddefg(ddc) {
    println(ddc)
}

ccddefg('sdfsdfsdf')
println('okokokokok')
";
let meta = FSRMeta::new();
let token = FSRModuleFrontEnd::parse(expr.as_bytes(), meta).unwrap();
let v = Bytecode::load_ast(FSRToken::Module(token));
let mut runtime = FSRThreadRuntime::new();
let mut vm = FSRVM::new();
runtime.start(v, &mut vm);
```

```
ip: 0, [BytecodeArg { operator: Load, arg: Variable(100, "ddc") }, BytecodeArg { operator: Load, arg: Variable(100, "abc") }, BytecodeArg { operator: DefineFn, arg: DefineFnArgs(2, 1) }]
2
ip: 4, [BytecodeArg { operator: Load, arg: ConstString(0, "asdf") }, BytecodeArg { operator: Load, arg: Variable(101, "ddc") }, BytecodeArg { operator: Assign, arg: None }]
ip: 5, [BytecodeArg { operator: Load, arg: Variable(101, "ddc") }, BytecodeArg { operator: Load, arg: Variable(100, "abc") }, BytecodeArg { operator: Call, arg: CallArgsNumber(1) }]
name: abc
offset: 1
ip: 1, [BytecodeArg { operator: AssignArgs, arg: Variable(100, "ddc") }]
100, name:ddc
ip: 2, [BytecodeArg { operator: Load, arg: Variable(100, "ddc") }, BytecodeArg { operator: Load, arg: Variable(101, "println") }, BytecodeArg { operator: Call, arg: CallArgsNumber(1) }]
name: println
asdf
ip: 3, [BytecodeArg { operator: EndDefineFn, arg: None }]
ip: 6, [BytecodeArg { operator: Load, arg: Variable(100, "ddc") }, BytecodeArg { operator: Load, arg: Variable(102, "ccddefg") }, BytecodeArg { operator: DefineFn, arg: DefineFnArgs(2, 1) }]
2
ip: 10, [BytecodeArg { operator: Load, arg: ConstString(0, "sdfsdfsdf") }, BytecodeArg { operator: Load, arg: Variable(102, "ccddefg") }, BytecodeArg { operator: Call, arg: CallArgsNumber(1) }]
name: ccddefg
offset: 7
ip: 7, [BytecodeArg { operator: AssignArgs, arg: Variable(100, "ddc") }]
100, name:ddc
ip: 8, [BytecodeArg { operator: Load, arg: Variable(100, "ddc") }, BytecodeArg { operator: Load, arg: Variable(101, "println") }, BytecodeArg { operator: Call, arg: CallArgsNumber(1) }]
name: println
sdfsdfsdf
ip: 9, [BytecodeArg { operator: EndDefineFn, arg: None }]
ip: 11, [BytecodeArg { operator: Load, arg: ConstString(0, "okokokokok") }, BytecodeArg { operator: Load, arg: Variable(103, "println") }, BytecodeArg { operator: Call, arg: CallArgsNumber(1) }]
name: println
okokokokok
```