@static
fn abc() {
    a: u8 = 0
    while a < 30000000 {
        a: u8 = a + 1
    }
}

abc()