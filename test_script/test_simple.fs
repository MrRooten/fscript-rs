struct Test {
    field: u64
    field2: u64
}

@static
fn simple() -> Test {
    t: Test = uninit
    t.field = 1002
    return t
}

@entry
fn test() -> u64 {
    t: Test = simple()
    return t.field
}

a = test()
println(a)