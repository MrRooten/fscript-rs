class Test {
    fn __new__(self) {
        self.abc = [1, 2, 3]
    }
}

t = Test()
t.abc[0] += 1
println(t.abc[0])