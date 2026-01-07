@static
fn simple(n: u64) -> u64 {
    a: u64 = 1
    return n
}

@entry
fn test() -> u64 {
    a: u64 = 2
    b: u64 = simple(3)
    
    return b
}


a = test()
println(a)