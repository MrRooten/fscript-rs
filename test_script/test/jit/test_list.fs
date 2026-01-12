struct Test {
    field: u64
    field2: u64
}

@static
fn simple() -> [u64, 4] {
    t: [u64, 4] = uninit
    t[0] = 1002
    return t
}

@entry
fn test() -> u64 {
    t: [u64, 4] = simple()
    v: u64 = t[0]
    return v
}

a = test()
println(a)