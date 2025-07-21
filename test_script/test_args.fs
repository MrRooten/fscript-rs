class Test {
    fn __new__(self) {
        self.value = [
            "abc",
            "ddc"
        ]
        return self
    }
}

t = Test()
println(t.value[0][0])