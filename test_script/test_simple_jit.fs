

@jit
fn jit_test() {
    a = 1
    for i in 0..3000000 {
        a.__add__(1)
    }
}

jit_test()