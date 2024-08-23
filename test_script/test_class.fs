class Abc {
    fn __new__(self) {
        self.abc = 0
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

if a.abc < 300 {
    a.abc = a.abc + 1 + a.abc + a.test()
}

println(a.abc)