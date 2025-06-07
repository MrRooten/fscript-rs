
class Abc {
    @jit
    fn __add__(self, other: Abc) -> Integer {
        #println("add")
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