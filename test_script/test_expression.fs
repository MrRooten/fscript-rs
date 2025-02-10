c = "abc" + "def"
println(c)
fn abc() {
    return 'abc'.len()
}

a = ((1 + 3 + 1 ) * 3 + 4 + abc()) * 4
println(a)

class Abc {
    fn __new__(self) {
        self.abc = 123
        return self
    }
}

abc = Abc()
println(abc)

true = 1 == 1

if true {
    println("true")
}

false = 1 != 1

if not false {
    println("not false")
}