

@jit
fn jit_test() {
    a = 0
    while a < 5 {
        
        if a == 1 {
            a = a + 1
            continue
        }
        println(a)
        a = a + 1
    }
}

jit_test()