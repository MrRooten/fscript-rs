@jit
fn abc() {
    a = 0
    while a < 3000000 {
        a = a + 1
    }
    println(a)
}

abc()