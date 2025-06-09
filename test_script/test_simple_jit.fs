
class Abc {
    @jit
    fn __add__(self, abc) {
        println("test: ", self, abc)
        return 1
    }
}

@jit
fn jit_test() {
    a = [1, 2, 3]
    println(a)
}

a = Abc()
jit_test()