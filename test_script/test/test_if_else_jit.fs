
class Abc {
    @jit
    fn __add__(self, abc) {
        println("test: ", self, abc)
        return 1
    }
}

@jit
fn jit_test() {
    abc = -2

    while abc < 3 {
        if abc > 1 {
            println('> 1: ', abc)
        } else if abc < -1 {
            println('< -1: ', abc)
        } else {
            println('else: ', abc)
        }
        abc = abc + 1
    }
}

a = Abc()
jit_test()