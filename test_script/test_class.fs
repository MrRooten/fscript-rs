class Abc {
    fn __new__(self) {
        self.abc = 123
        return self
    }

    fn __str__(self) {
        return 'return string'
    }
}

a = Abc()
println(a)

println("Get a.abc")
println(a.abc)

if a.abc > 3 {
    println('a.abc > 3')
}