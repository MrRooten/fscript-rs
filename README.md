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

    fn test(self) {
        dump(self)
        return 123
    }
}
c = Abc('sdf')
a = 1
a = a + 1
dump(a)
";
let v = Bytecode::compile("main", source_code);
let mut runtime = FSRThreadRuntime::new();
let mut vm = FSRVM::new();
runtime.start(&v, &mut vm);
```

```
IP: (0, 1) => BytecodeArg { operator: Load, arg: Variable(100, "Abc") }
IP: (0, 2) => BytecodeArg { operator: ClassDef, arg: DefineClassLine(10) }
IP: (1, 1) => BytecodeArg { operator: Load, arg: Variable(100, "self") }
IP: (1, 2) => BytecodeArg { operator: Load, arg: Variable(101, "abc") }
IP: (1, 3) => BytecodeArg { operator: Load, arg: Variable(100, "__new__") }
IP: (1, 4) => BytecodeArg { operator: DefineFn, arg: DefineFnArgs(3, 2) }
name: __new__
IP: (6, 1) => BytecodeArg { operator: Load, arg: Variable(100, "self") }
IP: (6, 2) => BytecodeArg { operator: Load, arg: Variable(101, "test") }
IP: (6, 3) => BytecodeArg { operator: DefineFn, arg: DefineFnArgs(3, 1) }
name: test
IP: (11, 1) => BytecodeArg { operator: EndDefineClass, arg: None }
IP: (12, 1) => BytecodeArg { operator: Load, arg: ConstString(0, "sdf") }
IP: (12, 2) => BytecodeArg { operator: Load, arg: Variable(100, "Abc") }
IP: (12, 3) => BytecodeArg { operator: Call, arg: CallArgsNumber(1) }
IP: (2, 1) => BytecodeArg { operator: AssignArgs, arg: Variable(100, "self") }
IP: (2, 2) => BytecodeArg { operator: AssignArgs, arg: Variable(101, "abc") }
IP: (3, 1) => BytecodeArg { operator: Load, arg: ConstInteger(0, 123) }
IP: (3, 2) => BytecodeArg { operator: Load, arg: Variable(100, "self") }
IP: (3, 3) => BytecodeArg { operator: Load, arg: Attr(101, "abc") }
IP: (3, 4) => BytecodeArg { operator: BinaryDot, arg: None }
dot_father_obj: FSRObject { obj_id: 5275408608, value: ClassInst(FSRClassInst { name: "Abc", attrs: {} }), ref_count: 0, cls: "Abc" }
name: abc
IP: (3, 5) => BytecodeArg { operator: Assign, arg: None }
FSRObject { obj_id: 5275408608, value: ClassInst(FSRClassInst { name: "Abc", attrs: {} }), ref_count: 0, cls: "Abc" }
IP: (4, 1) => BytecodeArg { operator: Load, arg: Variable(100, "self") }
IP: (4, 2) => BytecodeArg { operator: ReturnValue, arg: None }
IP: (12, 4) => BytecodeArg { operator: Load, arg: Variable(101, "c") }
IP: (12, 5) => BytecodeArg { operator: Assign, arg: None }
IP: (13, 1) => BytecodeArg { operator: Load, arg: ConstInteger(0, 1) }
IP: (13, 2) => BytecodeArg { operator: Load, arg: Variable(102, "a") }
IP: (13, 3) => BytecodeArg { operator: Assign, arg: None }
IP: (14, 1) => BytecodeArg { operator: Load, arg: Variable(102, "a") }
IP: (14, 2) => BytecodeArg { operator: Load, arg: ConstInteger(0, 1) }
IP: (14, 3) => BytecodeArg { operator: BinaryAdd, arg: None }
IP: (14, 4) => BytecodeArg { operator: Load, arg: Variable(102, "a") }
IP: (14, 5) => BytecodeArg { operator: Assign, arg: None }
IP: (15, 1) => BytecodeArg { operator: Load, arg: Variable(102, "a") }
IP: (15, 2) => BytecodeArg { operator: Load, arg: Variable(103, "dump") }
IP: (15, 3) => BytecodeArg { operator: Call, arg: CallArgsNumber(1) }
FSRObject {
    obj_id: 5293228576,
    value: Integer(
        2,
    ),
    ref_count: 0,
    cls: "Integer",
}
```
