class Abc {
    fn __new__(self) {
        self.abc = 0
        return self
    }

    fn test(self, a, b) {
        println(a)
        println(b)
    }
}

fn abc(a, b, c) {
    assert(a == "will print a", "arg a error")
    println(a)
    assert(b == "will print b", "arg b error")
    println(b)
    assert(c == "will print c", "arg c error")
    println(c)
}

a = Abc()
a.test(1, 2)

abc("will print a", "will print b", "will print c")