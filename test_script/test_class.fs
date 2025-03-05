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
}

a = Abc()
println(a.__str__()) # will prin 'return string'

if a.abc.ddc < 323 {
    a.abc.ddc = a.test() + a.abc.ddc
}

assert(a.abc.ddc == 447)

println(a.abc.ddc)

Abc::not_self()

println(Abc::__str__(a))