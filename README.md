## Introduce
Running on bytecode, simple and useless program language, zero dependence script language.
## Usage

not write interface yet, unit tests have some examples

```Rust
let source_code = "
class Abc {
    fn __new__(self, abc) {
        self.abc = 123
        println(self.abc)
        self.abc = 456
        println(self.abc)
        return self
    }

    fn test(self) {
        dump(self)
        return 123
    }
}
c = Abc('456')
b = Abc('123')
b.dd = c
dump(b.dd.abc)

a = 1
while a < 3 {
    println(a)
    a = a + 1
}
println(a)
";
let v = Bytecode::compile("main", source_code);
let mut runtime = FSRThreadRuntime::new();
let mut vm = FSRVM::new();
runtime.start(&v, &mut vm);
```

```
IP: (0, 1) => BytecodeArg { operator: Load, arg: Variable(100, "Abc") }
IP: (0, 2) => BytecodeArg { operator: ClassDef, arg: DefineClassLine(13) }
IP: (1, 1) => BytecodeArg { operator: Load, arg: Variable(100, "self") }
IP: (1, 2) => BytecodeArg { operator: Load, arg: Variable(101, "abc") }
IP: (1, 3) => BytecodeArg { operator: Load, arg: Variable(100, "__new__") }
IP: (1, 4) => BytecodeArg { operator: DefineFn, arg: DefineFnArgs(6, 2) }
IP: (9, 1) => BytecodeArg { operator: Load, arg: Variable(100, "self") }
IP: (9, 2) => BytecodeArg { operator: Load, arg: Variable(101, "test") }
IP: (9, 3) => BytecodeArg { operator: DefineFn, arg: DefineFnArgs(3, 1) }
IP: (14, 1) => BytecodeArg { operator: EndDefineClass, arg: None }
IP: (15, 1) => BytecodeArg { operator: Load, arg: ConstString(0, "456") }
IP: (15, 2) => BytecodeArg { operator: Load, arg: Variable(100, "Abc") }
IP: (15, 3) => BytecodeArg { operator: Call, arg: CallArgsNumber(1) }
IP: (2, 1) => BytecodeArg { operator: AssignArgs, arg: Variable(100, "self") }
IP: (2, 2) => BytecodeArg { operator: AssignArgs, arg: Variable(101, "abc") }
IP: (3, 1) => BytecodeArg { operator: Load, arg: ConstInteger(0, 123) }
IP: (3, 2) => BytecodeArg { operator: Load, arg: Variable(100, "self") }
IP: (3, 3) => BytecodeArg { operator: Load, arg: Attr(101, "abc") }
IP: (3, 4) => BytecodeArg { operator: BinaryDot, arg: None }
IP: (3, 5) => BytecodeArg { operator: Assign, arg: None }
IP: (4, 1) => BytecodeArg { operator: Load, arg: Variable(100, "self") }
IP: (4, 2) => BytecodeArg { operator: Load, arg: Attr(101, "abc") }
IP: (4, 3) => BytecodeArg { operator: BinaryDot, arg: None }
IP: (4, 4) => BytecodeArg { operator: Load, arg: Variable(102, "println") }
IP: (4, 5) => BytecodeArg { operator: Call, arg: CallArgsNumber(1) }
load object as args: FSRObject { obj_id: 5636112592, value: Integer(123), ref_count: 0, cls: "Integer" }
123
IP: (5, 1) => BytecodeArg { operator: Load, arg: ConstInteger(0, 456) }
IP: (5, 2) => BytecodeArg { operator: Load, arg: Variable(100, "self") }
IP: (5, 3) => BytecodeArg { operator: Load, arg: Attr(101, "abc") }
IP: (5, 4) => BytecodeArg { operator: BinaryDot, arg: None }
IP: (5, 5) => BytecodeArg { operator: Assign, arg: None }
IP: (6, 1) => BytecodeArg { operator: Load, arg: Variable(100, "self") }
IP: (6, 2) => BytecodeArg { operator: Load, arg: Attr(101, "abc") }
IP: (6, 3) => BytecodeArg { operator: BinaryDot, arg: None }
IP: (6, 4) => BytecodeArg { operator: Load, arg: Variable(102, "println") }
IP: (6, 5) => BytecodeArg { operator: Call, arg: CallArgsNumber(1) }
load object as args: FSRObject { obj_id: 5636113072, value: Integer(456), ref_count: 0, cls: "Integer" }
456
IP: (7, 1) => BytecodeArg { operator: Load, arg: Variable(100, "self") }
IP: (7, 2) => BytecodeArg { operator: ReturnValue, arg: None }
IP: (15, 4) => BytecodeArg { operator: Load, arg: Variable(101, "c") }
IP: (15, 5) => BytecodeArg { operator: Assign, arg: None }
IP: (16, 1) => BytecodeArg { operator: Load, arg: ConstString(0, "123") }
IP: (16, 2) => BytecodeArg { operator: Load, arg: Variable(100, "Abc") }
IP: (16, 3) => BytecodeArg { operator: Call, arg: CallArgsNumber(1) }
IP: (2, 1) => BytecodeArg { operator: AssignArgs, arg: Variable(100, "self") }
IP: (2, 2) => BytecodeArg { operator: AssignArgs, arg: Variable(101, "abc") }
IP: (3, 1) => BytecodeArg { operator: Load, arg: ConstInteger(0, 123) }
IP: (3, 2) => BytecodeArg { operator: Load, arg: Variable(100, "self") }
IP: (3, 3) => BytecodeArg { operator: Load, arg: Attr(101, "abc") }
IP: (3, 4) => BytecodeArg { operator: BinaryDot, arg: None }
IP: (3, 5) => BytecodeArg { operator: Assign, arg: None }
IP: (4, 1) => BytecodeArg { operator: Load, arg: Variable(100, "self") }
IP: (4, 2) => BytecodeArg { operator: Load, arg: Attr(101, "abc") }
IP: (4, 3) => BytecodeArg { operator: BinaryDot, arg: None }
IP: (4, 4) => BytecodeArg { operator: Load, arg: Variable(102, "println") }
IP: (4, 5) => BytecodeArg { operator: Call, arg: CallArgsNumber(1) }
load object as args: FSRObject { obj_id: 5636113408, value: Integer(123), ref_count: 0, cls: "Integer" }
123
IP: (5, 1) => BytecodeArg { operator: Load, arg: ConstInteger(0, 456) }
IP: (5, 2) => BytecodeArg { operator: Load, arg: Variable(100, "self") }
IP: (5, 3) => BytecodeArg { operator: Load, arg: Attr(101, "abc") }
IP: (5, 4) => BytecodeArg { operator: BinaryDot, arg: None }
IP: (5, 5) => BytecodeArg { operator: Assign, arg: None }
IP: (6, 1) => BytecodeArg { operator: Load, arg: Variable(100, "self") }
IP: (6, 2) => BytecodeArg { operator: Load, arg: Attr(101, "abc") }
IP: (6, 3) => BytecodeArg { operator: BinaryDot, arg: None }
IP: (6, 4) => BytecodeArg { operator: Load, arg: Variable(102, "println") }
IP: (6, 5) => BytecodeArg { operator: Call, arg: CallArgsNumber(1) }
load object as args: FSRObject { obj_id: 5636113760, value: Integer(456), ref_count: 0, cls: "Integer" }
456
IP: (7, 1) => BytecodeArg { operator: Load, arg: Variable(100, "self") }
IP: (7, 2) => BytecodeArg { operator: ReturnValue, arg: None }
IP: (16, 4) => BytecodeArg { operator: Load, arg: Variable(102, "b") }
IP: (16, 5) => BytecodeArg { operator: Assign, arg: None }
IP: (17, 1) => BytecodeArg { operator: Load, arg: Variable(101, "c") }
IP: (17, 2) => BytecodeArg { operator: Load, arg: Variable(102, "b") }
IP: (17, 3) => BytecodeArg { operator: Load, arg: Attr(103, "dd") }
IP: (17, 4) => BytecodeArg { operator: BinaryDot, arg: None }
IP: (17, 5) => BytecodeArg { operator: Assign, arg: None }
IP: (18, 1) => BytecodeArg { operator: Load, arg: Variable(102, "b") }
IP: (18, 2) => BytecodeArg { operator: Load, arg: Attr(103, "dd") }
IP: (18, 3) => BytecodeArg { operator: BinaryDot, arg: None }
IP: (18, 4) => BytecodeArg { operator: Load, arg: Attr(104, "abc") }
IP: (18, 5) => BytecodeArg { operator: BinaryDot, arg: None }
IP: (18, 6) => BytecodeArg { operator: Load, arg: Variable(105, "dump") }
IP: (18, 7) => BytecodeArg { operator: Call, arg: CallArgsNumber(1) }
load object as args: FSRObject { obj_id: 5636113072, value: Integer(456), ref_count: 0, cls: "Integer" }
FSRObject {
    obj_id: 5636113072,
    value: Integer(
        456,
    ),
    ref_count: 0,
    cls: "Integer",
}
IP: (19, 1) => BytecodeArg { operator: Load, arg: ConstInteger(0, 1) }
IP: (19, 2) => BytecodeArg { operator: Load, arg: Variable(106, "a") }
IP: (19, 3) => BytecodeArg { operator: Assign, arg: None }
IP: (20, 1) => BytecodeArg { operator: Load, arg: Variable(106, "a") }
IP: (20, 2) => BytecodeArg { operator: Load, arg: ConstInteger(0, 3) }
IP: (20, 3) => BytecodeArg { operator: CompareTest, arg: Compare("<") }
IP: (20, 4) => BytecodeArg { operator: WhileTest, arg: WhileTest(2) }
IP: (21, 1) => BytecodeArg { operator: Load, arg: Variable(106, "a") }
IP: (21, 2) => BytecodeArg { operator: Load, arg: Variable(107, "println") }
IP: (21, 3) => BytecodeArg { operator: Call, arg: CallArgsNumber(1) }
load object as args: FSRObject { obj_id: 5636113872, value: Integer(1), ref_count: 1, cls: "Integer" }
1
IP: (22, 1) => BytecodeArg { operator: Load, arg: Variable(106, "a") }
IP: (22, 2) => BytecodeArg { operator: Load, arg: ConstInteger(0, 1) }
IP: (22, 3) => BytecodeArg { operator: BinaryAdd, arg: None }
IP: (22, 4) => BytecodeArg { operator: Load, arg: Variable(106, "a") }
IP: (22, 5) => BytecodeArg { operator: Assign, arg: None }
IP: (22, 6) => BytecodeArg { operator: WhileBlockEnd, arg: WhileEnd(2) }
IP: (20, 1) => BytecodeArg { operator: Load, arg: Variable(106, "a") }
IP: (20, 2) => BytecodeArg { operator: Load, arg: ConstInteger(0, 3) }
IP: (20, 3) => BytecodeArg { operator: CompareTest, arg: Compare("<") }
IP: (20, 4) => BytecodeArg { operator: WhileTest, arg: WhileTest(2) }
IP: (21, 1) => BytecodeArg { operator: Load, arg: Variable(106, "a") }
IP: (21, 2) => BytecodeArg { operator: Load, arg: Variable(107, "println") }
IP: (21, 3) => BytecodeArg { operator: Call, arg: CallArgsNumber(1) }
load object as args: FSRObject { obj_id: 5636114208, value: Integer(2), ref_count: 0, cls: "Integer" }
2
IP: (22, 1) => BytecodeArg { operator: Load, arg: Variable(106, "a") }
IP: (22, 2) => BytecodeArg { operator: Load, arg: ConstInteger(0, 1) }
IP: (22, 3) => BytecodeArg { operator: BinaryAdd, arg: None }
IP: (22, 4) => BytecodeArg { operator: Load, arg: Variable(106, "a") }
IP: (22, 5) => BytecodeArg { operator: Assign, arg: None }
IP: (22, 6) => BytecodeArg { operator: WhileBlockEnd, arg: WhileEnd(2) }
IP: (20, 1) => BytecodeArg { operator: Load, arg: Variable(106, "a") }
IP: (20, 2) => BytecodeArg { operator: Load, arg: ConstInteger(0, 3) }
IP: (20, 3) => BytecodeArg { operator: CompareTest, arg: Compare("<") }
IP: (20, 4) => BytecodeArg { operator: WhileTest, arg: WhileTest(2) }
IP: (23, 1) => BytecodeArg { operator: Load, arg: Variable(106, "a") }
IP: (23, 2) => BytecodeArg { operator: Load, arg: Variable(107, "println") }
IP: (23, 3) => BytecodeArg { operator: Call, arg: CallArgsNumber(1) }
load object as args: FSRObject { obj_id: 5636114688, value: Integer(3), ref_count: 0, cls: "Integer" }
3
```
