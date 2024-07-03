class Abc {
    fn __new__(self) {
        self.abc = 123
        return self
    }

    fn __str__(self) {
        return 'return string'
    }

    fn test(self) {
        dump(self)
    }
}

a = Abc()
println(a)

println("Get a.abc")
println(a.abc)

if a.abc > 3 {
    println('a.abc > 3')
    a.test()
}

