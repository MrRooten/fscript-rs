struct Test {
    field: u64
    field2: u64

    @static
    fn __new__() {

    }
}



@entry
fn test() -> u64 {
    b: u64 = simple()
    a: u64 = 1
    return b
}

@static
fn simple() -> u64 {
    a: u64 = 1
    return a
}

a = test()
println(a)