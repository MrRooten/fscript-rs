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

    fn abcd(self) {
        return 'abc'.len()
    }
}


abc = Abc()

c = abc.abc + abc.abc

println(c)
assert(abc.abc == 123)
true = 1 == 1

assert(true)

false = 1 != 1

assert(!false)