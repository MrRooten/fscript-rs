struct Test {
    field: u64
    field2: u64
}

@static
fn simple(t: Ptr[Test]) -> u64 {
    t.field += 1000
    return t.field
}

@entry
fn test() -> u64 {
    t: Ptr[Test] = Test.alloc
    t.field = 2
    a: u64 = 0
    simple(t)
    

    return t.field
}

a = test()
println(a)