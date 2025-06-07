
class Abc {
    @jit
    fn __add__(self, other: Abc) -> Integer {
        #println("add")
        return 1
    }
}

@jit
fn jit_test(n) {
    println(n + n)
}

a = Abc()
jit_test(a)