
class Abc {
    @jit
    fn __add__(self, other: Abc) -> Integer {
        #println("add")
        return 1
    }
}

@jit
fn jit_test(n) {
    for i in 0..3000000 {
        c = n + n
    }
}

a = Abc()
jit_test(a)