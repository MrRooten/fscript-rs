struct Test {
    field: u64
}

@entry
fn test() -> Ptr[Test] {
    a: Ptr[Test] = Test.alloc
    return a
}