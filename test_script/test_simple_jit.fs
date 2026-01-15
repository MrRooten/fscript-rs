struct Test {
    field: u64
}

@static
fn simple(n: u64, t: Ptr[Test]) -> u64 {
    t.field = t.field + 1
    if n <= 1 {
        return n
    }
    return simple(n - 1, t)
}

@entry
fn test() -> u64 {
    a: u64 = 2
    t: Ptr[Test] = Test.alloc
    t.field = 0
    b: u64 = simple(100000, t)
    
    c: u64 = t.field
    return t.field
}

println(test())