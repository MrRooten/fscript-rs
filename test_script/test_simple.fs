struct Test {
    field: u64
    field2: u64
    field3: u64
    field4: u64
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
    a: u64 = 0
    while a < 30000000 {
        t = simple()
        a = a + 1
    }
    return t.field
}

a = test()
println(a)