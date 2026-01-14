struct Test {
    field: u64
}

@entry
fn test() -> u64 {
    t: Ptr[u64] = Test.alloc(3)
    t[1] = 42
    return t[1]
}

a = test()
println(f"a: {a}")
assert(a == 42, "alloc_array: testcase1: a should be 42")