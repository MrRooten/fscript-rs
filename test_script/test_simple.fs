struct Test {
    field: u64
    field2: u64
}

@static
fn simple(n: u64) -> Ptr[Test] {
    ptr: Ptr[Test] = Test.alloc
    ptr.field2 = n
    return ptr
}

@entry
fn test() -> u64 {
    out_ptr: Ptr[Test] = simple(2)
    return out_ptr.field2
}

a = test()
println(a)