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
    println(a)
    println(b)
    println(c)
}

a = Abc()
a.test(1, 2)

abc("will print a", "will print b", "will print c")