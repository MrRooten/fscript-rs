@static
fn test() -> u64 {
    a: u64 = 2
    while a < 3000000 {
        a = a + 1
    }
    
    return a
}

a = test()
println(a)