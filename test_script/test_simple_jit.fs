

@jit
fn jit_test() {
    a = 0
    while a < 5 {
        
        if a == 3 {
            a = a + 1
            break
        }
        println(a)
        a = a + 1
    }
}

jit_test()