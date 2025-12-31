@static
fn abc() {
    a: u64 = 1 + 2

    while a < 300000000 {
        a = a + 1
    }
}

abc()