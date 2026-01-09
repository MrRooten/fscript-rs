struct Test {
    field: u64

    @static
    fn __new__(self) {
        self.field = 10
    }
}

@entry
fn test() -> Ptr[Test] {
    a: Ptr[Test] = Test.alloc
    return a
}


a = test()
println(a)