struct String {
    inner: Ptr[u8]
    length: u64
}

@entry
fn test() -> u8 {
    t: Ptr[u8] = "abcd"
    a: u64 = 4
    s: String = uninit
    s.inner = t
    s.length = a
    c: u8 = s.inner[0]

    return c
}

a = test()
println(f"a == {a}")
