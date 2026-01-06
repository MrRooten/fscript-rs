struct Test {
    field1: i32
    field2: u64

    @static
    fn __new__(self, a: u64) {
        self.field1 = 0
        self.field2 = 0
    }
}
