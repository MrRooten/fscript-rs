@static
fn simple() {

}

@static
fn test() -> u64 {
    a: u64 = 2
    simple()
    
    return a
}

a = test()
println(a)