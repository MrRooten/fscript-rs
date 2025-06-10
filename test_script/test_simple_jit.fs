
class Abc {
    @jit
    fn __add__(self, abc) {
        println("test: ", self, abc)
        return 1
    }
}

@jit
fn jit_test() {
    a = 0
    while a < 3000000 {
        a = a + 1

    }

    println("jit_test done: ", a)
}

a = Abc()
jit_test()