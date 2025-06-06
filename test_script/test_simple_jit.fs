class Ddc {
    fn __new__(self) {
        self.ddc = 123 + 1
        return self
    }
}

class Abc {
    fn __new__(self) -> Abc {
        self.abc = Ddc()
        return self
    }

    fn __str__(self) -> String {
        return 'return string'
    }

    fn test(self) -> Integer {
        return 323
    }

    fn not_self() {
        println("not self")
        return 1
    }

    fn __add__(self, other: Abc) -> Integer {
        #println("add")
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

@jit
fn fib(n) {
    for i in 0..30000000 {
        n + n
    }
}

a = Abc()
fib(a)