struct Test {
    field: u64
    field2: u64
}

@entry
fn test() -> u64 {
    t: Ptr[Test] = Test.alloc
    t.field = 0
    t.field += 1
    return t.field
}

a = test()
println(a)