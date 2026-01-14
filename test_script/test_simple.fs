struct Test {
    field: u64

    @static
    fn init(self: Ptr[Test]) {
        self.field = 44444
    }
}

@entry
fn test() -> u64 {
    t: Ptr[Test] = Test.alloc
    t.init()
    b: u64 = 42
    t.free
    return b
}

a = test()
println(a)