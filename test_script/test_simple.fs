@static
fn abc() {
    a: u64 = 0
    while a < 300000000 {
        a = a + 1
    }
}

abc()