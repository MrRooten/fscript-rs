struct Test {
    field: u64
    field2: u64
}



@entry
fn test() -> u64 {
    simple()
    a: u64 = 1
    return a
}

@static
fn simple() {
    
}

a = test()
println(a)