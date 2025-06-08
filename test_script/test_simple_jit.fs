
class Abc {
    @jit
    fn __add__(self, abc) {
        println("test")
        return 1
    }
}

@jit
fn jit_test(n) {
    n.__add__(n)
}

a = Abc()
jit_test(a)