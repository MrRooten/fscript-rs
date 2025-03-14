c = "abc" + "def"
println(c)
fn abc() {
    return 'abc'.len()
}

a = ((1 + 3 + 1 ) * 3 + 4 + abc()) * 4
assert(a == 88)

class Abc {
    fn __new__(self) {
        self.abc = 123
        return self
    }
}

abc = Abc()
println(abc)
assert(abc.abc == 123)
true = 1 == 1

assert(true)

false = 1 != 1

assert(!false)