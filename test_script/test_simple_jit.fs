
class Abc {
    @jit
    fn __add__(self, abc) {
        println("test: ", self, abc)
        return 1
    }
}

@jit
fn jit_test() {
    a = 10
    if a > 10 {
        println("a is greater than 2")
    } else if a < 5 {
        println("a is less than 5")
    } else if a < 7 {
        println("a is less than 7")
    } else {
        println("a is greater than 7")
    }

    
}

a = Abc()
jit_test()