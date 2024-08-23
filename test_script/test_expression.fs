fn abc() {
    return 'abc'.len()
}

a = (1 + 3 + 1 ) * 3 + 4 + abc()
println(a)

class Abc {
    fn __new__(self) {
        self.abc = 123
        return self
    }
}

abc = Abc()
println(abc)