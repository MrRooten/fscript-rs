

@entry
fn test() -> u8 {
    t: Ptr[u8] = "abcd"
    a: u8 = t[0]
    return a
}

a = test()
println(f"a == {a}")
