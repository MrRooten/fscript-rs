@jit
fn abc() {
    a = 0
    while a < 2000000 {
        a = a + 1
    }
    println(a)
}

abc()