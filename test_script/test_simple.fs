class Ddc {
    fn __new__(self) {
        self.ddc = 123 + 1
        return self
    }
}

class Abc {
    fn __new__(self) {
        self.abc = Ddc()
        return self
    }

    fn __str__(self) {
        return 'return string'
    }

    fn test(self) {
        return 323
    }

    fn not_self() {
        println("not self")
        return 1
    }

    fn __add__(self, other) {
        println("add")
        return 1
    }

    fn __gt__(self, other) {
        println("gt")
        return true
    }

    fn more_args(self, a, b, c) {
        assert(a == 1)
        assert(b == 2)
        assert(c == 3)
        println("more args", a, b, c)
        return 1
    }
}



a = Abc()
println(a.__str__()) # will prin 'return string'


if a.abc.ddc < 323 {
    a.abc.ddc = a.test() + a.abc.ddc
}

println(a.abc.ddc)