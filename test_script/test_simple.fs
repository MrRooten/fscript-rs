@static
fn simple() -> u64 {
    a: u64 = 5
    return a
}

@static
fn test() -> u64 {
    a: u64 = 2
    b: u64 = simple()
    
    return b
}

a = test()
println(a)