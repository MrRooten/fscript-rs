struct Test {
    field: u64

    @static
    fn init(self: Ptr[Test], n: u64) {
        self.field = n
    }
}

@entry
fn test() -> u64 {
    t: Ptr[Test] = Test.alloc
    t.init(44444)
    return t.field
}

a = test()
println(f"a: {a}")
assert(a == 44444, "test_simple: testcase1: a should be 44444")