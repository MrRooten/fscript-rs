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
}

a = Abc()
println(a)

if a.abc.ddc < 323 {
    a.abc.ddc = a.test() + a.abc.ddc
}

println(a.abc.ddc)