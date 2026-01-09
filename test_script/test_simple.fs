struct Test {
    field: u64
}

@entry
fn test() -> u64 {
    a: Ptr[Test] = Test.alloc
    a.field = 42
    return a.field
}

a = test()
println(a)