
struct Test {
    field: u64
    field2: u64
}

@static
fn simple() -> Test {
    t: [Test, 3] = uninit
    t_ptr: Ptr[Test] = Test.alloc
    t_ptr.field = 42
    t[0] = t_ptr
    v: Test = t[0]
    return v
}

@entry
fn test() -> u64 {
    v:Test = simple()

    return v.field
}

a = test()
assert(a == 42)