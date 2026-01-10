
@entry
fn test(n: u64) -> u64 {
    a: [u64, 8] = uninit
    a[0] = 1
    c: u64 = a[0]
    b: u64 = 0
    return c
}

a = test(10)
println(a)